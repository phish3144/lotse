//! Lokaler Speicher: SQLCipher-Datenbank mit Änderungsprotokoll und Volltextindex.
//!
//! Die lokale Datenbank ist die Wahrheit auf diesem Gerät. Jede lokale Schreiboperation
//! bekommt eine HLC und landet im Änderungsprotokoll (`aenderungen`), aus dem der
//! Sync-Client Umschläge baut. Von außen (Pull) übernommene Datensätze werden **nicht**
//! ins Änderungsprotokoll geschrieben, sonst würden sie endlos zirkulieren.
//!
//! Schlüssel: Aus dem Account-Schlüssel werden der SQLCipher-Schlüssel (`lotse/local-db`)
//! und der Datensatz-Schlüssel für Umschläge (`lotse/records`) abgeleitet.

use std::path::Path;

use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::brief::{self, Auffaelligkeit};
use crate::crypto::{self, Key32};
use crate::hlc::Hlc;
use crate::model::*;
use crate::sync::{self, RecordKind, SyncState, Umschlag};
use crate::vault::TresorEintrag;
use crate::{now_ms, Error, Result, SCHEMA_VERSION};

pub struct Store {
    conn: Connection,
    hlc: Hlc,
    records_key: Key32,
    device_id: Ulid,
}

/// Eine Karte im Hafen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HafenKarte {
    pub projekt: Projekt,
    pub letzte_notiz: Option<Notiz>,
    pub offene_faeden: usize,
    pub auffaelligkeit: Auffaelligkeit,
    pub tage_seit: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Treffer {
    pub kind: String,
    pub ref_id: Ulid,
    pub projekt_id: Option<Ulid>,
    pub ausschnitt: String,
}

/// Ergebnis der Anwendung eines fremden Umschlags.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Angewendet {
    Uebernommen,
    /// Lokale Fassung war neuer; Umschlag verworfen.
    Verworfen,
    /// Übernommen, aber die lokale Fassung wich ab; Konfliktnotiz geschrieben.
    Konflikt,
}

