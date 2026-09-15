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
#[derive(Debug, Clone)]
pub enum Quelle {
    Ordner(PathBuf),
    Adresse(String),
    Dateien(Vec<PathBuf>),
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
}

/// Was Lotse für Titel, Kurs und Vorlage vorschlägt. Alles änderbar – es ist ein
/// Vorschlag, kein Befehl.
#[derive(Debug, Clone, Serialize)]
pub struct Vorschlag {
    pub titel: String,
    pub kurs: Option<String>,
    pub vorlage: Vorlage,
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
}

pub fn deuten(quelle: &Quelle, opt: &ScanOptionen) -> Result<Befund> {
    match quelle {
        Quelle::Ordner(p) => ordner(p, opt),
        Quelle::Adresse(a) => adresse(a),
        Quelle::Dateien(d) => dateien(d),
    }
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
    });

    Ok(Befund {
        quelle,
        vorschlag,
        funde,
        angesehen: unten.angesehen,
        abgebrochen: unten.abgebrochen,
        weitere_dokumente: weitere,
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
        }),
        funde: vec![Fund::Remote {
            url: a.to_string(),
            dienst,
        }],
        angesehen: 0,
        abgebrochen: false,
        weitere_dokumente: 0,
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
        }),
        funde,
        angesehen: 0,
        abgebrochen: false,
        weitere_dokumente: pfade.len().saturating_sub(MAX_DOKUMENTE),
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
