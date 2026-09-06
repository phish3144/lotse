//! Export: Klartext-Spiegel (Markdown) und verschlüsseltes `age`-Bundle.
//!
//! * `spiegel` schreibt pro Projekt `projekt.md` und `logbuch/JJJJ-MM.md`. Kein Tresor,
//!   kein Import aus `vault` (Bedrohungsmodell, Abschnitt 6).
//! * `bundle` ist der Fluchtweg: der gesamte Bestand als JSON, mit `age` und Passphrase
//!   verschlüsselt, mit dem generischen `age`-CLI ohne Lotse entschlüsselbar. Nur hier
//!   werden Tresor-Werte entschlüsselt, und nur mit ausdrücklich übergebenen Schlüsseln.

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::model::{slug, Notiz, Projekt, Referenz};
use crate::store::Store;
use crate::{Error, Result, SCHEMA_VERSION};

pub mod spiegel {
    use super::*;

    /// Schreibt den Spiegel in `dir`. Liefert die Zahl geschriebener Projekte.
    pub fn schreiben(store: &Store, dir: &Path) -> Result<usize> {
        fs::create_dir_all(dir)?;
        let mut n = 0;
        for p in store.projekte()? {
            let notizen = store.notizen(p.id)?;
            let referenzen = store.referenzen(p.id)?;
            let pdir = dir.join(slug(&p.titel));
            fs::create_dir_all(pdir.join("logbuch"))?;
            atomar_schreiben(
                &pdir.join("projekt.md"),
                projekt_md(&p, &referenzen).as_bytes(),
            )?;
            for (monat, text) in logbuch_md(&notizen) {
                atomar_schreiben(
                    &pdir.join("logbuch").join(format!("{monat}.md")),
                    text.as_bytes(),
                )?;
            }
            n += 1;
        }
        atomar_schreiben(
            &dir.join("README.md"),
            b"# Lotse-Spiegel\n\nAbgeleitete Klartext-Kopie. Wahrheit ist die Lotse-Datenbank; \
              Aenderungen hier fliessen nicht zurueck. Kein Tresor-Inhalt.\n",
        )?;
        Ok(n)
    }

