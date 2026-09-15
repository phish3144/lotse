//! Deuten: aus einer Quelle einen Befund machen.
//!
//! Eine Quelle ist ein Ordner, eine Adresse oder eine Handvoll Dateien. Ein Befund ist,
//! was darin gefunden wurde – **nie eine Frage, immer eine Feststellung**. Was Lotse
//! bereits weiß, wird nicht erfragt: Hat der Ordner ein Remote, steht es im Befund, statt
//! dass jemand nach »von GitHub importieren?« gefragt wird. Wer aus einer Adresse kommt,
//! hat die Frage ohnehin schon beantwortet, indem er sie eingegeben hat.
//!
//! Derselbe Baustein trägt beide Fälle: ein Projekt anlegen und an ein bestehendes
//! anhängen. Der Unterschied liegt danach, nicht hier.
//!
//! Dieses Modul liest nur und schreibt nichts. Es kennt weder Store noch Tresor.

use std::path::{Path, PathBuf};

use serde::Serialize;
use ulid::Ulid;
use walkdir::WalkDir;

use crate::detect::{self, ScanOptionen};
use crate::dokument;
use crate::forge::RepoZeiger;
use crate::git;
use crate::model::Vorlage;
use crate::Result;

/// Mehr Dokumente als das zeigt keine Liste sinnvoll an; der Rest wird gezählt und
/// genannt, nicht verschwiegen.
const MAX_DOKUMENTE: usize = 20;

/// Woher gedeutet wird.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Quelle {
    Ordner(PathBuf),
    Adresse(String),
    Dateien(Vec<PathBuf>),
    /// Kein Anhalt außer dem, was jemand getippt hat. Kein Sonderfall: ein Vorhaben
    /// braucht keine Quelle, und wer nur einen Namen weiß, soll nicht erst einen
    /// zweiten Weg suchen müssen.
    Titel(String),
}

/// Ordnet ein, was jemand in ein einziges Feld geworfen hat.
///
/// Ein Feld für alles, weil die Frage »ist das ein Ordner, eine Adresse oder ein Titel?«
/// eine ist, die das Programm beantworten kann. Wer sie stellt, verlangt vom Menschen
/// eine Einordnung, die er gerade gar nicht vorhatte.
///
/// Die Reihenfolge ist Absicht:
///
/// 1. **Adressen** zuerst, an ihrem Schema erkannt. Sie liegen nie auf der Platte, also
///    hat ein Dateisystem-Blick hier nichts zu suchen.
/// 2. **Pfade** nur mit Anker – `/`, `~/`, `./`, `../`, `\\` oder ein Laufwerksbuchstabe.
///    Ohne Anker bleibt »Haus/Garten« ein Titel; in einem Dialogfeld tippt niemand
///    relative Pfade, aber Titel mit Schrägstrich sehr wohl.
/// 3. **Alles andere** ist der Titel.
///
/// Ein angekerter Pfad, den es nicht gibt, ist ein **Fehler** und kein Titel: wer
/// `/home/ich/Grten` tippt, hat sich vertippt und will das hören, statt ein Vorhaben mit
/// diesem Namen zu bekommen.
pub fn einordnen(text: &str) -> Result<Quelle> {
    let t = text.trim();
    if t.is_empty() {
        return Ok(Quelle::Titel(String::new()));
    }
    if ist_adresse(t) {
        return Ok(Quelle::Adresse(t.to_string()));
    }
    let Some(pfad) = als_pfad(t) else {
        return Ok(Quelle::Titel(t.to_string()));
    };
    if pfad.is_dir() {
        return Ok(Quelle::Ordner(pfad));
    }
    if pfad.is_file() {
        return Ok(Quelle::Dateien(vec![pfad]));
    }
    Err(crate::Error::NotFound(format!(
        "Den Pfad »{}« gibt es nicht. Tippfehler? Wenn es ein Titel sein soll, lass den \
         führenden Schrägstrich weg.",
        pfad.display()
    )))
}

