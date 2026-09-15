//! Konto auf diesem Gerät: Kontodatei, Einrichtung, Entsperren, Desktop-Schlüssel.
//!
//! Gemeinsame Grundlage für CLI und Desktop-App. Die Kontodatei (`konto.json`) enthält
//! nur den Konto-Header mit gewrappten Schlüsseln, nie Geheimnisse. Der Desktop-Schlüssel
//! liegt im OS-Schlüsselbund (Feature `keychain`), sonst muss ihn der Aufrufer liefern.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::crypto::{self, KdfParams, Key32, KontoHeader, RecoveryCode};
use crate::export::atomar_schreiben;
use crate::model::{Art, Geraet, Notiz, Projekt, Quelle, Vorlage};
use crate::store::Store;
use crate::vault::DesktopKey;
use crate::{now_ms, Error, Result};

pub const DATEI: &str = "konto.json";
pub const DB_DATEI: &str = "lotse.db";

/// Lokale Kontodatei. Enthält keine geheimen Schlüssel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Konto {
    pub header: KontoHeader,
    pub geraet_id: Ulid,
    pub geraet_name: String,
}

/// Name dieses Rechners, wie ihn das System kennt. Wird beim Einrichten als Gerätename
/// vorgeschlagen: »MacBook-Pro« sagt in der Geräteliste mehr als »Dieser Rechner«, und
/// niemand tippt ihn gern selbst ab.
///
/// macOS hängt an den Hostnamen oft `.local`; das gehört zum Netz, nicht zum Gerät.
/// Liefert das System nichts Brauchbares, bleibt es beim alten Platzhalter – ein leerer
/// Eintrag in der Geräteliste wäre schlimmer als ein unspezifischer.
pub fn rechnername() -> String {
    let roh = gethostname::gethostname().to_string_lossy().to_string();
    let sauber = roh.trim().trim_end_matches(".local").trim();
    if sauber.is_empty() {
        "Dieser Rechner".to_string()
    } else {
        sauber.to_string()
    }
}

pub fn pfad(home: &Path) -> PathBuf {
    home.join(DATEI)
}

pub fn existiert(home: &Path) -> bool {
    pfad(home).exists()
}

pub fn lesen(home: &Path) -> Result<Konto> {
    let text = std::fs::read_to_string(pfad(home))
        .map_err(|_| Error::NotFound(format!("Kein Konto in {}", home.display())))?;
    Ok(serde_json::from_str(&text)?)
}

pub fn schreiben(home: &Path, konto: &Konto) -> Result<()> {
    std::fs::create_dir_all(home)?;
    atomar_schreiben(&pfad(home), serde_json::to_string_pretty(konto)?.as_bytes())
}

/// Alles, was nach der Einrichtung einmalig gezeigt oder abgelegt werden muss.
pub struct Eingerichtet {
    pub konto: Konto,
    pub store: Store,
    pub account_key: Key32,
    pub auth_key: Key32,
    pub recovery_auth_key: Key32,
    pub recovery_code: RecoveryCode,
    pub desktop_key: DesktopKey,
}

/// Richtet Konto und Datenbank ein. Schreibt die Kontodatei, legt das Gerät und Lotse als
/// Projekt Nr. 1 an. Den Desktop-Schlüssel legt der Aufrufer im Schlüsselbund ab
/// (`desktop_key_speichern`) oder zeigt ihn an.
pub fn einrichten(
    home: &Path,
    passwort: &[u8],
    geraet_name: &str,
    kdf: KdfParams,
) -> Result<Eingerichtet> {
    if existiert(home) {
        return Err(Error::Invalid(format!(
            "In {} ist bereits ein Konto eingerichtet",
            home.display()
        )));
    }
    if passwort.len() < 12 {
        return Err(Error::Invalid(
            "Das Master-Passwort sollte mindestens 12 Zeichen haben".into(),
        ));
    }
    let neu = crypto::konto_einrichten(passwort, kdf)?;
    let desktop_key = DesktopKey::generate()?;
    let geraet_id = Ulid::new();
    let konto = Konto {
        header: neu.header,
        geraet_id,
        geraet_name: geraet_name.to_string(),
    };
    schreiben(home, &konto)?;
    let mut store = Store::open(&home.join(DB_DATEI), &neu.account_key, geraet_id)?;
    store.geraet_speichern(&Geraet {
        id: geraet_id,
        name: geraet_name.to_string(),
        plattform: std::env::consts::OS.to_string(),
        angelegt: now_ms(),
        zuletzt_sync: None,
    })?;
    // Lotse ist Projekt Nr. 1 in Lotse.
    let mut p = Projekt::neu("Lotse", Vorlage::Software, now_ms());
    p.kurs = "Das Logbuch für alle meine Vorhaben. Vier Kernprobleme: Wiedereinstieg, Faden halten, Rad nicht neu erfinden, Geheimnisse sicher aufbewahren.".into();
    store.projekt_speichern(&p)?;
    store.notiz_speichern(&Notiz::neu(
        p.id,
        Quelle::Mensch,
        Art::Log,
        "Konto eingerichtet. Lotse ist Projekt Nr. 1 in Lotse.",
        now_ms(),
    ))?;
    Ok(Eingerichtet {
        konto,
        store,
        account_key: neu.account_key,
        auth_key: neu.auth_key,
        recovery_auth_key: neu.recovery_auth_key,
        recovery_code: neu.recovery_code,
        desktop_key,
    })
}