    pub fn projekt_md(p: &Projekt, referenzen: &[Referenz]) -> String {
        let mut s = String::new();
        s.push_str("---\n");
        s.push_str(&format!("lotse_schema: {SCHEMA_VERSION}\n"));
        s.push_str(&format!("id: {}\n", p.id));
        s.push_str(&format!("titel: {}\n", yaml_str(&p.titel)));
        s.push_str(&format!("status: {}\n", p.status.as_str()));
        s.push_str(&format!("vorlage: {}\n", p.vorlage.as_str()));
        if let Some(w) = &p.wiedervorlage {
            s.push_str(&format!("wiedervorlage: {w}\n"));
        }
        s.push_str(&format!(
            "erwartungsintervall_tage: {}\n",
            p.erwartungsintervall_tage
        ));
        s.push_str(&format!(
            "tags: [{}]\n",
            p.tags
                .iter()
                .map(|t| yaml_str(t))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        if let Some(a) = p.abgeleitet_von {
            s.push_str(&format!("abgeleitet_von: {a}\n"));
        }
        s.push_str(&format!("angelegt: {}\n", iso(p.angelegt)));
        s.push_str(&format!("zuletzt_beruehrt: {}\n", iso(p.zuletzt_beruehrt)));
        if !referenzen.is_empty() {
            s.push_str("referenzen:\n");
            for r in referenzen {
                s.push_str(&format!(
                    "  - {{typ: {}, ziel: {}, rolle: {}, pruefstatus: {}}}\n",
                    r.typ.as_str(),
                    yaml_str(&r.ziel),
                    r.rolle.as_str(),
                    r.pruefstatus.as_str()
                ));
            }
        }
        s.push_str("---\n\n");
        s.push_str(&format!("# {}\n\n", p.titel));
        if !p.kurs.is_empty() {
            s.push_str("## Kurs\n\n");
            s.push_str(&p.kurs);
            s.push('\n');
        }
        s
    }

    /// Gruppiert Notizen nach Monat; liefert (`JJJJ-MM`, Markdown).
    pub fn logbuch_md(notizen: &[Notiz]) -> Vec<(String, String)> {
        let mut monate: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        let mut sortiert: Vec<&Notiz> = notizen.iter().collect();
        sortiert.sort_by_key(|n| n.ts);
        for n in sortiert {
            let dt = datum(n.ts);
            let monat = format!("{:04}-{:02}", dt.year(), u8::from(dt.month()));
            let eintrag = monate
                .entry(monat)
                .or_insert_with(|| "# Logbuch\n\n".to_string());
            let status = if n.ist_offen() {
                " · offen"
            } else if n.erledigt_am.is_some() {
                " · erledigt"
            } else {
                ""
            };
            eintrag.push_str(&format!(
                "## {} · {} · {}{}\n\n{}\n\n",
                iso(n.ts),
                n.art.as_str(),
                n.quelle.as_str(),
                status,
                n.text.trim()
            ));
        }
        monate.into_iter().collect()
    }

    fn yaml_str(s: &str) -> String {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

/// Der Fluchtweg: alles als JSON.
#[derive(Debug, Serialize, Deserialize)]
pub struct Bundle {
    pub lotse_schema: u32,
    pub exportiert: String,
    pub projekte: Vec<Projekt>,
    pub notizen: Vec<Notiz>,
    pub referenzen: Vec<Referenz>,
    pub tresor: Vec<TresorKlartext>,
    /// Einträge, die mit den vorhandenen Schlüsseln nicht lesbar waren (Stufe `nur_desktop`
    /// ohne Desktop-Schlüssel, oder geschreddert).
    pub tresor_nicht_lesbar: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TresorKlartext {
    pub id: String,
    pub titel: String,
    pub projekt_ids: Vec<String>,
    pub stufe: String,
    pub felder: Vec<(String, String)>,
}

pub mod bundle {
    use super::*;
    use crate::vault::VaultKeys;
    use secrecy::SecretString;

    /// Baut das Klartext-Bundle. `keys` bestimmt, welche Tresor-Einträge entschlüsselt
    /// werden können.
    pub fn bauen(store: &Store, keys: Option<&VaultKeys>) -> Result<Bundle> {
        let mut projekte = Vec::new();
        let mut notizen = Vec::new();
        let mut referenzen = Vec::new();
        for p in store.projekte()? {
            notizen.extend(store.notizen(p.id)?);
            referenzen.extend(store.referenzen(p.id)?);
            projekte.push(p);
        }
        let mut tresor = Vec::new();
        let mut nicht_lesbar = Vec::new();
        for e in store.tresor_eintraege(None)? {
            match keys.filter(|k| e.lesbar_mit(k)) {
                Some(k) => {
                    let mut felder = Vec::new();
                    for f in &e.felder {
                        match e.feld_lesen(k, &f.name) {
                            Ok(v) => felder.push((f.name.clone(), v.to_string())),
                            Err(_) => {
                                nicht_lesbar.push(format!("{} / {}", e.titel, f.name));
                            }
                        }
                    }
                    tresor.push(TresorKlartext {
                        id: e.id.to_string(),
                        titel: e.titel.clone(),
                        projekt_ids: e.projekt_ids.iter().map(|u| u.to_string()).collect(),
                        stufe: e.stufe.as_str().to_string(),
                        felder,
                    });
                }
                None => nicht_lesbar.push(e.titel.clone()),
            }
        }
        Ok(Bundle {
            lotse_schema: SCHEMA_VERSION,
            exportiert: iso(crate::now_ms()),
            projekte,
            notizen,
            referenzen,
            tresor,
            tresor_nicht_lesbar: nicht_lesbar,
        })
    }

    /// Verschlüsselt Bytes mit `age` und Passphrase (scrypt). Entschlüsselbar per
    /// `age -d -o bundle.json bundle.json.age`.
    pub fn age_verschluesseln(klartext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
        let enc = age::Encryptor::with_user_passphrase(SecretString::from(passphrase.to_string()));
        let mut out = Vec::new();
        let mut w = enc
            .wrap_output(&mut out)
            .map_err(|e| Error::Other(format!("age: {e}")))?;
        w.write_all(klartext)?;
        w.finish().map_err(|e| Error::Other(format!("age: {e}")))?;
        Ok(out)
    }

    pub fn age_entschluesseln(ciphertext: &[u8], passphrase: &str) -> Result<Vec<u8>> {
        let dec = age::Decryptor::new(ciphertext).map_err(|e| Error::Other(format!("age: {e}")))?;
        let identity = age::scrypt::Identity::new(SecretString::from(passphrase.to_string()));
        let mut r = dec
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .map_err(|_| Error::Decrypt)?;
        let mut out = Vec::new();
        r.read_to_end(&mut out)?;
        Ok(out)
    }

    /// Schreibt `bundle.json.age` nach `ziel`.
    pub fn schreiben(
        store: &Store,
        keys: Option<&VaultKeys>,
        passphrase: &str,
        ziel: &Path,
    ) -> Result<Bundle> {
        let b = bauen(store, keys)?;
        let json = serde_json::to_vec_pretty(&b)?;
        let enc = age_verschluesseln(&json, passphrase)?;
        atomar_schreiben(ziel, &enc)?;
        Ok(b)
    }
}

/// Schreibt Temp + Rename, damit Sync-Werkzeuge nie halbe Dateien sehen.
pub fn atomar_schreiben(ziel: &Path, inhalt: &[u8]) -> Result<()> {
    let tmp = ziel.with_extension("tmp-lotse");
    fs::write(&tmp, inhalt)?;
    fs::rename(&tmp, ziel)?;
    Ok(())
}

fn datum(ms: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(ms.div_euclid(1000)).unwrap_or(OffsetDateTime::UNIX_EPOCH)
}

/// `JJJJ-MM-TT HH:MM` in UTC.
pub fn iso(ms: i64) -> String {
    let d = datum(ms);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        d.year(),
        u8::from(d.month()),
        d.day(),
        d.hour(),
        d.minute()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Key32;
    use crate::model::*;
    use crate::vault::{TresorEintrag, VaultKeys};
    use ulid::Ulid;

    #[test]
    fn spiegel_und_bundle() {
        let ak = Key32::random().unwrap();
        let mut s = Store::open_in_memory(&ak, Ulid::new()).unwrap();
        let mut p = Projekt::neu(
            "Gartenhaus Fundament",
            Vorlage::HausGarten,
            1_700_000_000_000,
        );
        p.kurs = "Fundament gießen.".into();
        s.projekt_speichern(&p).unwrap();
        s.notiz_speichern(&Notiz::neu(
            p.id,
            Quelle::Cli,
            Art::Offen,
            "Bewehrung?",
            1_700_000_000_000,
        ))
        .unwrap();
        s.referenz_speichern(&Referenz::neu(
            p.id,
            ReferenzTyp::Physisch,
            "Keller, Regal 3",
            Rolle::Material,
        ))
        .unwrap();
        let vk = VaultKeys::from_account_key(&ak);
        let e = TresorEintrag::neu(
            &vk,
            "Router",
            vec![p.id],
            Stufe::Ueberall,
            &[("pw", "geheim")],
            1,
        )
        .unwrap();
        s.tresor_speichern(&e).unwrap();

        let t = tempfile::tempdir().unwrap();
        assert_eq!(spiegel::schreiben(&s, t.path()).unwrap(), 1);
        let md =
            fs::read_to_string(t.path().join("gartenhaus-fundament").join("projekt.md")).unwrap();
        assert!(md.contains("titel: \"Gartenhaus Fundament\""));
        assert!(md.contains("Keller, Regal 3"));
        assert!(!md.contains("geheim") && !md.contains("Router"));
        let lb = fs::read_to_string(
            t.path()
                .join("gartenhaus-fundament")
                .join("logbuch")
                .join("2023-11.md"),
        )
        .unwrap();
        assert!(lb.contains("Bewehrung?"));

        let ziel = t.path().join("bundle.json.age");
        let b = bundle::schreiben(&s, Some(&vk), "passphrase", &ziel).unwrap();
        assert_eq!(
            b.tresor[0].felder[0],
            ("pw".to_string(), "geheim".to_string())
        );
        let raw = fs::read(&ziel).unwrap();
        assert!(raw.starts_with(b"age-encryption.org/v1"));
        assert!(!String::from_utf8_lossy(&raw).contains("geheim"));
        let dec = bundle::age_entschluesseln(&raw, "passphrase").unwrap();
        let back: Bundle = serde_json::from_slice(&dec).unwrap();
        assert_eq!(back.projekte[0].titel, "Gartenhaus Fundament");
        assert!(bundle::age_entschluesseln(&raw, "falsch").is_err());

        // Ohne Schlüssel bleibt der Tresor im Bundle leer, aber benannt.
        let ohne = bundle::bauen(&s, None).unwrap();
        assert!(ohne.tresor.is_empty());
        assert_eq!(ohne.tresor_nicht_lesbar, vec!["Router".to_string()]);
    }
}
