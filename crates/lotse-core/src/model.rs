//! Datenmodell (siehe `docs/CONCEPT.md`, Abschnitt 3).
//!
//! Die Feldnamen sind bewusst deutsch und entsprechen dem Konzept eins zu eins, weil
//! dieselben Namen im Klartext-Spiegel, in der Oberfläche und in den Sync-Datensätzen
//! auftauchen. Zeitstempel sind Unix-Millisekunden, Datumsangaben ohne Uhrzeit sind
//! `JJJJ-MM-TT`-Strings.

use serde::{Deserialize, Serialize};
use ulid::Ulid;

/// Projekt-Status. Festes, kleines, nicht anklagendes Vokabular.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Idee,
    Aktiv,
    Pausiert,
    Wartet,
    Abgeschlossen,
    Eingemottet,
}

impl Status {
    pub const ALLE: [Status; 6] = [
        Status::Idee,
        Status::Aktiv,
        Status::Pausiert,
        Status::Wartet,
        Status::Abgeschlossen,
        Status::Eingemottet,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Status::Idee => "idee",
            Status::Aktiv => "aktiv",
            Status::Pausiert => "pausiert",
            Status::Wartet => "wartet",
            Status::Abgeschlossen => "abgeschlossen",
            Status::Eingemottet => "eingemottet",
        }
    }

    pub fn parse(s: &str) -> Option<Status> {
        Status::ALLE.into_iter().find(|st| st.as_str() == s)
    }

    /// Ruhende Zustände tauchen in keiner Rückstands-Statistik auf.
    pub fn ruht(self) -> bool {
        !matches!(self, Status::Aktiv)
    }

    /// Ein Wechsel in diese Zustände verlangt eine Übergabenotiz.
    pub fn verlangt_uebergabe(self) -> bool {
        matches!(self, Status::Pausiert | Status::Wartet)
    }
}

/// Start-Vorlagen. Setzen nur Defaults, das Datenmodell bleibt für alle gleich.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Vorlage {
    Software,
    HardwareMaker,
    HausGarten,
    Kreativ,
    FinanzenVerwaltung,
    LernenForschung,
    ReiseVeranstaltung,
    Generisch,
}

impl Vorlage {
    pub const ALLE: [Vorlage; 8] = [
        Vorlage::Software,
        Vorlage::HardwareMaker,
        Vorlage::HausGarten,
        Vorlage::Kreativ,
        Vorlage::FinanzenVerwaltung,
        Vorlage::LernenForschung,
        Vorlage::ReiseVeranstaltung,
        Vorlage::Generisch,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Vorlage::Software => "software",
            Vorlage::HardwareMaker => "hardware_maker",
            Vorlage::HausGarten => "haus_garten",
            Vorlage::Kreativ => "kreativ",
            Vorlage::FinanzenVerwaltung => "finanzen_verwaltung",
            Vorlage::LernenForschung => "lernen_forschung",
            Vorlage::ReiseVeranstaltung => "reise_veranstaltung",
            Vorlage::Generisch => "generisch",
        }
    }

    pub fn parse(s: &str) -> Option<Vorlage> {
        Vorlage::ALLE.into_iter().find(|v| v.as_str() == s)
    }

    pub fn anzeigename(self) -> &'static str {
        match self {
            Vorlage::Software => "Software",
            Vorlage::HardwareMaker => "Hardware & Maker",
            Vorlage::HausGarten => "Haus & Garten",
            Vorlage::Kreativ => "Kreativ",
            Vorlage::FinanzenVerwaltung => "Finanzen & Verwaltung",
            Vorlage::LernenForschung => "Lernen & Forschung",
            Vorlage::ReiseVeranstaltung => "Reise & Veranstaltung",
            Vorlage::Generisch => "Generisch",
        }
    }

    /// Ab wie vielen Tagen Stille ein Projekt dieser Art auffällig wird.
    pub fn erwartungsintervall_tage(self) -> u32 {
        match self {
            Vorlage::Software => 14,
            Vorlage::HardwareMaker => 30,
            Vorlage::HausGarten => 60,
            Vorlage::Kreativ => 30,
            Vorlage::FinanzenVerwaltung => 90,
            Vorlage::LernenForschung => 30,
            Vorlage::ReiseVeranstaltung => 120,
            Vorlage::Generisch => 30,
        }
    }

    pub fn standard_tags(self) -> &'static [&'static str] {
        match self {
            Vorlage::Software => &["software"],
            Vorlage::HardwareMaker => &["hardware", "maker"],
            Vorlage::HausGarten => &["haus"],
            Vorlage::Kreativ => &["kreativ"],
            Vorlage::FinanzenVerwaltung => &["verwaltung"],
            Vorlage::LernenForschung => &["lernen"],
            Vorlage::ReiseVeranstaltung => &["reise"],
            Vorlage::Generisch => &[],
        }
    }
}