/// Erkennt eine Adresse am Schema. `www.` zählt mit, weil das jeder aus der Adresszeile
/// kopiert; `git@host:pfad` auch, weil das die Form ist, die GitHub zum Klonen anbietet.
fn ist_adresse(t: &str) -> bool {
    let l = t.to_ascii_lowercase();
    for schema in [
        "http://",
        "https://",
        "webcal://",
        "ssh://",
        "git://",
        "ftp://",
        "ftps://",
    ] {
        if l.starts_with(schema) {
            return true;
        }
    }
    if l.starts_with("www.") {
        return true;
    }
    // `git@github.com:owner/repo` – ein Doppelpunkt nach dem Wirt, kein Laufwerk.
    if let Some((vorn, hinten)) = t.split_once('@') {
        if !vorn.is_empty() && hinten.contains(':') && hinten.contains('.') {
            return true;
        }
    }
    false
}

/// Macht aus dem Text einen Pfad, wenn er wie einer **beginnt**. `file://` wird
/// entpackt, `~` durch das Heimatverzeichnis ersetzt.
fn als_pfad(t: &str) -> Option<PathBuf> {
    if let Some(rest) = t.strip_prefix("file://") {
        // `file:///tmp/x` – der dritte Schrägstrich gehört zum Pfad.
        let rest = rest.strip_prefix("localhost").unwrap_or(rest);
        return Some(PathBuf::from(if rest.starts_with('/') {
            rest.to_string()
        } else {
            format!("/{rest}")
        }));
    }
    if t == "~" || t.starts_with("~/") || t.starts_with("~\\") {
        let heim = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)?;
        let rest = t[1..].trim_start_matches(['/', '\\']);
        return Some(if rest.is_empty() {
            heim
        } else {
            heim.join(rest)
        });
    }
    let anker = t.starts_with('/')
        || t.starts_with("./")
        || t.starts_with("../")
        || t.starts_with(".\\")
        || t.starts_with("..\\")
        || t.starts_with("\\\\")
        || laufwerk(t);
    anker.then(|| PathBuf::from(t))
}

/// `C:\…` oder `C:/…` – ein Laufwerksbuchstabe, kein `git@host:pfad`.
fn laufwerk(t: &str) -> bool {
    let b = t.as_bytes();
    b.len() >= 3 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/')
}

/// Ein einzelner Fund. Jeder wird in der Oberfläche eine Zeile mit Haken, alle
/// vorangekreuzt. Was nicht gefunden wurde, taucht nicht auf.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "art", rename_all = "snake_case")]
pub enum Fund {
    /// Der Ordner gehört bereits zu einem Projekt. Dann wird nichts Neues angelegt,
    /// sondern angeboten, den Rest dort anzuhängen.
    SchonBekannt {
        id: Ulid,
        pfad: String,
    },
    Remote {
        url: String,
        /// Erkannte Gegenseite, falls es eine bekannte ist.
        dienst: Option<String>,
    },
    Unterprojekt {
        pfad: String,
        name: String,
        vorlage: String,
        marken: Vec<String>,
    },
    Dokument {
        pfad: String,
        name: String,
    },
    /// Projektseite, die das Repo selbst angibt. Wird eine Referenz.
    Startseite {
        url: String,
    },
}

