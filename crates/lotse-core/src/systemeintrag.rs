//! Sich auf dem Linux-Desktop einrichten.
//!
//! Ein AppImage wird nicht installiert – es liegt da, wo der Browser es hingelegt hat.
//! Daraus folgen drei Ärgernisse auf einmal: kein Eintrag im Menü, ein Selbsttausch, der
//! davon abhängt, dass der Download-Ordner noch existiert, und ein Dateiname, der die
//! Version im Namen trägt und nach dem Tausch die falsche nennt.
//!
//! Dieses Modul räumt alle drei auf: es legt das AppImage an eine feste, beschreibbare
//! Stelle mit festem Namen und schreibt Desktop-Datei und Icon dorthin, wo jeder
//! Linux-Desktop sie sucht (XDG Base Directory).
//!
//! **Nur auf ausdrückliches Ja.** Ein Programm, das sich beim ersten Start unbefragt im
//! System verteilt, ist genau das, was man Lotse nicht zutrauen soll. Und alles, was es
//! anlegt, lässt sich mit [`entfernen`] wieder wegnehmen – sonst wäre »einrichten« eine
//! Einbahnstraße.
//!
//! Auf macOS und Windows gibt es hier nichts zu tun: dort installiert ein Installer, und
//! [`stand`] sagt das, statt etwas anzubieten, was nicht passt.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{Error, Result};

/// Dateiname des AppImage an seinem Platz. Ohne Version – sonst nennt die Datei nach dem
/// ersten Selbsttausch die alte Fassung.
pub const DATEINAME: &str = "Lotse.AppImage";
/// Name der Desktop-Datei und des Icons. Ohne Punkt-Präfix, damit `Icon=lotse` greift.
pub const EINTRAG: &str = "lotse";

/// Wie Lotse auf diesem System liegt. Alles, was die Oberfläche braucht, um zu
/// entscheiden, ob sie überhaupt etwas anbieten soll.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Stand {
    /// Läuft diese Fassung als AppImage? Sonst ist hier nichts zu tun: ein Installer hat
    /// den Eintrag schon angelegt, oder die Paketverwaltung.
    pub appimage: bool,
    /// Wo die laufende Datei liegt.
    pub pfad: Option<String>,
    /// Wo sie liegen sollte.
    pub ziel: Option<String>,
    /// Liegt sie schon dort?
    pub am_platz: bool,
    /// Gibt es den Eintrag im Menü?
    pub menueintrag: bool,
}

/// Was getan wurde. Die Oberfläche nennt es genau so – niemand soll raten müssen, was
/// ein Programm gerade in sein System geschrieben hat.
#[derive(Debug, Clone, Serialize)]
pub struct Bericht {
    /// Die Datei, die von jetzt an gilt.
    pub ziel: String,
    /// Die heruntergeladene Datei, falls kopiert wurde. Die kann weg – aber das
    /// entscheidet niemand anders als der Mensch, dem sie gehört.
    pub alter_pfad: Option<String>,
    pub desktop_datei: String,
    pub icon_datei: String,
    /// `false`, wenn die Datei schon am Platz lag und nur der Eintrag fehlte.
    pub kopiert: bool,
}

/// Pfad der laufenden AppImage-Datei. `None`, wenn das hier kein AppImage ist.
///
/// `APPIMAGE` setzt der AppImage-Starter selbst; `std::env::current_exe` zeigt dagegen in
/// den entpackten Einhängepunkt unter `/tmp` und wäre wertlos.
pub fn appimage_pfad() -> Option<PathBuf> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let p = PathBuf::from(std::env::var_os("APPIMAGE")?);
    p.is_file().then_some(p)
}

/// `$XDG_DATA_HOME`, sonst `~/.local/share`.
pub fn daten_wurzel() -> Option<PathBuf> {
    if let Some(x) = std::env::var_os("XDG_DATA_HOME").filter(|v| !v.is_empty()) {
        return Some(PathBuf::from(x));
    }
    std::env::var_os("HOME")
        .filter(|v| !v.is_empty())
        .map(|h| PathBuf::from(h).join(".local").join("share"))
}