/// Projekt (Wurzel).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Projekt {
    pub id: Ulid,
    pub titel: String,
    /// 1–2 Sätze in eigenen Worten: worum geht es, was ist das Ziel.
    #[serde(default)]
    pub kurs: String,
    pub status: Status,
    /// `JJJJ-MM-TT`, für `pausiert` und `wartet`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wiedervorlage: Option<String>,
    pub erwartungsintervall_tage: u32,
    #[serde(default)]
    pub tags: Vec<String>,
    pub vorlage: Vorlage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abgeleitet_von: Option<Ulid>,
    pub angelegt: i64,
    pub zuletzt_beruehrt: i64,
}

impl Projekt {
    pub fn neu(titel: impl Into<String>, vorlage: Vorlage, jetzt_ms: i64) -> Projekt {
        Projekt {
            id: Ulid::new(),
            titel: titel.into(),
            kurs: String::new(),
            status: Status::Aktiv,
            wiedervorlage: None,
            erwartungsintervall_tage: vorlage.erwartungsintervall_tage(),
            tags: vorlage
                .standard_tags()
                .iter()
                .map(|t| t.to_string())
                .collect(),
            vorlage,
            abgeleitet_von: None,
            angelegt: jetzt_ms,
            zuletzt_beruehrt: jetzt_ms,
        }
    }
}

/// Woher ein Logbuch-Eintrag stammt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quelle {
    Mensch,
    Cli,
    Datei,
    Git,
    Import,
    Mcp,
    Ki,
    Sync,
}

impl Quelle {
    pub fn as_str(self) -> &'static str {
        match self {
            Quelle::Mensch => "mensch",
            Quelle::Cli => "cli",
            Quelle::Datei => "datei",
            Quelle::Git => "git",
            Quelle::Import => "import",
            Quelle::Mcp => "mcp",
            Quelle::Ki => "ki",
            Quelle::Sync => "sync",
        }
    }

    pub fn parse(s: &str) -> Option<Quelle> {
        [
            Quelle::Mensch,
            Quelle::Cli,
            Quelle::Datei,
            Quelle::Git,
            Quelle::Import,
            Quelle::Mcp,
            Quelle::Ki,
            Quelle::Sync,
        ]
        .into_iter()
        .find(|q| q.as_str() == s)
    }
}

/// Art eines Logbuch-Eintrags. Faden, Eintrag, Entscheidung, Statuswechsel und
/// Übergabenotiz sind bewusst eine Entität.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Art {
    Log,
    Offen,
    Entscheidung,
    Status,
    Uebergabe,
}

impl Art {
    pub fn as_str(self) -> &'static str {
        match self {
            Art::Log => "log",
            Art::Offen => "offen",
            Art::Entscheidung => "entscheidung",
            Art::Status => "status",
            Art::Uebergabe => "uebergabe",
        }
    }

    pub fn parse(s: &str) -> Option<Art> {
        [
            Art::Log,
            Art::Offen,
            Art::Entscheidung,
            Art::Status,
            Art::Uebergabe,
        ]
        .into_iter()
        .find(|a| a.as_str() == s)
    }
}

/// Logbuch-Eintrag. Append-only; Korrekturen sind neue Notizen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notiz {
    pub id: Ulid,
    pub projekt_id: Ulid,
    pub ts: i64,
    pub quelle: Quelle,
    pub art: Art,
    pub text: String,
    /// Nur für `art = offen`: wann der Faden erledigt wurde.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub erledigt_am: Option<i64>,
}

impl Notiz {
    pub fn neu(
        projekt_id: Ulid,
        quelle: Quelle,
        art: Art,
        text: impl Into<String>,
        ts: i64,
    ) -> Notiz {
        Notiz {
            id: Ulid::new(),
            projekt_id,
            ts,
            quelle,
            art,
            text: text.into(),
            erledigt_am: None,
        }
    }

