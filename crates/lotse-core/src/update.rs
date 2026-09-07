//! Nachsehen, ob es eine neuere Version gibt – mehr nicht.
//!
//! Lotse lädt nichts herunter und führt nichts aus. Es fragt die
//! Veröffentlichungen auf GitHub ab, vergleicht die Versionsnummer mit der
//! laufenden und nennt die passende Datei. Herunterladen und installieren tut der
//! Mensch, im Browser.
//!
//! Der Grund steht in `THREAT_MODEL.md`: die Installer sind bislang unsigniert. Ein
//! Programm, das sich selbst mit unsignierten Binärdaten überschreibt, wäre ein
//! bequemer Weg für jeden, der die Verbindung oder das Konto kontrolliert. Sobald es
//! signierte Bauten gibt, kann daraus ein echter Updater werden; bis dahin ist der
//! ehrliche Umfang: Bescheid sagen.

use serde::Deserialize;

use crate::forge::{Anbieter, RepoZeiger};
use crate::Result;

/// Eine Veröffentlichung, auf das Nötige reduziert.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Veroeffentlichung {
    /// Ohne führendes `v`: `0.2.0`.
    pub version: String,
    /// Seite mit allen Dateien und dem Änderungstext.
    pub seite: String,
    pub vorab: bool,
    pub dateien: Vec<Datei>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Datei {
    pub name: String,
    pub url: String,
    pub bytes: u64,
}

/// Vergleicht zwei Versionen nach dem Muster `haupt.neben.klein[-vorab]`.
/// `1.2.0` ist neuer als `1.2.0-rc1`, `0.10.0` neuer als `0.9.9`.
pub fn neuer_als(kandidat: &str, laufend: &str) -> bool {
    ordnung(kandidat) > ordnung(laufend)
}

fn ordnung(v: &str) -> (u64, u64, u64, u8, String) {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let (kern, vorab) = match v.split_once('-') {
        Some((k, s)) => (k, Some(s)),
        None => (v, None),
    };
    let mut teile = kern.split('.').map(|t| t.parse::<u64>().unwrap_or(0));
    (
        teile.next().unwrap_or(0),
        teile.next().unwrap_or(0),
        teile.next().unwrap_or(0),
        // Eine Vorabversion kommt vor der fertigen mit derselben Nummer.
        u8::from(vorab.is_none()),
        vorab.unwrap_or("").to_string(),
    )
}

/// Sucht die Datei, die auf dieses System passt. `os` und `arch` sind die Werte von
/// `std::env::consts`. `None` heißt: für dieses System liegt nichts bereit – dann
/// bleibt der Weg über die Veröffentlichungsseite.
pub fn passende_datei<'a>(v: &'a Veroeffentlichung, os: &str, arch: &str) -> Option<&'a Datei> {
    let brauchbar = |d: &&Datei| {
        let n = d.name.to_ascii_lowercase();
        // Prüfsummen, Signaturen, die Updater-Pakete und die Kommandozeile sind keine
        // Installer für die App.
        !n.ends_with(".sha256")
            && !n.ends_with(".sig")
            && !n.ends_with(".app.tar.gz")
            && !n.starts_with("lotse-cli-")
    };
    // Reihenfolge der Endungen je System: das Übliche zuerst.
    let endungen: &[&str] = match (os, arch) {
        ("windows", _) => &["-setup.exe", ".msi"],
        ("macos", "aarch64") => &["aarch64.dmg", ".dmg"],
        ("macos", _) => &["x64.dmg", ".dmg"],
        ("linux", _) => &[".appimage", ".deb", ".rpm"],
        _ => &[],
    };
    for endung in endungen {
        if let Some(d) = v
            .dateien
            .iter()
            .filter(brauchbar)
            .find(|d| d.name.to_ascii_lowercase().ends_with(endung))
        {
            return Some(d);
        }
    }
    None
}

// --------------------------------------------------------------- Abfrage

#[derive(Deserialize)]
struct ReleaseWire {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    assets: Vec<AssetWire>,
}

#[derive(Deserialize)]
struct AssetWire {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
}

/// Wählt aus der Liste die neueste brauchbare Veröffentlichung. Entwürfe zählen nie;
/// Vorabversionen nur, wenn `mit_vorab` gesetzt ist. Getrennt vom Holen, damit genau
/// dieser Teil gegen echte Antworten prüfbar ist.
fn neueste_aus(releases: Vec<ReleaseWire>, mit_vorab: bool) -> Option<Veroeffentlichung> {
    releases
        .into_iter()
        .filter(|r| !r.draft && (mit_vorab || !r.prerelease))
        .map(|r| Veroeffentlichung {
            version: r.tag_name.trim().trim_start_matches(['v', 'V']).to_string(),
            seite: r.html_url,
            vorab: r.prerelease,
            dateien: r
                .assets
                .into_iter()
                .map(|a| Datei {
                    name: a.name,
                    url: a.browser_download_url,
                    bytes: a.size,
                })
                .collect(),
        })
        // Nicht auf die Reihenfolge von GitHub verlassen.
        .max_by(|a, b| ordnung(&a.version).cmp(&ordnung(&b.version)))
}