/// Wohin das AppImage gehört.
pub fn ziel_pfad(wurzel: &Path) -> PathBuf {
    wurzel.join("lotse").join(DATEINAME)
}

fn desktop_pfad(wurzel: &Path) -> PathBuf {
    wurzel
        .join("applications")
        .join(format!("{EINTRAG}.desktop"))
}

fn icon_pfad(wurzel: &Path) -> PathBuf {
    wurzel
        .join("icons")
        .join("hicolor")
        .join("128x128")
        .join("apps")
        .join(format!("{EINTRAG}.png"))
}

pub fn stand() -> Stand {
    let Some(pfad) = appimage_pfad() else {
        return Stand::default();
    };
    let Some(wurzel) = daten_wurzel() else {
        return Stand {
            appimage: true,
            pfad: Some(pfad.to_string_lossy().to_string()),
            ..Stand::default()
        };
    };
    let ziel = ziel_pfad(&wurzel);
    Stand {
        appimage: true,
        am_platz: gleiche_datei(&pfad, &ziel),
        pfad: Some(pfad.to_string_lossy().to_string()),
        ziel: Some(ziel.to_string_lossy().to_string()),
        menueintrag: desktop_pfad(&wurzel).is_file(),
    }
}

/// Zwei Pfade, dieselbe Datei? Ein Vergleich der Zeichenketten würde einen Symlink oder
/// ein `./` im Pfad für einen Unterschied halten und die Datei über sich selbst kopieren.
fn gleiche_datei(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Legt das AppImage an seinen Platz und schreibt Desktop-Datei und Icon.
///
/// `icon_png` ist das Programmsymbol in 128×128 – die Hülle bringt es mit, damit hier
/// nicht im entpackten AppImage danach gesucht werden muss.
///
/// Die alte Datei bleibt liegen. Sie zu löschen wäre zwar aufgeräumter, aber Lotse löscht
/// nichts im Download-Ordner eines Menschen; der Bericht nennt sie, das genügt.
pub fn einrichten(icon_png: &[u8]) -> Result<Bericht> {
    let pfad = appimage_pfad().ok_or_else(|| {
        Error::Other(
            "Das hier läuft nicht als AppImage. Wo ein Installer installiert hat, gibt es \
             nichts einzurichten."
                .into(),
        )
    })?;
    let wurzel = daten_wurzel()
        .ok_or_else(|| Error::Other("Kein Nutzer-Datenverzeichnis gefunden ($HOME).".into()))?;

    let ziel = ziel_pfad(&wurzel);
    let kopiert = !gleiche_datei(&pfad, &ziel);
    if kopiert {
        if let Some(d) = ziel.parent() {
            std::fs::create_dir_all(d)?;
        }
        // Kopieren, nicht verschieben: über Dateisystemgrenzen scheitert ein Verschieben,
        // und die laufende Datei unter sich wegzuziehen ist keine gute Idee.
        std::fs::copy(&pfad, &ziel)?;
        ausfuehrbar_machen(&ziel)?;
    }

    let icon = icon_pfad(&wurzel);
    if let Some(d) = icon.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(&icon, icon_png)?;

    let desktop = desktop_pfad(&wurzel);
    if let Some(d) = desktop.parent() {
        std::fs::create_dir_all(d)?;
    }
    std::fs::write(&desktop, desktop_inhalt(&ziel))?;

    Ok(Bericht {
        ziel: ziel.to_string_lossy().to_string(),
        alter_pfad: kopiert.then(|| pfad.to_string_lossy().to_string()),
        desktop_datei: desktop.to_string_lossy().to_string(),
        icon_datei: icon.to_string_lossy().to_string(),
        kopiert,
    })
}

/// Nimmt Menüeintrag und Icon wieder weg. Das AppImage bleibt: es ist das Programm, das
/// gerade läuft, und wer es loswerden will, löscht eine Datei.
pub fn entfernen() -> Result<()> {
    let wurzel = daten_wurzel()
        .ok_or_else(|| Error::Other("Kein Nutzer-Datenverzeichnis gefunden ($HOME).".into()))?;
    for p in [desktop_pfad(&wurzel), icon_pfad(&wurzel)] {
        if p.exists() {
            std::fs::remove_file(p)?;
        }
    }
    Ok(())
}

/// Inhalt der Desktop-Datei. `Exec` wird zitiert: Datenverzeichnisse mit Leerzeichen im
/// Pfad sind selten, aber sie kommen vor, und ohne Anführungszeichen startet dort nichts.
fn desktop_inhalt(ziel: &Path) -> String {
    format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=Lotse\n\
         Comment=Logbuch für alle Vorhaben\n\
         Exec=\"{}\" %U\n\
         Icon={EINTRAG}\n\
         Terminal=false\n\
         Categories=Office;ProjectManagement;\n\
         StartupWMClass=Lotse\n",
        ziel.to_string_lossy()
    )
}