    pub fn ist_offen(&self) -> bool {
        self.art == Art::Offen && self.erledigt_am.is_none()
    }

    /// Vorlage für eine Entscheidung: ein Freitext mit Überschriften, keine Pflicht-Unterfelder.
    pub fn entscheidungs_vorlage() -> &'static str {
        "**Kontext:** \n\n**Entschieden:** \n\n**Verworfen, weil:** \n\n**Neu bewerten, wenn:** "
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenzTyp {
    Ordner,
    GitRepo,
    Url,
    Datei,
    Physisch,
    Geraet,
    Passwortmanager,
    Anhang,
}

impl ReferenzTyp {
    pub fn as_str(self) -> &'static str {
        match self {
            ReferenzTyp::Ordner => "ordner",
            ReferenzTyp::GitRepo => "git_repo",
            ReferenzTyp::Url => "url",
            ReferenzTyp::Datei => "datei",
            ReferenzTyp::Physisch => "physisch",
            ReferenzTyp::Geraet => "geraet",
            ReferenzTyp::Passwortmanager => "passwortmanager",
            ReferenzTyp::Anhang => "anhang",
        }
    }

    pub fn parse(s: &str) -> Option<ReferenzTyp> {
        [
            ReferenzTyp::Ordner,
            ReferenzTyp::GitRepo,
            ReferenzTyp::Url,
            ReferenzTyp::Datei,
            ReferenzTyp::Physisch,
            ReferenzTyp::Geraet,
            ReferenzTyp::Passwortmanager,
            ReferenzTyp::Anhang,
        ]
        .into_iter()
        .find(|t| t.as_str() == s)
    }

    /// Pfade gelten nur auf dem Gerät, auf dem sie angelegt wurden.
    ///
    /// Achtung: Der Typ allein entscheidet das nicht. `git_repo` steht für zwei Dinge –
    /// einen Ordner mit `.git` **und** eine Adresse wie `https://github.com/o/r`. Nur das
    /// erste ist gerätegebunden. Deshalb fragt man besser `Referenz::geraetegebunden`,
    /// die auch das Ziel ansieht.
    pub fn geraetegebunden(self) -> bool {
        matches!(
            self,
            ReferenzTyp::Ordner | ReferenzTyp::GitRepo | ReferenzTyp::Datei
        )
    }
}

/// Zeigt das Ziel ins Netz statt auf die Platte?
///
/// `git_repo` kann beides sein. Wer das verwechselt, sucht `https://github.com/o/r` im
/// Dateisystem, findet es nicht und meldet »nicht erreichbar« – für eine Adresse, die es
/// gibt.
pub fn ziel_ist_adresse(ziel: &str) -> bool {
    let z = ziel.trim();
    ["http://", "https://", "ssh://", "git://"]
        .iter()
        .any(|p| z.len() > p.len() && z[..p.len()].eq_ignore_ascii_case(p))
        // `git@github.com:o/r.git` – die Kurzform, die `git remote -v` ausgibt.
        || (z.starts_with("git@") && z.contains(':'))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rolle {
    Material,
    Ergebnis,
    Doku,
}

impl Rolle {
    pub fn as_str(self) -> &'static str {
        match self {
            Rolle::Material => "material",
            Rolle::Ergebnis => "ergebnis",
            Rolle::Doku => "doku",
        }
    }
    pub fn parse(s: &str) -> Option<Rolle> {
        [Rolle::Material, Rolle::Ergebnis, Rolle::Doku]
            .into_iter()
            .find(|r| r.as_str() == s)
    }
}

/// `nicht_pruefbar` ist ein ehrlicher Zustand, kein Fehler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pruefstatus {
    Ok,
    NichtErreichbar,
    NichtPruefbar,
}

impl Pruefstatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Pruefstatus::Ok => "ok",
            Pruefstatus::NichtErreichbar => "nicht_erreichbar",
            Pruefstatus::NichtPruefbar => "nicht_pruefbar",
        }
    }
    pub fn parse(s: &str) -> Option<Pruefstatus> {
        [
            Pruefstatus::Ok,
            Pruefstatus::NichtErreichbar,
            Pruefstatus::NichtPruefbar,
        ]
        .into_iter()
        .find(|p| p.as_str() == s)
    }
}