/// Fragt die Veröffentlichungen ab. Ohne Token, weil es um ein öffentliches Repo geht.
pub fn neueste(zeiger: &RepoZeiger, mit_vorab: bool) -> Result<Option<Veroeffentlichung>> {
    if zeiger.anbieter != Anbieter::GitHub {
        return Ok(None);
    }
    let a = crate::forge::agent();
    let url = format!(
        "https://api.github.com/repos/{}/{}/releases?per_page=10",
        zeiger.owner, zeiger.repo
    );
    let releases: Vec<ReleaseWire> = crate::forge::hole(&a, &url, Anbieter::GitHub, None)?;
    Ok(neueste_aus(releases, mit_vorab))
}

/// Das Repo, aus dem Lotse selbst kommt.
pub fn eigenes_repo() -> RepoZeiger {
    RepoZeiger {
        anbieter: Anbieter::GitHub,
        owner: "phish3144".into(),
        repo: "lotse".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn daten(name: &str) -> String {
        std::fs::read_to_string(format!("{}/tests/daten/{name}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("{name}: {e}"))
    }

    #[test]
    fn vergleicht_versionen_der_reihe_nach() {
        assert!(neuer_als("0.3.0", "0.2.0"));
        assert!(neuer_als("v0.3.0", "0.2.9"));
        assert!(neuer_als("0.10.0", "0.9.9"));
        assert!(neuer_als("1.0.0", "1.0.0-rc1"));
        assert!(!neuer_als("0.2.0", "0.2.0"));
        assert!(!neuer_als("0.1.9", "0.2.0"));
        assert!(!neuer_als("0.2.0-rc1", "0.2.0"));
        // Unsinn wird zu 0.0.0 und löst nie ein Update aus.
        assert!(!neuer_als("keine Version", "0.1.0"));
    }

    /// `gh_releases.json` ist die Antwort von api.github.com für dieses Repo, gekürzt
    /// auf die gelesenen Felder.
    #[test]
    fn liest_echte_veroeffentlichungen() {
        let wire: Vec<ReleaseWire> = serde_json::from_str(&daten("gh_releases.json")).unwrap();
        // v0.2.0 ist als Vorabversion markiert: ohne `mit_vorab` gibt es nichts.
        assert!(neueste_aus(
            serde_json::from_str(&daten("gh_releases.json")).unwrap(),
            false
        )
        .is_none());
        let v = neueste_aus(wire, true).expect("Vorabversion");
        assert_eq!(v.version, "0.2.0");
        assert!(v.vorab);
        assert!(v
            .seite
            .starts_with("https://github.com/phish3144/lotse/releases"));
        assert!(v.dateien.len() > 5);
    }

    #[test]
    fn findet_die_datei_fuer_das_system() {
        let v = neueste_aus(
            serde_json::from_str(&daten("gh_releases.json")).unwrap(),
            true,
        )
        .unwrap();
        let name = |os, arch| passende_datei(&v, os, arch).map(|d| d.name.clone());
        assert_eq!(
            name("windows", "x86_64").as_deref(),
            Some("Lotse_0.2.0_x64-setup.exe")
        );
        assert_eq!(
            name("macos", "aarch64").as_deref(),
            Some("Lotse_0.2.0_aarch64.dmg")
        );
        assert_eq!(
            name("macos", "x86_64").as_deref(),
            Some("Lotse_0.2.0_x64.dmg")
        );
        assert_eq!(
            name("linux", "x86_64").as_deref(),
            Some("Lotse_0.2.0_amd64.AppImage")
        );
        // Für ein System ohne Bau gibt es nichts – und keine falsche Datei.
        assert_eq!(name("freebsd", "x86_64"), None);
    }

    #[test]
    fn nimmt_weder_pruefsummen_noch_die_kommandozeile() {
        let v = neueste_aus(
            serde_json::from_str(&daten("gh_releases.json")).unwrap(),
            true,
        )
        .unwrap();
        for (os, arch) in [
            ("windows", "x86_64"),
            ("macos", "aarch64"),
            ("linux", "x86_64"),
        ] {
            let d = passende_datei(&v, os, arch).unwrap();
            assert!(!d.name.ends_with(".sha256"), "{}", d.name);
            assert!(!d.name.starts_with("lotse-cli-"), "{}", d.name);
            assert!(d.bytes > 0, "{}", d.name);
        }
    }

    #[test]
    fn nimmt_die_hoechste_version_nicht_die_erste() {
        let json = r#"[
          {"tag_name":"v0.1.0","html_url":"https://x.invalid/1","assets":[]},
          {"tag_name":"v0.3.0","html_url":"https://x.invalid/3","assets":[]},
          {"tag_name":"v0.2.0","html_url":"https://x.invalid/2","assets":[]}
        ]"#;
        let v = neueste_aus(serde_json::from_str(json).unwrap(), false).unwrap();
        assert_eq!(v.version, "0.3.0");
    }

    #[test]
    fn uebergeht_entwuerfe() {
        let json = r#"[
          {"tag_name":"v9.9.9","html_url":"https://x.invalid/9","draft":true,"assets":[]},
          {"tag_name":"v0.2.0","html_url":"https://x.invalid/2","assets":[]}
        ]"#;
        let v = neueste_aus(serde_json::from_str(json).unwrap(), false).unwrap();
        assert_eq!(v.version, "0.2.0");
    }
}