/// Was Lotse für Titel, Kurs und Vorlage vorschlägt. Alles änderbar – es ist ein
/// Vorschlag, kein Befehl.
#[derive(Debug, Clone, Serialize)]
pub struct Vorschlag {
    pub titel: String,
    pub kurs: Option<String>,
    pub vorlage: Vorlage,
    /// Vorgeschlagene Tags. Kommen aus der Vorlage und, bei einem Repo, aus seinen Themen.
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Befund {
    /// Wie die Quelle hieß, in lesbarer Form – für die Überschrift des Befunds.
    pub quelle: String,
    /// Fehlt, wenn die Quelle bereits zu einem Projekt gehört.
    pub vorschlag: Option<Vorschlag>,
    pub funde: Vec<Fund>,
    pub angesehen: usize,
    /// Die Suche hat an ihrer Grenze aufgehört. Kein Fehler, aber es muss dastehen –
    /// sonst hält man die Liste für vollständig.
    pub abgebrochen: bool,
    /// Dokumente über die aufgeführten hinaus.
    pub weitere_dokumente: usize,
    /// Die Gegenseite sagt, dort passiere nichts mehr. Gehört in den Befund: sonst
    /// wartet jemand auf Bewegung, die nicht kommt.
    #[serde(default)]
    pub archiviert: bool,
    /// Die Gegenseite war nicht erreichbar. Kein Abbruch – der Befund gilt trotzdem,
    /// aber er soll nicht so aussehen, als hätte es dort nichts gegeben.
    #[serde(default)]
    pub gegenseite_fehler: Option<String>,
}

pub fn deuten(quelle: &Quelle, opt: &ScanOptionen) -> Result<Befund> {
    match quelle {
        Quelle::Ordner(p) => ordner(p, opt),
        Quelle::Adresse(a) => adresse(a),
        Quelle::Dateien(d) => dateien(d),
        Quelle::Titel(t) => Ok(nur_titel(t)),
    }
}

/// Ein Befund ohne Funde. Keine leere Seite: Titel und Vorlage stehen drin, der Rest ist
/// eben nichts – und das ist eine Feststellung wie jede andere.
fn nur_titel(titel: &str) -> Befund {
    Befund {
        quelle: titel.trim().to_string(),
        vorschlag: Some(Vorschlag {
            titel: titel.trim().to_string(),
            kurs: None,
            vorlage: Vorlage::Generisch,
            tags: Vec::new(),
        }),
        funde: Vec::new(),
        angesehen: 0,
        abgebrochen: false,
        weitere_dokumente: 0,
        archiviert: false,
        gegenseite_fehler: None,
    }
}

/// Trägt in den Befund ein, was die Gegenseite über sich selbst sagt.
///
/// **Was schon dasteht, bleibt stehen.** Eine README auf der Platte kennt das Vorhaben
/// besser als ein Einzeiler auf GitHub, und was von dort kommt, soll nichts überschreiben,
/// was aus den eigenen Dateien gelesen wurde.
///
/// Reine Funktion: das Holen macht die Hülle, weil dafür ein Token aus dem Tresor nötig
/// sein kann – und der ist für dieses Modul unerreichbar.
pub fn anreichern(befund: &mut Befund, sb: &crate::forge::Steckbrief) {
    befund.archiviert = sb.archiviert;

    if let Some(v) = befund.vorschlag.as_mut() {
        if v.kurs.is_none() {
            // Die Beschreibung zuerst: sie ist als Einzeiler geschrieben, eine README
            // nicht. Erst wenn sie fehlt, die erste brauchbare README-Zeile.
            v.kurs = sb
                .beschreibung
                .clone()
                .or_else(|| sb.readme.as_deref().and_then(detect::kurs_vorschlag));
        }
        for thema in &sb.themen {
            let t = thema.trim();
            if !t.is_empty() && !v.tags.iter().any(|x| x.eq_ignore_ascii_case(t)) {
                v.tags.push(t.to_string());
            }
        }
    }

    // Die Startseite nur, wenn sie nicht dasselbe ist wie der Remote – zweimal derselbe
    // Link hilft niemandem.
    if let Some(url) = sb.startseite.as_deref() {
        let schon = befund.funde.iter().any(|f| match f {
            Fund::Remote { url: u, .. } | Fund::Startseite { url: u } => gleiche_adresse(u, url),
            _ => false,
        });
        if !schon {
            befund.funde.push(Fund::Startseite {
                url: url.to_string(),
            });
        }
    }
}

/// Zwei Adressen, dieselbe Stelle? Schema, `www.` und ein Schlussschrägstrich
/// unterscheiden nichts.
fn gleiche_adresse(a: &str, b: &str) -> bool {
    fn kern(s: &str) -> String {
        let l = s.trim().trim_end_matches('/').to_ascii_lowercase();
        let ohne = l
            .strip_prefix("https://")
            .or_else(|| l.strip_prefix("http://"))
            .unwrap_or(&l);
        ohne.strip_prefix("www.").unwrap_or(ohne).to_string()
    }
    kern(a) == kern(b)
}

fn ordner(dir: &Path, opt: &ScanOptionen) -> Result<Befund> {
    let quelle = dir.to_string_lossy().to_string();
    let mut funde = Vec::new();

    // Gehört der Ordner schon zu einem Projekt, ist alles Weitere hinfällig: es gibt
    // nichts anzulegen, nur etwas anzuhängen.
    if let Some(id) = detect::marker_lesen(dir) {
        funde.push(Fund::SchonBekannt {
            id,
            pfad: quelle.clone(),
        });
        return Ok(Befund {
            quelle,
            vorschlag: None,
            funde,
            angesehen: 1,
            abgebrochen: false,
            weitere_dokumente: 0,
            archiviert: false,
            gegenseite_fehler: None,
        });
    }

    // `detect::scan` beginnt eine Ebene tiefer. Der gewählte Ordner selbst ist aber der
    // wahrscheinlichste Treffer überhaupt – den muss man eigens ansehen.
    let selbst = detect::erkenne(dir);

    if let Some(url) = git::remotes(dir).into_iter().next() {
        let dienst = RepoZeiger::erkennen(&url).map(|z| z.anbieter.as_str().to_string());
        funde.push(Fund::Remote { url, dienst });
    }

    let unten = detect::scan(std::slice::from_ref(&dir.to_path_buf()), opt)?;
    for k in &unten.kandidaten {
        funde.push(Fund::Unterprojekt {
            pfad: k.pfad.clone(),
            name: k.name.clone(),
            vorlage: k.vorlage.as_str().to_string(),
            marken: k.marken.clone(),
        });
    }

    let (dok, weitere) = dokumente_suchen(dir, opt);
    funde.extend(dok);

    let vorschlag = Some(Vorschlag {
        titel: selbst
            .as_ref()
            .map(|k| k.name.clone())
            .unwrap_or_else(|| ordnername(dir)),
        kurs: selbst
            .as_ref()
            .and_then(|k| k.readme.as_deref())
            .and_then(detect::kurs_vorschlag),
        vorlage: selbst
            .as_ref()
            .map(|k| k.vorlage)
            .unwrap_or(Vorlage::Generisch),
        tags: Vec::new(),
    });

    Ok(Befund {
        quelle,
        vorschlag,
        funde,
        angesehen: unten.angesehen,
        abgebrochen: unten.abgebrochen,
        weitere_dokumente: weitere,
        archiviert: false,
        gegenseite_fehler: None,
    })
}

fn adresse(a: &str) -> Result<Befund> {
    let zeiger = RepoZeiger::erkennen(a);
    let titel = zeiger
        .as_ref()
        .map(|z| z.repo.clone())
        .unwrap_or_else(|| letzter_teil(a));
    let dienst = zeiger.as_ref().map(|z| z.anbieter.as_str().to_string());

    Ok(Befund {
        quelle: a.to_string(),
        vorschlag: Some(Vorschlag {
            titel,
            kurs: None,
            // Eine Repo-Adresse ist Software. Bei allem anderen maßt Lotse sich nichts an.
            vorlage: if zeiger.is_some() {
                Vorlage::Software
            } else {
                Vorlage::Generisch
            },
            tags: Vec::new(),
        }),
        funde: vec![Fund::Remote {
            url: a.to_string(),
            dienst,
        }],
        angesehen: 0,
        abgebrochen: false,
        weitere_dokumente: 0,
        archiviert: false,
        gegenseite_fehler: None,
    })
}

fn dateien(pfade: &[PathBuf]) -> Result<Befund> {
    let mut funde = Vec::new();
    for p in pfade.iter().take(MAX_DOKUMENTE) {
        if dokument::format_von(p).is_some() {
            funde.push(Fund::Dokument {
                pfad: p.to_string_lossy().to_string(),
                name: dateiname(p),
            });
        }
    }
    let titel = pfade
        .first()
        .map(|p| stamm(p))
        .unwrap_or_else(|| "Neues Vorhaben".to_string());

    Ok(Befund {
        quelle: format!("{} Datei(en)", pfade.len()),
        vorschlag: Some(Vorschlag {
            titel,
            kurs: None,
            vorlage: Vorlage::Generisch,
            tags: Vec::new(),
        }),
        funde,
        angesehen: 0,
        abgebrochen: false,
        weitere_dokumente: pfade.len().saturating_sub(MAX_DOKUMENTE),
        archiviert: false,
        gegenseite_fehler: None,
    })
}

/// Deutbare Dateien im Ordner. Nur zwei Ebenen tief: was tiefer liegt, gehört
/// erfahrungsgemäß zum Werkzeug und nicht zum Vorhaben.
fn dokumente_suchen(dir: &Path, opt: &ScanOptionen) -> (Vec<Fund>, usize) {
    let mut gefunden = Vec::new();
    let mut gezaehlt = 0usize;

    let it = WalkDir::new(dir)
        .min_depth(1)
        .max_depth(2)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !(name.starts_with('.') || opt.ausschluss.iter().any(|a| a == name.as_ref()))
        });

    for entry in it.flatten() {
        if !entry.file_type().is_file() {
            continue;
        }
        if detect::nie_lesen(&entry.file_name().to_string_lossy()) {
            continue;
        }
        if dokument::format_von(entry.path()).is_none() {
            continue;
        }
        gezaehlt += 1;
        if gefunden.len() < MAX_DOKUMENTE {
            gefunden.push(Fund::Dokument {
                pfad: entry.path().to_string_lossy().to_string(),
                name: dateiname(entry.path()),
            });
        }
    }
    (gefunden, gezaehlt.saturating_sub(MAX_DOKUMENTE))
}

