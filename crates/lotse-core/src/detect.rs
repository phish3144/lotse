//! Automatische Projekterkennung (Hafeneinfahrt).
//!
//! Durchsucht registrierte Wurzelordner nach Erkennungsmarken und liefert Kandidaten mit
//! vorgeschlagener Vorlage. Liest ausschließlich Verzeichniseinträge und Dateinamen, plus
//! die ersten Zeilen einer README. Keine Inhalte sonst. Importiert **nie** aus `vault`.

use std::fs;
use std::path::{Path, PathBuf};

use ulid::Ulid;
use walkdir::WalkDir;

use crate::model::{Kandidat, Vorlage};
use crate::Result;

/// Name der Marker-Datei, mit der Lotse einen Ordner nach Umbenennen wiedererkennt.
pub const MARKER: &str = ".lotse-projekt";

/// Ordner, in die nie hinabgestiegen wird.
pub const AUSSCHLUSS: &[&str] = &[
    "node_modules",
    "target",
    ".git",
    "build",
    "dist",
    "out",
    ".venv",
    "venv",
    "__pycache__",
    ".cache",
    ".next",
    ".idea",
    ".vscode",
    ".gradle",
    "Library",
    "AppData",
    ".Trash",
    "Pods",
    "DerivedData",
];

/// Dateien, deren Inhalt Lotse nie liest (auch nicht für die Suche).
pub const NIE_LESEN: &[&str] = &[
    ".env",
    "*.pem",
    "*.key",
    "id_rsa*",
    "id_ed25519*",
    "*.kdbx",
    "*.p12",
];

#[derive(Debug, Clone)]
pub struct ScanOptionen {
    pub max_tiefe: usize,
    pub ausschluss: Vec<String>,
}