fn ausfuehrbar_machen(p: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut rechte = std::fs::metadata(p)?.permissions();
        rechte.set_mode(rechte.mode() | 0o111);
        std::fs::set_permissions(p, rechte)?;
    }
    #[cfg(not(unix))]
    let _ = p;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ziel_hat_keine_version_im_namen() {
        // Sonst nennt die Datei nach dem ersten Selbsttausch die alte Fassung.
        let z = ziel_pfad(Path::new("/h/.local/share"));
        assert_eq!(z, Path::new("/h/.local/share/lotse/Lotse.AppImage"));
        assert!(!DATEINAME.contains(char::is_numeric));
    }

    #[test]
    fn desktop_datei_zeigt_auf_den_festen_platz() {
        let inhalt = desktop_inhalt(Path::new("/h/.local/share/lotse/Lotse.AppImage"));
        assert!(inhalt.starts_with("[Desktop Entry]\n"));
        assert!(inhalt.contains("Exec=\"/h/.local/share/lotse/Lotse.AppImage\" %U\n"));
        assert!(inhalt.contains("Icon=lotse\n"));
        // Ohne diese Zeile klebt das Fenster unter GNOME an einem zweiten Symbol.
        assert!(inhalt.contains("StartupWMClass=Lotse\n"));
        assert!(inhalt.ends_with('\n'));
    }

    #[test]
    fn pfade_folgen_xdg() {
        let w = Path::new("/h/.local/share");
        assert_eq!(
            desktop_pfad(w),
            Path::new(&format!("{}/applications/lotse.desktop", w.display()))
        );
        assert_eq!(
            icon_pfad(w),
            Path::new(&format!(
                "{}/icons/hicolor/128x128/apps/lotse.png",
                w.display()
            ))
        );
    }

    #[test]
    fn gleiche_datei_erkennt_denselben_pfad_anders_geschrieben() {
        let t = tempfile::tempdir().unwrap();
        let a = t.path().join("x");
        std::fs::write(&a, b"x").unwrap();
        let umweg = t.path().join(".").join("x");
        assert!(gleiche_datei(&a, &umweg));
        assert!(!gleiche_datei(&a, &t.path().join("y")));
    }

    #[test]
    fn ohne_appimage_gibt_es_nichts_einzurichten() {
        // Auf allen Systemen wahr, wenn `APPIMAGE` nicht gesetzt ist – und auf macOS und
        // Windows auch dann, wenn jemand die Variable setzt.
        if std::env::var_os("APPIMAGE").is_none() {
            assert!(appimage_pfad().is_none());
            assert!(!stand().appimage);
            assert!(einrichten(b"").is_err());
        }
    }
}