fn ordnername(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| p.to_string_lossy().to_string())
}

fn dateiname(p: &Path) -> String {
    p.file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn stamm(p: &Path) -> String {
    p.file_stem()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Neues Vorhaben".to_string())
}

fn letzter_teil(a: &str) -> String {
    a.trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or(a)
        .trim_end_matches(".git")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn git(dir: &Path, args: &[&str]) {
        std::process::Command::new("git")
            .current_dir(dir)
            .args(args)
            .output()
            .expect("git");
    }

    // ------------------------------------------- Ein Feld nimmt alles

    #[test]
    fn adressen_werden_am_schema_erkannt() {
        for t in [
            "https://github.com/o/r",
            "http://example.org",
            "www.example.org/kalender.ics",
            "webcal://example.org/k.ics",
            "git@github.com:owner/repo.git",
            "ssh://git@example.org/r.git",
        ] {
            assert_eq!(
                einordnen(t).unwrap(),
                Quelle::Adresse(t.to_string()),
                "{t} sollte eine Adresse sein"
            );
        }
    }

    #[test]
    fn ein_titel_mit_schraegstrich_bleibt_ein_titel() {
        // Ohne Anker ist es kein Pfad. In einem Dialogfeld tippt niemand relative Pfade,
        // aber »Haus/Garten« oder »Kunde: Meier/2026« sehr wohl.
        for t in ["Haus/Garten", "Gartenhaus", "Kunde Meier 2026", "C:Bericht"] {
            assert_eq!(
                einordnen(t).unwrap(),
                Quelle::Titel(t.to_string()),
                "{t} sollte ein Titel sein"
            );
        }
    }

    #[test]
    fn leeres_feld_ist_ein_leerer_titel_kein_fehler() {
        assert_eq!(einordnen("   ").unwrap(), Quelle::Titel(String::new()));
    }

    #[test]
    fn ordner_und_datei_werden_am_dateisystem_unterschieden() {
        let t = tempfile::tempdir().unwrap();
        let datei = t.path().join("angebot.pdf");
        fs::write(&datei, b"%PDF-1.4").unwrap();

        let dir_text = t.path().to_string_lossy().to_string();
        assert_eq!(
            einordnen(&dir_text).unwrap(),
            Quelle::Ordner(t.path().to_path_buf())
        );
        assert_eq!(
            einordnen(&datei.to_string_lossy()).unwrap(),
            Quelle::Dateien(vec![datei.clone()])
        );
        // Leerzeichen drumherum passieren beim Kopieren aus dem Dateimanager.
        assert_eq!(
            einordnen(&format!("  {dir_text}  ")).unwrap(),
            Quelle::Ordner(t.path().to_path_buf())
        );
    }

    #[test]
    fn file_url_wird_entpackt() {
        let t = tempfile::tempdir().unwrap();
        let url = format!("file://{}", t.path().display());
        assert_eq!(
            einordnen(&url).unwrap(),
            Quelle::Ordner(t.path().to_path_buf())
        );
    }

    #[test]
    fn tilde_wird_ersetzt() {
        let heim = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"));
        if heim.is_none() {
            return;
        }
        // `~` selbst ist ein Ordner, der es gibt.
        assert!(matches!(einordnen("~").unwrap(), Quelle::Ordner(_)));
    }

    #[test]
    fn ein_pfad_der_fehlt_ist_ein_fehler_kein_titel() {
        // Wer sich vertippt, will das hören – und kein Vorhaben namens »/home/ich/Grten«.
        let e = einordnen("/gibt/es/ganz/sicher/nicht/4f2a").unwrap_err();
        let text = e.to_string();
        assert!(text.contains("gibt es nicht"), "{text}");
        assert!(
            text.contains("Titel"),
            "die Meldung muss den Ausweg nennen: {text}"
        );
    }

    #[test]
    fn nur_ein_titel_gibt_einen_befund_ohne_funde() {
        let b = deuten(
            &Quelle::Titel("Gartenhaus".into()),
            &ScanOptionen::default(),
        )
        .unwrap();
        let v = b.vorschlag.expect("Vorschlag");
        assert_eq!(v.titel, "Gartenhaus");
        assert_eq!(v.vorlage, Vorlage::Generisch);
        assert!(b.funde.is_empty());
        assert!(!b.abgebrochen);
    }

    // --------------------------------------- Was die Gegenseite beisteuert

    fn steckbrief() -> crate::forge::Steckbrief {
        crate::forge::Steckbrief {
            beschreibung: Some("Das Logbuch für alle Vorhaben".into()),
            themen: vec!["rust".into(), "tauri".into()],
            startseite: Some("https://lotse.sanctora.eu/".into()),
            standard_branch: Some("main".into()),
            archiviert: false,
            privat: false,
            readme: Some("# Lotse\n\nAus der README.\n".into()),
        }
    }

    #[test]
    fn die_beschreibung_wird_zum_kurs() {
        let mut b = deuten(
            &Quelle::Adresse("https://github.com/o/r".into()),
            &ScanOptionen::default(),
        )
        .unwrap();
        assert_eq!(b.vorschlag.as_ref().unwrap().kurs, None, "vorher leer");

        anreichern(&mut b, &steckbrief());
        let v = b.vorschlag.unwrap();
        assert_eq!(v.kurs.as_deref(), Some("Das Logbuch für alle Vorhaben"));
        assert_eq!(v.tags, vec!["rust".to_string(), "tauri".to_string()]);
    }

    #[test]
    fn ohne_beschreibung_hilft_die_readme_aus() {
        let mut b = deuten(
            &Quelle::Adresse("https://github.com/o/r".into()),
            &ScanOptionen::default(),
        )
        .unwrap();
        let sb = crate::forge::Steckbrief {
            beschreibung: None,
            ..steckbrief()
        };
        anreichern(&mut b, &sb);
        // Die Überschrift wird übersprungen, wie bei einer README auf der Platte.
        assert_eq!(
            b.vorschlag.unwrap().kurs.as_deref(),
            Some("Aus der README.")
        );
    }

    #[test]
    fn ein_kurs_von_der_platte_wird_nicht_ueberschrieben() {
        // Die eigenen Dateien kennen das Vorhaben besser als ein Einzeiler auf GitHub.
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("Gartenhaus");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("README.md"),
            "# Gartenhaus\n\nFundament bis Oktober.\n",
        )
        .unwrap();

        let mut b = deuten(&Quelle::Ordner(dir), &ScanOptionen::default()).unwrap();
        anreichern(&mut b, &steckbrief());
        assert_eq!(
            b.vorschlag.unwrap().kurs.as_deref(),
            Some("Fundament bis Oktober.")
        );
    }

    #[test]
    fn die_startseite_wird_ein_fund() {
        let mut b = deuten(
            &Quelle::Adresse("https://github.com/o/r".into()),
            &ScanOptionen::default(),
        )
        .unwrap();
        anreichern(&mut b, &steckbrief());
        assert!(b
            .funde
            .iter()
            .any(|f| matches!(f, Fund::Startseite { url } if url.contains("sanctora"))));
    }

    #[test]
    fn dieselbe_adresse_kommt_nicht_zweimal() {
        // Viele Repos tragen sich selbst als Startseite ein.
        let mut b = deuten(
            &Quelle::Adresse("https://github.com/o/r".into()),
            &ScanOptionen::default(),
        )
        .unwrap();
        let sb = crate::forge::Steckbrief {
            startseite: Some("http://www.GitHub.com/o/r/".into()),
            ..steckbrief()
        };
        anreichern(&mut b, &sb);
        assert!(
            !b.funde.iter().any(|f| matches!(f, Fund::Startseite { .. })),
            "Schema, www. und Schlussschrägstrich unterscheiden nichts"
        );
    }

    #[test]
    fn themen_kommen_nicht_doppelt() {
        let mut b = deuten(
            &Quelle::Adresse("https://github.com/o/r".into()),
            &ScanOptionen::default(),
        )
        .unwrap();
        anreichern(&mut b, &steckbrief());
        anreichern(&mut b, &steckbrief());
        assert_eq!(b.vorschlag.unwrap().tags.len(), 2);
    }

    #[test]
    fn archiviert_steht_im_befund() {
        let mut b = deuten(
            &Quelle::Adresse("https://github.com/o/r".into()),
            &ScanOptionen::default(),
        )
        .unwrap();
        assert!(!b.archiviert);
        anreichern(
            &mut b,
            &crate::forge::Steckbrief {
                archiviert: true,
                ..steckbrief()
            },
        );
        assert!(b.archiviert, "sonst wartet jemand auf Bewegung");
    }

    #[test]
    fn ordner_mit_marker_wird_nicht_noch_einmal_angelegt() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path();
        let id = Ulid::new();
        detect::marker_schreiben(dir, id).unwrap();

        let b = deuten(&Quelle::Ordner(dir.to_path_buf()), &ScanOptionen::default()).unwrap();
        assert!(
            b.vorschlag.is_none(),
            "Für einen bekannten Ordner gibt es nichts vorzuschlagen"
        );
        assert!(matches!(b.funde.as_slice(), [Fund::SchonBekannt { id: g, .. }] if *g == id));
    }

    #[test]
    fn remote_steht_im_befund_statt_als_frage() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path();
        fs::write(dir.join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        git(dir, &["init", "-q"]);
        git(
            dir,
            &["remote", "add", "origin", "https://github.com/o/r.git"],
        );

        let b = deuten(&Quelle::Ordner(dir.to_path_buf()), &ScanOptionen::default()).unwrap();
        let remote = b.funde.iter().find_map(|f| match f {
            Fund::Remote { url, dienst } => Some((url.clone(), dienst.clone())),
            _ => None,
        });
        let (url, dienst) = remote.expect("Remote muss im Befund stehen");
        assert!(url.contains("github.com/o/r"));
        assert_eq!(dienst.as_deref(), Some("GitHub"));
        assert_eq!(b.vorschlag.unwrap().vorlage, Vorlage::Software);
    }

    #[test]
    fn der_gewaehlte_ordner_selbst_wird_gedeutet() {
        // `detect::scan` beginnt eine Ebene tiefer – der wahrscheinlichste Treffer wäre
        // damit genau der, den man angeklickt hat.
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("Hochbeet");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("README.md"),
            "# Hochbeet\n\nSüdseite, zwei Kisten.\n",
        )
        .unwrap();

        let b = deuten(&Quelle::Ordner(dir.clone()), &ScanOptionen::default()).unwrap();
        let v = b.vorschlag.expect("Vorschlag");
        assert_eq!(v.titel, "Hochbeet");
        assert_eq!(v.kurs.as_deref(), Some("Südseite, zwei Kisten."));
    }

    #[test]
    fn unterprojekte_werden_gefunden_und_gezaehlt() {
        let t = tempfile::tempdir().unwrap();
        for name in ["eins", "zwei", "drei"] {
            let d = t.path().join(name);
            fs::create_dir_all(&d).unwrap();
            fs::write(d.join("Cargo.toml"), "[package]\n").unwrap();
        }
        let b = deuten(
            &Quelle::Ordner(t.path().to_path_buf()),
            &ScanOptionen::default(),
        )
        .unwrap();
        let unter = b
            .funde
            .iter()
            .filter(|f| matches!(f, Fund::Unterprojekt { .. }))
            .count();
        assert_eq!(unter, 3);
        assert!(!b.abgebrochen);
    }

    #[test]
    fn budget_bricht_ab_und_sagt_es() {
        let t = tempfile::tempdir().unwrap();
        for i in 0..40 {
            fs::create_dir_all(t.path().join(format!("o{i}"))).unwrap();
        }
        let opt = ScanOptionen {
            max_ordner: 5,
            ..Default::default()
        };
        let b = deuten(&Quelle::Ordner(t.path().to_path_buf()), &opt).unwrap();
        assert!(b.abgebrochen, "Abbruch muss im Befund stehen");
        assert!(b.angesehen <= 6);
    }

    #[test]
    fn adresse_braucht_keinen_ordner() {
        let b = deuten(
            &Quelle::Adresse("https://github.com/phish3144/lotse".into()),
            &ScanOptionen::default(),
        )
        .unwrap();
        let v = b.vorschlag.expect("Vorschlag");
        assert_eq!(v.titel, "lotse");
        assert_eq!(v.vorlage, Vorlage::Software);
        assert!(matches!(
            b.funde.as_slice(),
            [Fund::Remote { dienst, .. }] if dienst.as_deref() == Some("GitHub")
        ));
    }

    #[test]
    fn dokumente_im_ordner_werden_gefunden() {
        let t = tempfile::tempdir().unwrap();
        fs::write(t.path().join("angebot.pdf"), b"%PDF-1.4").unwrap();
        fs::write(t.path().join("notizen.md"), "hallo").unwrap();
        fs::write(t.path().join("bild.png"), b"x").unwrap();

        let b = deuten(
            &Quelle::Ordner(t.path().to_path_buf()),
            &ScanOptionen::default(),
        )
        .unwrap();
        let namen: Vec<_> = b
            .funde
            .iter()
            .filter_map(|f| match f {
                Fund::Dokument { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect();
        assert!(namen.contains(&"angebot.pdf"));
        assert!(namen.contains(&"notizen.md"));
        assert!(!namen.contains(&"bild.png"), "Bilder sind nicht deutbar");
    }
}