/// Referenz: zeigt nach außen. Lotse zieht nichts nach innen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Referenz {
    pub id: Ulid,
    pub projekt_id: Ulid,
    pub typ: ReferenzTyp,
    /// Pfad, URL oder Ortsbeschreibung ("Keller, Regal 3, blaue Kiste").
    pub ziel: String,
    pub rolle: Rolle,
    /// Für gerätegebundene Pfade: auf welchem Gerät der Pfad gilt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geraet_id: Option<Ulid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zuletzt_geprueft: Option<i64>,
    pub pruefstatus: Pruefstatus,
}

impl Referenz {
    /// Gilt diese Referenz nur auf einem Gerät? Adressen gelten überall.
    pub fn geraetegebunden(&self) -> bool {
        self.typ.geraetegebunden() && !ziel_ist_adresse(&self.ziel)
    }

    pub fn neu(
        projekt_id: Ulid,
        typ: ReferenzTyp,
        ziel: impl Into<String>,
        rolle: Rolle,
    ) -> Referenz {
        // Bis zur ersten Prüfung ist jede Referenz »nicht geprüft« – ehrlich, kein Fehler.
        let pruefstatus = Pruefstatus::NichtPruefbar;
        Referenz {
            id: Ulid::new(),
            projekt_id,
            typ,
            ziel: ziel.into(),
            rolle,
            geraet_id: None,
            zuletzt_geprueft: None,
            pruefstatus,
        }
    }
}

/// Tresor-Stufe. `nur_desktop` braucht zusätzlich den Desktop-Schlüssel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stufe {
    Ueberall,
    NurDesktop,
}

impl Stufe {
    pub fn as_str(self) -> &'static str {
        match self {
            Stufe::Ueberall => "ueberall",
            Stufe::NurDesktop => "nur_desktop",
        }
    }
    pub fn parse(s: &str) -> Option<Stufe> {
        [Stufe::Ueberall, Stufe::NurDesktop]
            .into_iter()
            .find(|st| st.as_str() == s)
    }
}

/// Gerät, auf dem ein Client installiert ist.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Geraet {
    pub id: Ulid,
    pub name: String,
    pub plattform: String,
    pub angelegt: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zuletzt_sync: Option<i64>,
}

/// Kandidat aus der Projekterkennung (Hafeneinfahrt). Kein Sync-Datensatz; lebt nur
/// lokal, bis er bestätigt wurde.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kandidat {
    pub pfad: String,
    pub name: String,
    pub vorlage: Vorlage,
    pub marken: Vec<String>,
    /// Gesetzt, wenn der Ordner eine `.lotse-projekt`-Marker-Datei trägt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bekannte_id: Option<Ulid>,
    pub hat_git: bool,
    pub readme: Option<String>,
}

/// Erlaubte Zeichen für Dateinamen im Klartext-Spiegel.
pub fn slug(titel: &str) -> String {
    let mut out = String::with_capacity(titel.len());
    let mut letzter_strich = true;
    for c in titel.chars() {
        let c = match c {
            'ä' => 'a',
            'ö' => 'o',
            'ü' => 'u',
            'Ä' => 'a',
            'Ö' => 'o',
            'Ü' => 'u',
            'ß' => 's',
            c => c,
        };
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            letzter_strich = false;
        } else if !letzter_strich {
            out.push('-');
            letzter_strich = true;
        }
    }
    let out = out.trim_end_matches('-').to_string();
    if out.is_empty() {
        "projekt".to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_roundtrip() {
        for s in Status::ALLE {
            assert_eq!(Status::parse(s.as_str()), Some(s));
            let json = serde_json::to_string(&s).unwrap();
            assert_eq!(json, format!("\"{}\"", s.as_str()));
        }
    }

    #[test]
    fn vorlage_defaults() {
        let p = Projekt::neu("Gartenhaus", Vorlage::HausGarten, 1);
        assert_eq!(p.erwartungsintervall_tage, 60);
        assert_eq!(p.status, Status::Aktiv);
        assert_eq!(p.tags, vec!["haus"]);
    }

    #[test]
    fn slug_umlaute() {
        assert_eq!(slug("Gartenhaus Fundament"), "gartenhaus-fundament");
        assert_eq!(slug("Übung: Löten & Prüfen!"), "ubung-loten-prufen");
        assert_eq!(slug("   "), "projekt");
    }
}