const MIGRATIONEN: &[&str] = &[
    // 1
    r#"
    CREATE TABLE meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
    CREATE TABLE projekte (
        id TEXT PRIMARY KEY, titel TEXT NOT NULL, kurs TEXT NOT NULL DEFAULT '',
        status TEXT NOT NULL, wiedervorlage TEXT, erwartungsintervall_tage INTEGER NOT NULL,
        tags TEXT NOT NULL DEFAULT '[]', vorlage TEXT NOT NULL, abgeleitet_von TEXT,
        angelegt INTEGER NOT NULL, zuletzt_beruehrt INTEGER NOT NULL,
        hlc TEXT NOT NULL, deleted INTEGER NOT NULL DEFAULT 0
    );
    CREATE TABLE notizen (
        id TEXT PRIMARY KEY, projekt_id TEXT NOT NULL, ts INTEGER NOT NULL,
        quelle TEXT NOT NULL, art TEXT NOT NULL, text TEXT NOT NULL, erledigt_am INTEGER,
        hlc TEXT NOT NULL, deleted INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX notizen_projekt ON notizen(projekt_id, ts);
    CREATE INDEX notizen_offen ON notizen(art, erledigt_am) WHERE art = 'offen';
    CREATE TABLE referenzen (
        id TEXT PRIMARY KEY, projekt_id TEXT NOT NULL, typ TEXT NOT NULL, ziel TEXT NOT NULL,
        rolle TEXT NOT NULL, geraet_id TEXT, zuletzt_geprueft INTEGER, pruefstatus TEXT NOT NULL,
        hlc TEXT NOT NULL, deleted INTEGER NOT NULL DEFAULT 0
    );
    CREATE INDEX referenzen_projekt ON referenzen(projekt_id);
    CREATE TABLE tresor (
        id TEXT PRIMARY KEY, titel TEXT NOT NULL, projekt_ids TEXT NOT NULL DEFAULT '[]',
        stufe TEXT NOT NULL, json TEXT NOT NULL, angelegt INTEGER NOT NULL, geaendert INTEGER NOT NULL,
        hlc TEXT NOT NULL, deleted INTEGER NOT NULL DEFAULT 0
    );
    CREATE TABLE geraete (
        id TEXT PRIMARY KEY, json TEXT NOT NULL, hlc TEXT NOT NULL, deleted INTEGER NOT NULL DEFAULT 0
    );
    CREATE TABLE aenderungen (
        local_seq INTEGER PRIMARY KEY AUTOINCREMENT, record_id TEXT NOT NULL, kind TEXT NOT NULL,
        hlc TEXT NOT NULL, deleted INTEGER NOT NULL DEFAULT 0
    );
    CREATE TABLE kandidaten (
        pfad TEXT PRIMARY KEY, json TEXT NOT NULL, gesehen INTEGER NOT NULL, verworfen INTEGER NOT NULL DEFAULT 0
    );
    CREATE VIRTUAL TABLE suche USING fts5(
        text, kind UNINDEXED, ref_id UNINDEXED, projekt_id UNINDEXED,
        tokenize = 'unicode61 remove_diacritics 2'
    );
    "#,
];

impl Store {
    /// Öffnet (oder erzeugt) die Datenbank. Ein falscher Schlüssel führt zu einem Fehler
    /// beim ersten Lesen, nie zu einer stillen Neuanlage.
    pub fn open(path: &Path, account_key: &Key32, device_id: Ulid) -> Result<Store> {
        let conn = Connection::open(path)?;
        Self::init(conn, account_key, device_id)
    }

    /// Nur für Tests: Datenbank im Speicher, trotzdem mit SQLCipher-Schlüssel.
    pub fn open_in_memory(account_key: &Key32, device_id: Ulid) -> Result<Store> {
        let conn = Connection::open_in_memory()?;
        Self::init(conn, account_key, device_id)
    }

    fn init(conn: Connection, account_key: &Key32, device_id: Ulid) -> Result<Store> {
        let db_key = account_key.subkey(crypto::info::LOCAL_DB);
        let hex = db_key.to_hex();
        conn.pragma_update(None, "key", format!("x'{}'", hex.as_str()))?;
        // Ein falscher Schlüssel fällt hier auf ("file is not a database").
        conn.query_row("SELECT count(*) FROM sqlite_master", [], |_| Ok(()))?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;

        let mut store = Store {
            conn,
            hlc: Hlc::new(device_id),
            records_key: account_key.subkey(crypto::info::RECORDS),
            device_id,
        };
        store.migrate()?;
        let largest: Option<String> = store.conn.query_row(
            "SELECT max(hlc) FROM (
                SELECT hlc FROM projekte UNION ALL SELECT hlc FROM notizen UNION ALL
                SELECT hlc FROM referenzen UNION ALL SELECT hlc FROM tresor UNION ALL
                SELECT hlc FROM geraete UNION ALL SELECT hlc FROM aenderungen)",
            [],
            |r| r.get(0),
        )?;
        store.hlc.restore(largest.as_deref())?;
        Ok(store)
    }

    fn migrate(&mut self) -> Result<()> {
        let hat_meta: bool = self
            .conn
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='meta'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)?;
        let mut version: u32 = if hat_meta {
            self.meta_get("schema_version")?
                .and_then(|v| v.parse().ok())
                .unwrap_or(0)
        } else {
            0
        };
        while (version as usize) < MIGRATIONEN.len() {
            let tx = self.conn.transaction()?;
            tx.execute_batch(MIGRATIONEN[version as usize])?;
            version += 1;
            tx.execute(
                "INSERT INTO meta(key, value) VALUES ('schema_version', ?1)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![version.to_string()],
            )?;
            tx.commit()?;
        }
        if version != SCHEMA_VERSION {
            return Err(Error::Other(format!(
                "Schema-Version {version} passt nicht zum Kern ({SCHEMA_VERSION})"
            )));
        }
        Ok(())
    }

    pub fn device_id(&self) -> Ulid {
        self.device_id
    }

    // ---------------------------------------------------------------- meta

    pub fn meta_get(&self, key: &str) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| {
                r.get(0)
            })
            .optional()?)
    }

    pub fn meta_set(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO meta(key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    // ------------------------------------------------------------- projekte

    fn projekt_aus_row(r: &Row) -> rusqlite::Result<Projekt> {
        let tags: String = r.get("tags")?;
        Ok(Projekt {
            id: ulid_col(r, "id")?,
            titel: r.get("titel")?,
            kurs: r.get("kurs")?,
            status: Status::parse(&r.get::<_, String>("status")?).unwrap_or(Status::Aktiv),
            wiedervorlage: r.get("wiedervorlage")?,
            erwartungsintervall_tage: r.get::<_, i64>("erwartungsintervall_tage")? as u32,
            tags: serde_json::from_str(&tags).unwrap_or_default(),
            vorlage: Vorlage::parse(&r.get::<_, String>("vorlage")?).unwrap_or(Vorlage::Generisch),
            abgeleitet_von: opt_ulid_col(r, "abgeleitet_von")?,
            angelegt: r.get("angelegt")?,
            zuletzt_beruehrt: r.get("zuletzt_beruehrt")?,
        })
    }

    fn projekt_row_schreiben(&self, p: &Projekt, hlc: &str, deleted: bool) -> Result<()> {
        self.conn.execute(
            "INSERT INTO projekte (id, titel, kurs, status, wiedervorlage, erwartungsintervall_tage,
                tags, vorlage, abgeleitet_von, angelegt, zuletzt_beruehrt, hlc, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET titel=excluded.titel, kurs=excluded.kurs,
                status=excluded.status, wiedervorlage=excluded.wiedervorlage,
                erwartungsintervall_tage=excluded.erwartungsintervall_tage, tags=excluded.tags,
                vorlage=excluded.vorlage, abgeleitet_von=excluded.abgeleitet_von,
                angelegt=excluded.angelegt, zuletzt_beruehrt=excluded.zuletzt_beruehrt,
                hlc=excluded.hlc, deleted=excluded.deleted",
            params![
                p.id.to_string(),
                p.titel,
                p.kurs,
                p.status.as_str(),
                p.wiedervorlage,
                p.erwartungsintervall_tage as i64,
                serde_json::to_string(&p.tags)?,
                p.vorlage.as_str(),
                p.abgeleitet_von.map(|u| u.to_string()),
                p.angelegt,
                p.zuletzt_beruehrt,
                hlc,
                deleted as i64,
            ],
        )?;
        self.suche_ersetzen(
            "projekt",
            p.id,
            Some(p.id),
            &format!("{} {} {}", p.titel, p.kurs, p.tags.join(" ")),
            deleted,
        )?;
        Ok(())
    }

    /// Legt ein Projekt an oder überschreibt es (lokale Änderung).
    pub fn projekt_speichern(&mut self, p: &Projekt) -> Result<()> {
        let hlc = self.hlc.next(now_ms());
        self.projekt_row_schreiben(p, &hlc, false)?;
        self.aenderung(RecordKind::Project, p.id, &hlc, false)
    }

    pub fn projekt(&self, id: Ulid) -> Result<Option<Projekt>> {
        Ok(self
            .conn
            .query_row(
                "SELECT * FROM projekte WHERE id = ?1 AND deleted = 0",
                params![id.to_string()],
                Self::projekt_aus_row,
            )
            .optional()?)
    }

    pub fn projekt_nach_titel(&self, titel: &str) -> Result<Option<Projekt>> {
        Ok(self
            .conn
            .query_row(
                "SELECT * FROM projekte WHERE deleted = 0 AND lower(titel) = lower(?1) LIMIT 1",
                params![titel],
                Self::projekt_aus_row,
            )
            .optional()?)
    }

    pub fn projekte(&self) -> Result<Vec<Projekt>> {
        let mut st = self
            .conn
            .prepare("SELECT * FROM projekte WHERE deleted = 0 ORDER BY zuletzt_beruehrt DESC")?;
        let rows = st.query_map([], Self::projekt_aus_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Löscht ein Projekt samt Notizen und Referenzen (Tombstones für den Sync).
    pub fn projekt_loeschen(&mut self, id: Ulid) -> Result<()> {
        let Some(p) = self.projekt(id)? else {
            return Err(Error::NotFound(format!("Projekt {id}")));
        };
        let hlc = self.hlc.next(now_ms());
        self.projekt_row_schreiben(&p, &hlc, true)?;
        self.aenderung(RecordKind::Project, id, &hlc, true)?;
        for n in self.notizen(id)? {
            let hlc = self.hlc.next(now_ms());
            self.conn.execute(
                "UPDATE notizen SET deleted = 1, hlc = ?2 WHERE id = ?1",
                params![n.id.to_string(), hlc],
            )?;
            self.suche_ersetzen("notiz", n.id, Some(id), "", true)?;
            self.aenderung(RecordKind::Note, n.id, &hlc, true)?;
        }
        for r in self.referenzen(id)? {
            let hlc = self.hlc.next(now_ms());
            self.conn.execute(
                "UPDATE referenzen SET deleted = 1, hlc = ?2 WHERE id = ?1",
                params![r.id.to_string(), hlc],
            )?;
            self.aenderung(RecordKind::Reference, r.id, &hlc, true)?;
        }
        Ok(())
    }

    /// Das Postkorb-Projekt für unzugeordnete Gedanken; wird bei Bedarf angelegt.
    pub fn postkorb(&mut self) -> Result<Projekt> {
        if let Some(id) = self.meta_get("postkorb_id")? {
            if let Some(p) = Ulid::from_string(&id)
                .ok()
                .and_then(|u| self.projekt(u).ok().flatten())
            {
                return Ok(p);
            }
        }
        let mut p = Projekt::neu("Postkorb", Vorlage::Generisch, now_ms());
        p.kurs = "Unzugeordnete Gedanken. Regelmäßig leeren.".into();
        p.erwartungsintervall_tage = 3650;
        self.projekt_speichern(&p)?;
        self.meta_set("postkorb_id", &p.id.to_string())?;
        Ok(p)
    }

    /// Statuswechsel. Verlangt eine Übergabenotiz beim Wechsel auf `pausiert`/`wartet`.
    pub fn status_setzen(
        &mut self,
        projekt_id: Ulid,
        neu: Status,
        uebergabe: Option<&str>,
        wiedervorlage: Option<&str>,
        quelle: Quelle,
    ) -> Result<()> {
        let mut p = self
            .projekt(projekt_id)?
            .ok_or_else(|| Error::NotFound(format!("Projekt {projekt_id}")))?;
        if neu.verlangt_uebergabe() && uebergabe.map(str::trim).unwrap_or("").is_empty() {
            return Err(Error::Invalid(
                "Wechsel auf »pausiert« oder »wartet« braucht eine Übergabenotiz".into(),
            ));
        }
        let alt = p.status;
        let jetzt = now_ms();
        p.status = neu;
        p.wiedervorlage = wiedervorlage
            .map(|s| s.to_string())
            .or(if neu.verlangt_uebergabe() {
                p.wiedervorlage.clone()
            } else {
                None
            });
        p.zuletzt_beruehrt = jetzt;
        self.projekt_speichern(&p)?;
        self.notiz_speichern(&Notiz::neu(
            projekt_id,
            quelle,
            Art::Status,
            format!("{} → {}", alt.as_str(), neu.as_str()),
            jetzt,
        ))?;
        if let Some(text) = uebergabe.map(str::trim).filter(|t| !t.is_empty()) {
            self.notiz_speichern(&Notiz::neu(
                projekt_id,
                quelle,
                Art::Uebergabe,
                text,
                jetzt + 1,
            ))?;
        }
        Ok(())
    }

    // -------------------------------------------------------------- notizen

    fn notiz_aus_row(r: &Row) -> rusqlite::Result<Notiz> {
        Ok(Notiz {
            id: ulid_col(r, "id")?,
            projekt_id: ulid_col(r, "projekt_id")?,
            ts: r.get("ts")?,
            quelle: Quelle::parse(&r.get::<_, String>("quelle")?).unwrap_or(Quelle::Mensch),
            art: Art::parse(&r.get::<_, String>("art")?).unwrap_or(Art::Log),
            text: r.get("text")?,
            erledigt_am: r.get("erledigt_am")?,
        })
    }

    fn notiz_row_schreiben(&self, n: &Notiz, hlc: &str, deleted: bool) -> Result<()> {
        self.conn.execute(
            "INSERT INTO notizen (id, projekt_id, ts, quelle, art, text, erledigt_am, hlc, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET projekt_id=excluded.projekt_id, ts=excluded.ts,
                quelle=excluded.quelle, art=excluded.art, text=excluded.text,
                erledigt_am=excluded.erledigt_am, hlc=excluded.hlc, deleted=excluded.deleted",
            params![
                n.id.to_string(),
                n.projekt_id.to_string(),
                n.ts,
                n.quelle.as_str(),
                n.art.as_str(),
                n.text,
                n.erledigt_am,
                hlc,
                deleted as i64
            ],
        )?;
        self.suche_ersetzen("notiz", n.id, Some(n.projekt_id), &n.text, deleted)?;
        self.conn.execute(
            "UPDATE projekte SET zuletzt_beruehrt = max(zuletzt_beruehrt, ?2) WHERE id = ?1",
            params![n.projekt_id.to_string(), n.ts],
        )?;
        Ok(())
    }

    /// Schreibt eine Notiz (lokale Änderung).
    pub fn notiz_speichern(&mut self, n: &Notiz) -> Result<()> {
        if n.text.trim().is_empty() {
            return Err(Error::Invalid("Leere Notiz".into()));
        }
        let hlc = self.hlc.next(now_ms());
        self.notiz_row_schreiben(n, &hlc, false)?;
        self.aenderung(RecordKind::Note, n.id, &hlc, false)
    }

    pub fn notiz(&self, id: Ulid) -> Result<Option<Notiz>> {
        Ok(self
            .conn
            .query_row(
                "SELECT * FROM notizen WHERE id = ?1 AND deleted = 0",
                params![id.to_string()],
                Self::notiz_aus_row,
            )
            .optional()?)
    }

    /// Alle Notizen eines Projekts, chronologisch.
    pub fn notizen(&self, projekt_id: Ulid) -> Result<Vec<Notiz>> {
        let mut st = self.conn.prepare(
            "SELECT * FROM notizen WHERE projekt_id = ?1 AND deleted = 0 ORDER BY ts ASC, id ASC",
        )?;
        let rows = st.query_map(params![projekt_id.to_string()], Self::notiz_aus_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn letzte_notiz(&self, projekt_id: Ulid) -> Result<Option<Notiz>> {
        Ok(self
            .conn
            .query_row(
                "SELECT * FROM notizen WHERE projekt_id = ?1 AND deleted = 0 ORDER BY ts DESC, id DESC LIMIT 1",
                params![projekt_id.to_string()],
                Self::notiz_aus_row,
            )
            .optional()?)
    }

    /// Markiert einen offenen Faden als erledigt.
    pub fn faden_erledigen(&mut self, id: Ulid) -> Result<()> {
        let mut n = self
            .notiz(id)?
            .ok_or_else(|| Error::NotFound(format!("Notiz {id}")))?;
        if n.art != Art::Offen {
            return Err(Error::Invalid(
                "Nur offene Fäden können erledigt werden".into(),
            ));
        }
        n.erledigt_am = Some(now_ms());
        self.notiz_speichern(&n)
    }

    /// Alle offenen Fäden über alle Projekte, älteste zuerst.
    pub fn offene_faeden(&self, nur_projekt: Option<Ulid>) -> Result<Vec<Notiz>> {
        let mut st = self.conn.prepare(
            "SELECT n.* FROM notizen n JOIN projekte p ON p.id = n.projekt_id
             WHERE n.art = 'offen' AND n.erledigt_am IS NULL AND n.deleted = 0 AND p.deleted = 0
               AND (?1 IS NULL OR n.projekt_id = ?1)
             ORDER BY n.ts ASC",
        )?;
        let rows = st.query_map(
            params![nur_projekt.map(|u| u.to_string())],
            Self::notiz_aus_row,
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    // ----------------------------------------------------------- referenzen

    fn referenz_aus_row(r: &Row) -> rusqlite::Result<Referenz> {
        Ok(Referenz {
            id: ulid_col(r, "id")?,
            projekt_id: ulid_col(r, "projekt_id")?,
            typ: ReferenzTyp::parse(&r.get::<_, String>("typ")?).unwrap_or(ReferenzTyp::Url),
            ziel: r.get("ziel")?,
            rolle: Rolle::parse(&r.get::<_, String>("rolle")?).unwrap_or(Rolle::Material),
            geraet_id: opt_ulid_col(r, "geraet_id")?,
            zuletzt_geprueft: r.get("zuletzt_geprueft")?,
            pruefstatus: Pruefstatus::parse(&r.get::<_, String>("pruefstatus")?)
                .unwrap_or(Pruefstatus::NichtPruefbar),
        })
    }

    fn referenz_row_schreiben(&self, x: &Referenz, hlc: &str, deleted: bool) -> Result<()> {
        self.conn.execute(
            "INSERT INTO referenzen (id, projekt_id, typ, ziel, rolle, geraet_id, zuletzt_geprueft, pruefstatus, hlc, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET projekt_id=excluded.projekt_id, typ=excluded.typ, ziel=excluded.ziel,
                rolle=excluded.rolle, geraet_id=excluded.geraet_id, zuletzt_geprueft=excluded.zuletzt_geprueft,
                pruefstatus=excluded.pruefstatus, hlc=excluded.hlc, deleted=excluded.deleted",
            params![
                x.id.to_string(),
                x.projekt_id.to_string(),
                x.typ.as_str(),
                x.ziel,
                x.rolle.as_str(),
                x.geraet_id.map(|u| u.to_string()),
                x.zuletzt_geprueft,
                x.pruefstatus.as_str(),
                hlc,
                deleted as i64
            ],
        )?;
        self.suche_ersetzen("referenz", x.id, Some(x.projekt_id), &x.ziel, deleted)?;
        Ok(())
    }

    pub fn referenz_speichern(&mut self, x: &Referenz) -> Result<()> {
        let mut x = x.clone();
        if x.geraetegebunden() {
            if x.geraet_id.is_none() {
                x.geraet_id = Some(self.device_id);
            }
        } else {
            // Eine Adresse an ein Gerät zu binden war ein Fehler: auf jedem anderen Gerät
            // wäre sie dann für immer »nicht prüfbar«. Ältere Bestände heilen hier.
            x.geraet_id = None;
        }
        let hlc = self.hlc.next(now_ms());
        self.referenz_row_schreiben(&x, &hlc, false)?;
        self.aenderung(RecordKind::Reference, x.id, &hlc, false)
    }

    /// Eine einzelne Referenz. `None`, wenn es sie nicht (mehr) gibt.
    pub fn referenz(&self, id: Ulid) -> Result<Option<Referenz>> {
        Ok(self
            .conn
            .query_row(
                "SELECT * FROM referenzen WHERE id = ?1 AND deleted = 0",
                params![id.to_string()],
                Self::referenz_aus_row,
            )
            .optional()?)
    }

    pub fn referenzen(&self, projekt_id: Ulid) -> Result<Vec<Referenz>> {
        let mut st = self.conn.prepare(
            "SELECT * FROM referenzen WHERE projekt_id = ?1 AND deleted = 0 ORDER BY typ, ziel",
        )?;
        let rows = st.query_map(params![projekt_id.to_string()], Self::referenz_aus_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Ordner- und Repo-Referenzen, die auf diesem Gerät gelten (für den Beobachter).
    pub fn ordner_referenzen(&self) -> Result<Vec<Referenz>> {
        let mut st = self.conn.prepare(
            "SELECT r.* FROM referenzen r JOIN projekte p ON p.id = r.projekt_id
             WHERE r.deleted = 0 AND p.deleted = 0 AND r.typ IN ('ordner', 'git_repo')
               AND (r.geraet_id IS NULL OR r.geraet_id = ?1)",
        )?;
        let rows = st.query_map(params![self.device_id.to_string()], Self::referenz_aus_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Prüft eine Referenz auf diesem Gerät – **nur Pfade**.
    ///
    /// Adressen bleiben `nicht_pruefbar`: der Speicher hat kein Netz. Wer eine Adresse
    /// wirklich prüfen will, nimmt `referenz_status_setzen` und fragt vorher draußen nach
    /// (die Hülle tut das über `forge`/`netz`). Früher landete eine Adresse hier im
    /// Pfad-Zweig und kam als »nicht erreichbar« zurück.
    pub fn referenz_pruefen(&mut self, id: Ulid) -> Result<Pruefstatus> {
        let mut x = self
            .conn
            .query_row(
                "SELECT * FROM referenzen WHERE id = ?1 AND deleted = 0",
                params![id.to_string()],
                Self::referenz_aus_row,
            )
            .optional()?
            .ok_or_else(|| Error::NotFound(format!("Referenz {id}")))?;
        let status = match x.typ {
            _ if crate::model::ziel_ist_adresse(&x.ziel) => Pruefstatus::NichtPruefbar,
            ReferenzTyp::Ordner | ReferenzTyp::GitRepo | ReferenzTyp::Datei => {
                if x.geraet_id.is_some_and(|g| g != self.device_id) {
                    Pruefstatus::NichtPruefbar
                } else if Path::new(&x.ziel).exists() {
                    Pruefstatus::Ok
                } else {
                    Pruefstatus::NichtErreichbar
                }
            }
            _ => Pruefstatus::NichtPruefbar,
        };
        x.pruefstatus = status;
        x.zuletzt_geprueft = Some(now_ms());
        self.referenz_speichern(&x)?;
        Ok(status)
    }

    /// Schreibt einen Prüfstatus, den jemand anders ermittelt hat – die Hülle, nachdem
    /// sie eine Adresse tatsächlich im Netz gefragt hat.
    pub fn referenz_status_setzen(&mut self, id: Ulid, status: Pruefstatus) -> Result<()> {
        let mut x = self
            .conn
            .query_row(
                "SELECT * FROM referenzen WHERE id = ?1 AND deleted = 0",
                params![id.to_string()],
                Self::referenz_aus_row,
            )
            .optional()?
            .ok_or_else(|| Error::NotFound(format!("Referenz {id}")))?;
        x.pruefstatus = status;
        x.zuletzt_geprueft = Some(now_ms());
        self.referenz_speichern(&x)
    }

    // --------------------------------------------------------------- tresor

    /// Speichert einen (bereits verschlüsselten) Tresor-Eintrag. Der Speicher sieht nur
    /// Titel, Zuordnung und Stufe im Klartext; die Werte sind innerhalb von `json`
    /// versiegelt.
    pub fn tresor_speichern(&mut self, e: &TresorEintrag) -> Result<()> {
        let hlc = self.hlc.next(now_ms());
        self.tresor_row_schreiben(e, &hlc, false)?;
        self.aenderung(RecordKind::VaultEntry, e.id, &hlc, false)
    }

    fn tresor_row_schreiben(&self, e: &TresorEintrag, hlc: &str, deleted: bool) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tresor (id, titel, projekt_ids, stufe, json, angelegt, geaendert, hlc, deleted)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(id) DO UPDATE SET titel=excluded.titel, projekt_ids=excluded.projekt_ids,
                stufe=excluded.stufe, json=excluded.json, angelegt=excluded.angelegt,
                geaendert=excluded.geaendert, hlc=excluded.hlc, deleted=excluded.deleted",
            params![
                e.id.to_string(),
                e.titel,
                serde_json::to_string(&e.projekt_ids)?,
                e.stufe.as_str(),
                serde_json::to_string(e)?,
                e.angelegt,
                e.geaendert,
                hlc,
                deleted as i64
            ],
        )?;
        self.suche_ersetzen("tresor", e.id, None, &e.titel, deleted)?;
        Ok(())
    }

    fn tresor_aus_row(r: &Row) -> rusqlite::Result<TresorEintrag> {
        let json: String = r.get("json")?;
        serde_json::from_str(&json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    }

    pub fn tresor_eintrag(&self, id: Ulid) -> Result<Option<TresorEintrag>> {
        Ok(self
            .conn
            .query_row(
                "SELECT json FROM tresor WHERE id = ?1 AND deleted = 0",
                params![id.to_string()],
                Self::tresor_aus_row,
            )
            .optional()?)
    }

    /// Alle Einträge, optional nur die eines Projekts.
    pub fn tresor_eintraege(&self, projekt: Option<Ulid>) -> Result<Vec<TresorEintrag>> {
        let mut st = self
            .conn
            .prepare("SELECT json, projekt_ids FROM tresor WHERE deleted = 0 ORDER BY titel")?;
        let rows = st.query_map([], Self::tresor_aus_row)?;
        let alle: Vec<TresorEintrag> = rows.collect::<rusqlite::Result<_>>()?;
        Ok(match projekt {
            None => alle,
            Some(p) => alle
                .into_iter()
                .filter(|e| e.projekt_ids.contains(&p))
                .collect(),
        })
    }

    /// Löscht einen Eintrag: Crypto-Shredding plus Tombstone.
    pub fn tresor_loeschen(&mut self, id: Ulid) -> Result<()> {
        let mut e = self
            .tresor_eintrag(id)?
            .ok_or_else(|| Error::NotFound(format!("Tresor-Eintrag {id}")))?;
        e.schreddern(now_ms());
        let hlc = self.hlc.next(now_ms());
        self.tresor_row_schreiben(&e, &hlc, true)?;
        self.aenderung(RecordKind::VaultEntry, id, &hlc, true)
    }

    // -------------------------------------------------------------- geraete

    pub fn geraet_speichern(&mut self, g: &Geraet) -> Result<()> {
        let hlc = self.hlc.next(now_ms());
        self.conn.execute(
            "INSERT INTO geraete (id, json, hlc, deleted) VALUES (?1, ?2, ?3, 0)
             ON CONFLICT(id) DO UPDATE SET json=excluded.json, hlc=excluded.hlc, deleted=0",
            params![g.id.to_string(), serde_json::to_string(g)?, hlc],
        )?;
        self.aenderung(RecordKind::Device, g.id, &hlc, false)
    }

    pub fn geraete(&self) -> Result<Vec<Geraet>> {
        let mut st = self
            .conn
            .prepare("SELECT json FROM geraete WHERE deleted = 0")?;
        let rows = st.query_map([], |r| {
            let json: String = r.get(0)?;
            serde_json::from_str(&json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    // ----------------------------------------------------------- kandidaten

    /// Merkt Kandidaten aus der Projekterkennung (Hafeneinfahrt). Verworfene bleiben
    /// verworfen, bis der Ordner verschwindet.
    pub fn kandidaten_merken(&mut self, ks: &[Kandidat]) -> Result<()> {
        let jetzt = now_ms();
        let tx = self.conn.transaction()?;
        for k in ks {
            tx.execute(
                "INSERT INTO kandidaten (pfad, json, gesehen, verworfen) VALUES (?1, ?2, ?3, 0)
                 ON CONFLICT(pfad) DO UPDATE SET json=excluded.json, gesehen=excluded.gesehen",
                params![k.pfad, serde_json::to_string(k)?, jetzt],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn kandidaten(&self) -> Result<Vec<Kandidat>> {
        let mut st = self
            .conn
            .prepare("SELECT json FROM kandidaten WHERE verworfen = 0 ORDER BY pfad")?;
        let rows = st.query_map([], |r| {
            let json: String = r.get(0)?;
            serde_json::from_str(&json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    pub fn kandidat_verwerfen(&self, pfad: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE kandidaten SET verworfen = 1 WHERE pfad = ?1",
            params![pfad],
        )?;
        Ok(())
    }

    pub fn kandidat_entfernen(&self, pfad: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM kandidaten WHERE pfad = ?1", params![pfad])?;
        Ok(())
    }

    // ---------------------------------------------------------------- hafen

    /// Alle Projekte als Karten mit Auffälligkeit, für die Startseite.
    pub fn hafen(&self, jetzt_ms: i64) -> Result<Vec<HafenKarte>> {
        let mut karten = Vec::new();
        let offene: std::collections::HashMap<Ulid, usize> = {
            let mut m = std::collections::HashMap::new();
            for n in self.offene_faeden(None)? {
                *m.entry(n.projekt_id).or_insert(0) += 1;
            }
            m
        };
        for p in self.projekte()? {
            let letzte = self.letzte_notiz(p.id)?;
            let letzte_ts = letzte.as_ref().map(|n| n.ts);
            karten.push(HafenKarte {
                auffaelligkeit: brief::auffaelligkeit(&p, letzte_ts, jetzt_ms),
                tage_seit: brief::tage_seit(&p, letzte_ts, jetzt_ms),
                offene_faeden: offene.get(&p.id).copied().unwrap_or(0),
                letzte_notiz: letzte,
                projekt: p,
            });
        }
        karten.sort_by(|a, b| {
            let rang = |k: &HafenKarte| match k.auffaelligkeit {
                Auffaelligkeit::Ueberfaellig => 0,
                Auffaelligkeit::Auffaellig => 1,
                Auffaelligkeit::Ruhig => 2,
            };
            rang(a)
                .cmp(&rang(b))
                .then_with(|| b.projekt.zuletzt_beruehrt.cmp(&a.projekt.zuletzt_beruehrt))
        });
        Ok(karten)
    }

    /// Der Brief für ein Projekt.
    pub fn brief(&self, projekt_id: Ulid, jetzt_ms: i64) -> Result<brief::Brief> {
        let p = self
            .projekt(projekt_id)?
            .ok_or_else(|| Error::NotFound(format!("Projekt {projekt_id}")))?;
        let notizen = self.notizen(projekt_id)?;
        Ok(brief::brief(&p, &notizen, jetzt_ms))
    }

    // ---------------------------------------------------------------- suche

    fn suche_ersetzen(
        &self,
        kind: &str,
        ref_id: Ulid,
        projekt_id: Option<Ulid>,
        text: &str,
        deleted: bool,
    ) -> Result<()> {
        self.conn.execute(
            "DELETE FROM suche WHERE kind = ?1 AND ref_id = ?2",
            params![kind, ref_id.to_string()],
        )?;
        if !deleted && !text.trim().is_empty() {
            self.conn.execute(
                "INSERT INTO suche (text, kind, ref_id, projekt_id) VALUES (?1, ?2, ?3, ?4)",
                params![
                    text,
                    kind,
                    ref_id.to_string(),
                    projekt_id.map(|u| u.to_string())
                ],
            )?;
        }
        Ok(())
    }

    /// Volltextsuche über Projekte, Notizen, Referenzen und Tresor-Titel.
    pub fn suche(&self, anfrage: &str) -> Result<Vec<Treffer>> {
        let tokens: Vec<String> = anfrage
            .split_whitespace()
            .map(|t| format!("\"{}\"*", t.replace('"', "")))
            .collect();
        if tokens.is_empty() {
            return Ok(Vec::new());
        }
        let match_expr = tokens.join(" ");
        let mut st = self.conn.prepare(
            "SELECT kind, ref_id, projekt_id, snippet(suche, 0, '[', ']', '…', 12)
             FROM suche WHERE suche MATCH ?1 ORDER BY rank LIMIT 50",
        )?;
        let rows = st.query_map(params![match_expr], |r| {
            Ok(Treffer {
                kind: r.get(0)?,
                ref_id: ulid_col_idx(r, 1)?,
                projekt_id: opt_ulid_col_idx(r, 2)?,
                ausschnitt: r.get(3)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    // ----------------------------------------------------------------- sync

    fn aenderung(&self, kind: RecordKind, id: Ulid, hlc: &str, deleted: bool) -> Result<()> {
        self.conn.execute(
            "INSERT INTO aenderungen (record_id, kind, hlc, deleted) VALUES (?1, ?2, ?3, ?4)",
            params![id.to_string(), kind.as_str(), hlc, deleted as i64],
        )?;
        Ok(())
    }

    pub fn sync_state(&self) -> Result<SyncState> {
        Ok(match self.meta_get("sync_state")? {
            Some(s) => serde_json::from_str(&s)?,
            None => SyncState::default(),
        })
    }

    pub fn sync_state_setzen(&self, s: &SyncState) -> Result<()> {
        self.meta_set("sync_state", &serde_json::to_string(s)?)
    }

    /// Klartext-Datensatz für einen Umschlag, oder `None`, wenn gelöscht.
    fn payload(&self, kind: RecordKind, id: Ulid) -> Result<Option<serde_json::Value>> {
        let ids = id.to_string();
        Ok(match kind {
            RecordKind::Project => self
                .conn
                .query_row(
                    "SELECT * FROM projekte WHERE id = ?1 AND deleted = 0",
                    params![ids],
                    Self::projekt_aus_row,
                )
                .optional()?
                .map(serde_json::to_value)
                .transpose()?,
            RecordKind::Note => self
                .conn
                .query_row(
                    "SELECT * FROM notizen WHERE id = ?1 AND deleted = 0",
                    params![ids],
                    Self::notiz_aus_row,
                )
                .optional()?
                .map(serde_json::to_value)
                .transpose()?,
            RecordKind::Reference => self
                .conn
                .query_row(
                    "SELECT * FROM referenzen WHERE id = ?1 AND deleted = 0",
                    params![ids],
                    Self::referenz_aus_row,
                )
                .optional()?
                .map(serde_json::to_value)
                .transpose()?,
            RecordKind::VaultEntry => self
                .conn
                .query_row(
                    "SELECT json FROM tresor WHERE id = ?1 AND deleted = 0",
                    params![ids],
                    |r| r.get::<_, String>(0),
                )
                .optional()?
                .map(|j| serde_json::from_str(&j))
                .transpose()?,
            RecordKind::Device => self
                .conn
                .query_row(
                    "SELECT json FROM geraete WHERE id = ?1 AND deleted = 0",
                    params![ids],
                    |r| r.get::<_, String>(0),
                )
                .optional()?
                .map(|j| serde_json::from_str(&j))
                .transpose()?,
            RecordKind::Settings => None,
        })
    }

    /// Baut Umschläge für alle noch nicht gepushten Änderungen. Pro Datensatz nur der
    /// jüngste Stand; ältere Einträge im Protokoll werden übersprungen.
    pub fn ausstehende_umschlaege(&self, limit: usize) -> Result<(Vec<Umschlag>, u64)> {
        let state = self.sync_state()?;
        let mut st = self.conn.prepare(
            "SELECT local_seq, record_id, kind, hlc, deleted FROM aenderungen
             WHERE local_seq > ?1 ORDER BY local_seq ASC LIMIT ?2",
        )?;
        let rows = st.query_map(
            params![state.last_pushed_local_seq as i64, limit as i64],
            |r| {
                Ok((
                    r.get::<_, i64>(0)? as u64,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, i64>(4)? != 0,
                ))
            },
        )?;
        let mut neueste: std::collections::BTreeMap<String, (u64, RecordKind, String, bool)> =
            std::collections::BTreeMap::new();
        let mut max_seq = state.last_pushed_local_seq;
        for row in rows {
            let (seq, id, kind, hlc, deleted) = row?;
            max_seq = max_seq.max(seq);
            if let Some(k) = RecordKind::parse(&kind) {
                neueste.insert(id, (seq, k, hlc, deleted));
            }
        }
        let mut out = Vec::with_capacity(neueste.len());
        for (id, (_, kind, hlc, deleted)) in neueste {
            let id = Ulid::from_string(&id)
                .map_err(|_| Error::Invalid("ULID im Änderungsprotokoll".into()))?;
            let umschlag = if deleted {
                Umschlag::tombstone(kind, id, hlc, self.device_id)
            } else {
                match self.payload(kind, id)? {
                    Some(v) => {
                        Umschlag::seal(&self.records_key, kind, id, hlc, self.device_id, &v)?
                    }
                    // Inzwischen gelöscht: Tombstone mit dem aktuellen HLC schicken.
                    None => Umschlag::tombstone(kind, id, hlc, self.device_id),
                }
            };
            out.push(umschlag);
        }
        Ok((out, max_seq))
    }

    /// Nach erfolgreichem Push: Stand merken.
    pub fn gepusht_bis(&self, local_seq: u64) -> Result<()> {
        let mut s = self.sync_state()?;
        s.last_pushed_local_seq = s.last_pushed_local_seq.max(local_seq);
        s.last_sync_ms = Some(now_ms());
        self.sync_state_setzen(&s)
    }

    /// Wendet einen fremden Umschlag an (Pull). Schreibt **nicht** ins Änderungsprotokoll,
    /// außer bei einer Konfliktnotiz.
    pub fn umschlag_anwenden(&mut self, u: &Umschlag) -> Result<Angewendet> {
        u.validate()?;
        self.hlc.observe(&u.hlc)?;
        let tabelle = match u.kind {
            RecordKind::Project => "projekte",
            RecordKind::Note => "notizen",
            RecordKind::Reference => "referenzen",
            RecordKind::VaultEntry => "tresor",
            RecordKind::Device => "geraete",
            RecordKind::Settings => return Ok(Angewendet::Verworfen),
        };
        let vorhanden: Option<String> = self
            .conn
            .query_row(
                &format!("SELECT hlc FROM {tabelle} WHERE id = ?1"),
                params![u.id.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        if !sync::gewinnt(&u.hlc, vorhanden.as_deref()) {
            return Ok(Angewendet::Verworfen);
        }

        if u.deleted {
            match u.kind {
                RecordKind::Project => {
                    if let Some(p) = self.projekt(u.id)? {
                        self.projekt_row_schreiben(&p, &u.hlc, true)?;
                    } else {
                        self.conn.execute(
                            "UPDATE projekte SET deleted = 1, hlc = ?2 WHERE id = ?1",
                            params![u.id.to_string(), u.hlc],
                        )?;
                    }
                }
                _ => {
                    self.conn.execute(
                        &format!("UPDATE {tabelle} SET deleted = 1, hlc = ?2 WHERE id = ?1"),
                        params![u.id.to_string(), u.hlc],
                    )?;
                    let kind = match u.kind {
                        RecordKind::Note => "notiz",
                        RecordKind::Reference => "referenz",
                        RecordKind::VaultEntry => "tresor",
                        _ => "",
                    };
                    if !kind.is_empty() {
                        self.suche_ersetzen(kind, u.id, None, "", true)?;
                    }
                }
            }
            return Ok(Angewendet::Uebernommen);
        }

        match u.kind {
            RecordKind::Project => {
                let neu: Projekt = u.open(&self.records_key)?;
                let alt = self.projekt(u.id)?;
                let konflikt = alt.as_ref().is_some_and(|a| {
                    a.titel != neu.titel || a.kurs != neu.kurs || a.status != neu.status
                });
                self.projekt_row_schreiben(&neu, &u.hlc, false)?;
                if let (true, Some(a)) = (konflikt, alt) {
                    let text = format!(
                        "Konflikt beim Abgleich: dieses Gerät hatte Titel »{}«, Status »{}«, Kurs »{}«. \
                         Übernommen wurde die neuere Fassung von Gerät {}.",
                        a.titel,
                        a.status.as_str(),
                        a.kurs,
                        &u.device_id.to_string()[20..]
                    );
                    self.notiz_speichern(&Notiz::neu(
                        u.id,
                        Quelle::Sync,
                        Art::Log,
                        text,
                        now_ms(),
                    ))?;
                    return Ok(Angewendet::Konflikt);
                }
            }
            RecordKind::Note => {
                let n: Notiz = u.open(&self.records_key)?;
                self.notiz_row_schreiben(&n, &u.hlc, false)?;
            }
            RecordKind::Reference => {
                let x: Referenz = u.open(&self.records_key)?;
                self.referenz_row_schreiben(&x, &u.hlc, false)?;
            }
            RecordKind::VaultEntry => {
                let e: TresorEintrag = u.open(&self.records_key)?;
                self.tresor_row_schreiben(&e, &u.hlc, false)?;
            }
            RecordKind::Device => {
                let g: Geraet = u.open(&self.records_key)?;
                self.conn.execute(
                    "INSERT INTO geraete (id, json, hlc, deleted) VALUES (?1, ?2, ?3, 0)
                     ON CONFLICT(id) DO UPDATE SET json=excluded.json, hlc=excluded.hlc, deleted=0",
                    params![g.id.to_string(), serde_json::to_string(&g)?, u.hlc],
                )?;
            }
            RecordKind::Settings => {}
        }
        Ok(Angewendet::Uebernommen)
    }

    /// Nach einem Pull: Server-Sequenz merken.
    pub fn gepullt_bis(&self, server_seq: u64) -> Result<()> {
        let mut s = self.sync_state()?;
        s.last_server_seq = s.last_server_seq.max(server_seq);
        s.last_sync_ms = Some(now_ms());
        self.sync_state_setzen(&s)
    }

    /// Anzahl noch nicht gepushter Änderungen.
    pub fn ausstehend(&self) -> Result<u64> {
        let s = self.sync_state()?;
        Ok(self.conn.query_row(
            "SELECT count(*) FROM aenderungen WHERE local_seq > ?1",
            params![s.last_pushed_local_seq as i64],
            |r| r.get::<_, i64>(0),
        )? as u64)
    }
}

fn ulid_col(r: &Row, name: &str) -> rusqlite::Result<Ulid> {
    let s: String = r.get(name)?;
    Ulid::from_string(&s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn opt_ulid_col(r: &Row, name: &str) -> rusqlite::Result<Option<Ulid>> {
    let s: Option<String> = r.get(name)?;
    Ok(s.and_then(|s| Ulid::from_string(&s).ok()))
}

fn ulid_col_idx(r: &Row, idx: usize) -> rusqlite::Result<Ulid> {
    let s: String = r.get(idx)?;
    Ulid::from_string(&s).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(idx, rusqlite::types::Type::Text, Box::new(e))
    })
}

fn opt_ulid_col_idx(r: &Row, idx: usize) -> rusqlite::Result<Option<Ulid>> {
    let s: Option<String> = r.get(idx)?;
    Ok(s.and_then(|s| Ulid::from_string(&s).ok()))
}

#[cfg(test)]
mod tests {
    /// Eine Adresse ist kein Dateipfad.
    ///
    /// `git_repo` steht für beides: einen Ordner mit `.git` und eine Adresse. Früher lief
    /// die Adresse in den Pfad-Zweig, `Path::exists()` sagte nein, und die Oberfläche
    /// meldete »nicht erreichbar« für ein Repo, das es gibt.
    #[test]
    fn adresse_wird_nicht_im_dateisystem_gesucht() {
        use crate::model::{Projekt, Referenz, ReferenzTyp, Rolle, Vorlage};
        let ak = crate::crypto::Key32::random().unwrap();
        let mut s = Store::open_in_memory(&ak, Ulid::new()).unwrap();
        let p = Projekt::neu("Lotse", Vorlage::Software, now_ms());
        s.projekt_speichern(&p).unwrap();

        let adresse = Referenz::neu(
            p.id,
            ReferenzTyp::GitRepo,
            "https://github.com/phish3144/lotse",
            Rolle::Material,
        );
        s.referenz_speichern(&adresse).unwrap();
        assert_eq!(
            s.referenz_pruefen(adresse.id).unwrap(),
            Pruefstatus::NichtPruefbar,
            "eine Adresse ist im Speicher nicht prüfbar – aber ganz sicher nicht »nicht erreichbar«"
        );

        // Und sie hängt an keinem Gerät: sonst wäre sie auf dem zweiten Rechner blind.
        assert_eq!(s.referenz(adresse.id).unwrap().unwrap().geraet_id, None);

        // Die Hülle trägt das Ergebnis von draußen nach.
        s.referenz_status_setzen(adresse.id, Pruefstatus::Ok)
            .unwrap();
        let danach = s.referenz(adresse.id).unwrap().unwrap();
        assert_eq!(danach.pruefstatus, Pruefstatus::Ok);
        assert!(danach.zuletzt_geprueft.is_some());

        // Ein echter Ordner bleibt ein Pfad und wird ans Gerät gebunden.
        let t = tempfile::tempdir().unwrap();
        let ordner = Referenz::neu(
            p.id,
            ReferenzTyp::GitRepo,
            t.path().to_string_lossy().to_string(),
            Rolle::Material,
        );
        s.referenz_speichern(&ordner).unwrap();
        assert_eq!(s.referenz_pruefen(ordner.id).unwrap(), Pruefstatus::Ok);
        assert!(s.referenz(ordner.id).unwrap().unwrap().geraet_id.is_some());

        let weg = Referenz::neu(p.id, ReferenzTyp::Ordner, "/gibt/es/nicht", Rolle::Material);
        s.referenz_speichern(&weg).unwrap();
        assert_eq!(
            s.referenz_pruefen(weg.id).unwrap(),
            Pruefstatus::NichtErreichbar
        );
    }

    use super::*;
    use crate::vault::VaultKeys;

    fn store() -> (Store, Key32) {
        let ak = Key32::random().unwrap();
        (Store::open_in_memory(&ak, Ulid::new()).unwrap(), ak)
    }

    #[test]
    fn projekt_und_notizen() {
        let (mut s, _) = store();
        let p = Projekt::neu("Gartenhaus", Vorlage::HausGarten, now_ms());
        s.projekt_speichern(&p).unwrap();
        s.notiz_speichern(&Notiz::neu(
            p.id,
            Quelle::Cli,
            Art::Log,
            "Beton bestellt",
            now_ms(),
        ))
        .unwrap();
        s.notiz_speichern(&Notiz::neu(
            p.id,
            Quelle::Mensch,
            Art::Offen,
            "Bewehrung nötig?",
            now_ms() + 1,
        ))
        .unwrap();
        assert_eq!(s.notizen(p.id).unwrap().len(), 2);
        assert_eq!(s.offene_faeden(None).unwrap().len(), 1);
        let hafen = s.hafen(now_ms()).unwrap();
        assert_eq!(hafen.len(), 1);
        assert_eq!(hafen[0].offene_faeden, 1);
        assert_eq!(
            hafen[0].letzte_notiz.as_ref().unwrap().text,
            "Bewehrung nötig?"
        );
        let treffer = s.suche("beton").unwrap();
        assert_eq!(treffer.len(), 1);
        assert_eq!(treffer[0].kind, "notiz");
        assert!(s
            .suche("garten")
            .unwrap()
            .iter()
            .any(|t| t.kind == "projekt"));
    }

    #[test]
    fn falscher_schluessel_wird_erkannt() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("lotse.db");
        let ak = Key32::random().unwrap();
        {
            let mut s = Store::open(&path, &ak, Ulid::new()).unwrap();
            s.projekt_speichern(&Projekt::neu("x", Vorlage::Generisch, 1))
                .unwrap();
        }
        let falsch = Key32::random().unwrap();
        assert!(Store::open(&path, &falsch, Ulid::new()).is_err());
        let s = Store::open(&path, &ak, Ulid::new()).unwrap();
        assert_eq!(s.projekte().unwrap().len(), 1);
        // Rohdatei enthält den Projekttitel nicht im Klartext.
        let raw = std::fs::read(&path).unwrap();
        assert!(
            !raw.windows(1).any(|w| w == b"x")
                || !String::from_utf8_lossy(&raw).contains("Generisch")
        );
    }

    #[test]
    fn status_verlangt_uebergabe() {
        let (mut s, _) = store();
        let p = Projekt::neu("P", Vorlage::Software, now_ms());
        s.projekt_speichern(&p).unwrap();
        assert!(s
            .status_setzen(p.id, Status::Pausiert, None, None, Quelle::Mensch)
            .is_err());
        s.status_setzen(
            p.id,
            Status::Pausiert,
            Some("Warte auf Teile. Nächster Schritt: löten."),
            Some("2030-01-01"),
            Quelle::Mensch,
        )
        .unwrap();
        let p2 = s.projekt(p.id).unwrap().unwrap();
        assert_eq!(p2.status, Status::Pausiert);
        assert_eq!(p2.wiedervorlage.as_deref(), Some("2030-01-01"));
        let arten: Vec<Art> = s.notizen(p.id).unwrap().iter().map(|n| n.art).collect();
        assert_eq!(arten, vec![Art::Status, Art::Uebergabe]);
        let b = s.brief(p.id, now_ms()).unwrap();
        assert!(b.letzte_uebergabe.unwrap().contains("löten"));
    }

    #[test]
    fn sync_roundtrip_zwischen_zwei_geraeten() {
        let ak = Key32::random().unwrap();
        let mut a = Store::open_in_memory(&ak, Ulid::new()).unwrap();
        let mut b = Store::open_in_memory(&ak, Ulid::new()).unwrap();

        let p = Projekt::neu("Synced", Vorlage::Software, now_ms());
        a.projekt_speichern(&p).unwrap();
        a.notiz_speichern(&Notiz::neu(p.id, Quelle::Cli, Art::Log, "hallo", now_ms()))
            .unwrap();
        let vk = VaultKeys::from_account_key(&ak);
        let e = TresorEintrag::neu(
            &vk,
            "Router",
            vec![p.id],
            Stufe::Ueberall,
            &[("pw", "geheim")],
            now_ms(),
        )
        .unwrap();
        a.tresor_speichern(&e).unwrap();

        let (umschlaege, seq) = a.ausstehende_umschlaege(500).unwrap();
        assert_eq!(umschlaege.len(), 3);
        a.gepusht_bis(seq).unwrap();
        assert_eq!(a.ausstehend().unwrap(), 0);

        for u in &umschlaege {
            let json = serde_json::to_string(u).unwrap();
            assert!(
                !json.contains("Synced") && !json.contains("hallo") && !json.contains("Router")
            );
            let u2: Umschlag = serde_json::from_str(&json).unwrap();
            assert_eq!(b.umschlag_anwenden(&u2).unwrap(), Angewendet::Uebernommen);
        }
        assert_eq!(b.projekte().unwrap()[0].titel, "Synced");
        assert_eq!(b.notizen(p.id).unwrap()[0].text, "hallo");
        let eb = b.tresor_eintrag(e.id).unwrap().unwrap();
        assert_eq!(eb.feld_lesen(&vk, "pw").unwrap().as_str(), "geheim");
        // Übernommene Umschläge erzeugen keine neuen ausstehenden Änderungen.
        assert_eq!(b.ausstehend().unwrap(), 0);

        // Älterer Umschlag verliert.
        let mut alt = umschlaege[0].clone();
        alt.hlc = crate::hlc::format(1, 0, &a.device_id());
        assert_eq!(b.umschlag_anwenden(&alt).unwrap(), Angewendet::Verworfen);

        // Konflikt: B ändert den Titel, dann kommt A's neuere Fassung.
        let mut pb = b.projekt(p.id).unwrap().unwrap();
        pb.titel = "Anders".into();
        b.projekt_speichern(&pb).unwrap();
        let mut pa = a.projekt(p.id).unwrap().unwrap();
        pa.titel = "Neuer".into();
        std::thread::sleep(std::time::Duration::from_millis(2));
        a.projekt_speichern(&pa).unwrap();
        let (ua, _) = a.ausstehende_umschlaege(500).unwrap();
        assert_eq!(b.umschlag_anwenden(&ua[0]).unwrap(), Angewendet::Konflikt);
        assert_eq!(b.projekt(p.id).unwrap().unwrap().titel, "Neuer");
        assert!(b
            .notizen(p.id)
            .unwrap()
            .iter()
            .any(|n| n.quelle == Quelle::Sync));
    }

    #[test]
    fn tombstone_loescht() {
        let ak = Key32::random().unwrap();
        let mut a = Store::open_in_memory(&ak, Ulid::new()).unwrap();
        let mut b = Store::open_in_memory(&ak, Ulid::new()).unwrap();
        let p = Projekt::neu("Weg", Vorlage::Generisch, now_ms());
        a.projekt_speichern(&p).unwrap();
        let (u, seq) = a.ausstehende_umschlaege(500).unwrap();
        a.gepusht_bis(seq).unwrap();
        b.umschlag_anwenden(&u[0]).unwrap();
        a.projekt_loeschen(p.id).unwrap();
        let (u, _) = a.ausstehende_umschlaege(500).unwrap();
        assert!(u[0].deleted);
        b.umschlag_anwenden(&u[0]).unwrap();
        assert!(b.projekt(p.id).unwrap().is_none());
        assert!(b.suche("weg").unwrap().is_empty());
    }

    #[test]
    fn postkorb_wird_einmal_angelegt() {
        let (mut s, _) = store();
        let a = s.postkorb().unwrap();
        let b = s.postkorb().unwrap();
        assert_eq!(a.id, b.id);
        assert_eq!(s.projekte().unwrap().len(), 1);
    }
}