impl Default for ScanOptionen {
    fn default() -> Self {
        ScanOptionen {
            max_tiefe: 4,
            ausschluss: AUSSCHLUSS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Sucht Kandidaten unter den Wurzelordnern. Ein erkannter Projektordner wird nicht
/// weiter hinabgestiegen (Unterordner eines Projekts sind keine eigenen Projekte).
pub fn scan(wurzeln: &[PathBuf], opt: &ScanOptionen) -> Result<Vec<Kandidat>> {
    let mut out = Vec::new();
    for wurzel in wurzeln {
        if !wurzel.is_dir() {
            continue;
        }
        let mut it = WalkDir::new(wurzel)
            .min_depth(1)
            .max_depth(opt.max_tiefe)
            .follow_links(false)
            .into_iter();
        while let Some(entry) = it.next() {
            let Ok(entry) = entry else { continue };
            if !entry.file_type().is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy();
            if name.starts_with('.') || opt.ausschluss.iter().any(|a| a == name.as_ref()) {
                it.skip_current_dir();
                continue;
            }
            if let Some(k) = erkenne(entry.path()) {
                out.push(k);
                it.skip_current_dir();
            }
        }
    }
    out.sort_by(|a, b| a.pfad.cmp(&b.pfad));
    Ok(out)
}

/// Prüft einen einzelnen Ordner auf Erkennungsmarken.
pub fn erkenne(dir: &Path) -> Option<Kandidat> {
    let Ok(rd) = fs::read_dir(dir) else {
        return None;
    };
    let mut namen: Vec<String> = Vec::new();
    let mut hat_git = false;
    for e in rd.flatten() {
        let n = e.file_name().to_string_lossy().to_string();
        if n == ".git" {
            hat_git = true;
        }
        namen.push(n);
    }
    let lower: Vec<String> = namen.iter().map(|n| n.to_lowercase()).collect();
    let hat = |exact: &str| lower.iter().any(|n| n == exact);
    let endet = |ext: &str| lower.iter().filter(|n| n.ends_with(ext)).count();
    let beginnt = |prefix: &str| lower.iter().any(|n| n.starts_with(prefix));

    let mut marken: Vec<String> = Vec::new();
    let mut merke = |m: &str| marken.push(m.to_string());

    // Reihenfolge = Priorität. Spezifische Marken vor allgemeinen.
    let vorlage = if hat("platformio.ini")
        || endet(".kicad_pro") > 0
        || endet(".ino") > 0
        || endet(".scad") > 0
        || endet(".f3d") > 0
        || endet(".3mf") > 0
        || endet(".stl") >= 3
    {
        for m in [
            "platformio.ini",
            ".kicad_pro",
            ".ino",
            ".scad",
            ".f3d",
            ".3mf",
            ".stl",
        ] {
            if hat(m) || endet(m) > 0 {
                merke(m);
            }
        }
        Some(Vorlage::HardwareMaker)
    } else if hat("cargo.toml")
        || hat("package.json")
        || hat("pyproject.toml")
        || hat("go.mod")
        || endet(".sln") > 0
        || hat("cmakelists.txt")
        || hat("pom.xml")
        || hat("build.gradle")
        || hat("mix.exs")
        || hat("gemfile")
    {
        for m in [
            "cargo.toml",
            "package.json",
            "pyproject.toml",
            "go.mod",
            ".sln",
            "cmakelists.txt",
            "pom.xml",
            "build.gradle",
            "mix.exs",
            "gemfile",
        ] {
            if hat(m) || endet(m) > 0 {
                merke(m);
            }
        }
        Some(Vorlage::Software)
    } else if endet(".ipynb") > 0 || endet(".bib") > 0 || endet(".tex") > 0 {
        for m in [".ipynb", ".bib", ".tex"] {
            if endet(m) > 0 {
                merke(m);
            }
        }
        Some(Vorlage::LernenForschung)
    } else if hat_git {
        merke(".git");
        Some(Vorlage::Software)
    } else if endet(".als") > 0
        || endet(".logicx") > 0
        || endet(".flp") > 0
        || endet(".rpp") > 0
        || endet(".scriv") > 0
        || endet(".psd") > 0
        || endet(".afphoto") > 0
        || raw_bilder(&lower) >= 5
    {
        for m in [
            ".als", ".logicx", ".flp", ".rpp", ".scriv", ".psd", ".afphoto",
        ] {
            if endet(m) > 0 {
                merke(m);
            }
        }
        if raw_bilder(&lower) >= 5 {
            merke("raw-fotos");
        }
        Some(Vorlage::Kreativ)
    } else if pdfs_mit_jahr(&lower) >= 3 {
        merke("pdf-mit-jahreszahl");
        Some(Vorlage::FinanzenVerwaltung)
    } else if bilder(&lower) >= 8 && endet(".pdf") >= 1 {
        merke("fotos+pdf");
        Some(Vorlage::HausGarten)
    } else if beginnt("readme") {
        merke("readme");
        Some(Vorlage::Generisch)
    } else {
        None
    };
    let vorlage = vorlage?;

    let readme = namen
        .iter()
        .find(|n| n.to_lowercase().starts_with("readme"))
        .and_then(|n| fs::read_to_string(dir.join(n)).ok())
        .map(|s| s.chars().take(600).collect::<String>());

    Some(Kandidat {
        pfad: dir.to_string_lossy().to_string(),
        name: dir
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Projekt".into()),
        vorlage,
        marken,
        bekannte_id: marker_lesen(dir),
        hat_git,
        readme,
    })
}

fn raw_bilder(lower: &[String]) -> usize {
    lower
        .iter()
        .filter(|n| {
            [".cr2", ".cr3", ".nef", ".arw", ".dng", ".raf", ".orf"]
                .iter()
                .any(|e| n.ends_with(e))
        })
        .count()
}

fn bilder(lower: &[String]) -> usize {
    lower
        .iter()
        .filter(|n| {
            [".jpg", ".jpeg", ".png", ".heic", ".webp"]
                .iter()
                .any(|e| n.ends_with(e))
        })
        .count()
}

fn pdfs_mit_jahr(lower: &[String]) -> usize {
    lower
        .iter()
        .filter(|n| n.ends_with(".pdf") && enthaelt_jahr(n))
        .count()
}

fn enthaelt_jahr(s: &str) -> bool {
    let b = s.as_bytes();
    b.windows(4)
        .any(|w| w.iter().all(u8::is_ascii_digit) && (w.starts_with(b"19") || w.starts_with(b"20")))
}

/// Liest die Projekt-ID aus der Marker-Datei, falls vorhanden.
pub fn marker_lesen(dir: &Path) -> Option<Ulid> {
    let s = fs::read_to_string(dir.join(MARKER)).ok()?;
    Ulid::from_string(s.trim()).ok()
}

/// Schreibt die Marker-Datei. Inhalt ist nur die ULID, damit sie in jedem Repo harmlos ist.
pub fn marker_schreiben(dir: &Path, id: Ulid) -> Result<()> {
    fs::write(dir.join(MARKER), format!("{id}\n"))?;
    Ok(())
}

/// Kurzer Kurs-Vorschlag aus der README: erste nicht-leere Nicht-Überschrift-Zeile.
pub fn kurs_vorschlag(readme: &str) -> Option<String> {
    readme
        .lines()
        .map(str::trim)
        .find(|l| {
            !l.is_empty() && !l.starts_with('#') && !l.starts_with("![") && !l.starts_with("[![")
        })
        .map(|l| l.chars().take(200).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(dir: &Path, files: &[&str]) {
        fs::create_dir_all(dir).unwrap();
        for f in files {
            if let Some(d) = f.strip_suffix('/') {
                fs::create_dir_all(dir.join(d)).unwrap();
            } else {
                fs::write(dir.join(f), b"x").unwrap();
            }
        }
    }

    #[test]
    fn erkennt_vorlagen() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path();
        mk(
            &root.join("rust-tool"),
            &["Cargo.toml", "src/", ".git/", "README.md"],
        );
        mk(&root.join("lampe"), &["platformio.ini", ".git/"]);
        mk(
            &root.join("steuer"),
            &["Bescheid 2024.pdf", "Lohn 2025.pdf", "Spenden 2023.pdf"],
        );
        mk(&root.join("paper"), &["main.tex", "lit.bib"]);
        mk(&root.join("song"), &["idee.als"]);
        mk(&root.join("leer"), &["notizen.txt"]);
        mk(&root.join("node_modules").join("x"), &["package.json"]);
        mk(&root.join("rust-tool").join("sub"), &["package.json"]);
        fs::write(
            root.join("rust-tool").join("README.md"),
            "# Tool\n\nEin kleines Werkzeug.\n",
        )
        .unwrap();

        let ks = scan(&[root.to_path_buf()], &ScanOptionen::default()).unwrap();
        let by: std::collections::HashMap<String, &Kandidat> =
            ks.iter().map(|k| (k.name.clone(), k)).collect();
        assert_eq!(by["rust-tool"].vorlage, Vorlage::Software);
        assert!(by["rust-tool"].hat_git);
        assert_eq!(
            kurs_vorschlag(by["rust-tool"].readme.as_ref().unwrap()).as_deref(),
            Some("Ein kleines Werkzeug.")
        );
        assert_eq!(by["lampe"].vorlage, Vorlage::HardwareMaker);
        assert_eq!(by["steuer"].vorlage, Vorlage::FinanzenVerwaltung);
        assert_eq!(by["paper"].vorlage, Vorlage::LernenForschung);
        assert_eq!(by["song"].vorlage, Vorlage::Kreativ);
        assert!(!by.contains_key("leer"));
        assert!(!by.contains_key("x"));
        assert!(
            !by.contains_key("sub"),
            "Unterordner eines Projekts sind keine Projekte"
        );
    }

    #[test]
    fn marker_roundtrip() {
        let t = tempfile::tempdir().unwrap();
        let id = Ulid::new();
        marker_schreiben(t.path(), id).unwrap();
        assert_eq!(marker_lesen(t.path()), Some(id));
        mk(t.path(), &["Cargo.toml"]);
        assert_eq!(erkenne(t.path()).unwrap().bekannte_id, Some(id));
    }
}