/// Was beim Zurücksetzen tatsächlich verschwunden ist. Wird angezeigt, damit niemand
/// raten muss, ob der Schlüsselbund mit drankam.
#[derive(Debug, Clone, Serialize)]
pub struct Zuruecksetzung {
    pub dateien: usize,
    pub schluesselbund: bool,
}

/// Entfernt Lotse von diesem Gerät: Kontodatei, Datenbank und der Desktop-Schlüssel im
/// OS-Schlüsselbund.
///
/// Der Schlüsselbund ist der Grund, warum das eine eigene Funktion ist. Wer nur den
/// Datenordner löscht, lässt ein Geheimnis zurück, das danach niemand mehr zuordnen kann
/// – die Geräte-ID steht ja in der Datei, die man gerade gelöscht hat. Deshalb wird sie
/// hier zuerst gelesen.
///
/// Ein fehlender oder gesperrter Schlüsselbund bricht das Löschen nicht ab: ein Rechner
/// ohne Secret Service soll sich trotzdem zurücksetzen lassen. Das Feld `schluesselbund`
/// sagt, ob der Eintrag wirklich weg ist.
pub fn zuruecksetzen(home: &Path) -> Result<Zuruecksetzung> {
    let geraet_id = lesen(home).ok().map(|k| k.geraet_id);
    let schluesselbund = match geraet_id {
        Some(id) => cfg!(feature = "keychain") && desktop_key_loeschen(id).is_ok(),
        None => false,
    };

    let mut dateien = 0;
    for f in [DATEI, DB_DATEI, "lotse.db-wal", "lotse.db-shm"] {
        let p = home.join(f);
        if p.exists() {
            std::fs::remove_file(p)?;
            dateien += 1;
        }
    }
    Ok(Zuruecksetzung {
        dateien,
        schluesselbund,
    })
}

pub struct Entsperrt {
    pub konto: Konto,
    pub store: Store,
    pub account_key: Key32,
    pub auth_key: Key32,
}

/// Entsperrt Konto und Datenbank mit dem Master-Passwort.
pub fn entsperren(home: &Path, passwort: &[u8]) -> Result<Entsperrt> {
    let konto = lesen(home)?;
    let (account_key, auth_key) =
        crypto::konto_entsperren(&konto.header, passwort).map_err(|_| Error::Decrypt)?;
    let store = Store::open(&home.join(DB_DATEI), &account_key, konto.geraet_id)?;
    Ok(Entsperrt {
        konto,
        store,
        account_key,
        auth_key,
    })
}

/// Legt Kontodatei und Datenbank für ein Gerät an, das sich per Sync-Login angemeldet hat.
pub fn aus_login(
    home: &Path,
    header: KontoHeader,
    geraet_id: Ulid,
    geraet_name: &str,
    account_key: &Key32,
) -> Result<(Konto, Store)> {
    if existiert(home) {
        return Err(Error::Invalid(format!(
            "In {} ist bereits ein Konto eingerichtet",
            home.display()
        )));
    }
    let konto = Konto {
        header,
        geraet_id,
        geraet_name: geraet_name.to_string(),
    };
    schreiben(home, &konto)?;
    let mut store = Store::open(&home.join(DB_DATEI), account_key, geraet_id)?;
    store.geraet_speichern(&Geraet {
        id: geraet_id,
        name: geraet_name.to_string(),
        plattform: std::env::consts::OS.to_string(),
        angelegt: now_ms(),
        zuletzt_sync: None,
    })?;
    Ok((konto, store))
}

/// Standard-Datenordner: `$LOTSE_HOME`, sonst das Nutzer-Datenverzeichnis.
pub fn standard_home(fallback: Option<PathBuf>) -> Option<PathBuf> {
    std::env::var_os("LOTSE_HOME")
        .map(PathBuf::from)
        .or(fallback)
}

// ------------------------------------------------------------- Schlüsselbund

#[cfg(feature = "keychain")]
const KEYRING_DIENST: &str = "app.lotse.desktop";

/// Desktop-Schlüssel im OS-Schlüsselbund ablegen (Keychain, Credential Manager,
/// Secret Service). Ohne Feature `keychain` immer `Err`.
pub fn desktop_key_speichern(geraet_id: Ulid, key: &DesktopKey) -> Result<()> {
    #[cfg(feature = "keychain")]
    {
        let entry = keyring::Entry::new(KEYRING_DIENST, &format!("desktop-key/{geraet_id}"))
            .map_err(|e| Error::Other(format!("Schlüsselbund: {e}")))?;
        entry
            .set_password(&key.display())
            .map_err(|e| Error::Other(format!("Schlüsselbund: {e}")))?;
        Ok(())
    }
    #[cfg(not(feature = "keychain"))]
    {
        let _ = (geraet_id, key);
        Err(Error::Other(
            "Kein Schlüsselbund verfügbar (Feature `keychain` fehlt)".into(),
        ))
    }
}

/// Desktop-Schlüssel aus dem OS-Schlüsselbund. `Ok(None)`, wenn keiner abgelegt ist oder
/// kein Schlüsselbund verfügbar ist.
pub fn desktop_key_laden(geraet_id: Ulid) -> Result<Option<DesktopKey>> {
    #[cfg(feature = "keychain")]
    {
        let entry = match keyring::Entry::new(KEYRING_DIENST, &format!("desktop-key/{geraet_id}")) {
            Ok(e) => e,
            Err(_) => return Ok(None),
        };
        match entry.get_password() {
            Ok(s) => Ok(Some(DesktopKey::parse(&s)?)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Ok(None),
        }
    }
    #[cfg(not(feature = "keychain"))]
    {
        let _ = geraet_id;
        Ok(None)
    }
}

/// Entfernt den Desktop-Schlüssel aus dem OS-Schlüsselbund. Auch dann `Ok`, wenn gar
/// keiner abgelegt war – gewollt ist der Zustand, nicht die Handlung.
pub fn desktop_key_loeschen(geraet_id: Ulid) -> Result<()> {
    #[cfg(feature = "keychain")]
    {
        let entry = keyring::Entry::new(KEYRING_DIENST, &format!("desktop-key/{geraet_id}"))
            .map_err(|e| Error::Other(format!("Schlüsselbund: {e}")))?;
        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(Error::Other(format!("Schlüsselbund: {e}"))),
        }
    }
    #[cfg(not(feature = "keychain"))]
    {
        let _ = geraet_id;
        Ok(())
    }
}

/// Desktop-Schlüssel aus der Umgebung (`LOTSE_DESKTOP_KEY`), für Skripte und Rechner ohne
/// Schlüsselbund.
pub fn desktop_key_aus_umgebung() -> Result<Option<DesktopKey>> {
    match std::env::var("LOTSE_DESKTOP_KEY") {
        Ok(s) if !s.trim().is_empty() => Ok(Some(DesktopKey::parse(&s)?)),
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn einrichten_und_entsperren() {
        let t = tempfile::tempdir().unwrap();
        let home = t.path().join("daten");
        let e = einrichten(
            &home,
            b"korrekt batterie pferd",
            "Test",
            KdfParams::schnell_fuer_tests(),
        )
        .unwrap();
        assert!(existiert(&home));
        assert_eq!(e.store.projekte().unwrap()[0].titel, "Lotse");
        drop(e);
        assert!(matches!(entsperren(&home, b"falsch"), Err(Error::Decrypt)));
        let u = entsperren(&home, b"korrekt batterie pferd").unwrap();
        assert_eq!(u.konto.geraet_name, "Test");
        assert_eq!(u.store.projekte().unwrap().len(), 1);
        drop(u);
        assert!(einrichten(
            &home,
            b"korrekt batterie pferd",
            "Test",
            KdfParams::schnell_fuer_tests()
        )
        .is_err());
        let z = zuruecksetzen(&home).unwrap();
        assert!(z.dateien >= 2, "Kontodatei und Datenbank müssen fallen");
        assert!(!existiert(&home));
        assert!(!home.join(DB_DATEI).exists());
    }

    #[test]
    fn zuruecksetzen_ohne_konto_ist_kein_fehler() {
        // Wer zweimal zurücksetzt, hat nicht weniger Anspruch auf einen leeren Ordner
        // als beim ersten Mal.
        let t = tempfile::tempdir().unwrap();
        let z = zuruecksetzen(t.path()).unwrap();
        assert_eq!(z.dateien, 0);
        assert!(!z.schluesselbund);
    }

    #[test]
    fn rechnername_ist_nie_leer() {
        // Ein leerer Eintrag in der Geräteliste wäre schlimmer als ein unspezifischer.
        let n = rechnername();
        assert!(!n.trim().is_empty());
        assert!(!n.ends_with(".local"));
    }

    #[test]
    fn kurzes_passwort_abgelehnt() {
        let t = tempfile::tempdir().unwrap();
        assert!(einrichten(t.path(), b"kurz", "x", KdfParams::schnell_fuer_tests()).is_err());
        assert!(!existiert(t.path()));
    }
}
