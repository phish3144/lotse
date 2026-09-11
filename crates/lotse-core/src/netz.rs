//! Ein Agent für alle Abfragen nach außen, und eine Stelle, an der aus einem
//! Netzfehler deutscher Klartext wird.
//!
//! Zwei Einstellungen, die überall gelten sollen:
//!
//! 1. **TLS gegen den Wurzelspeicher des Systems.** Dieselbe Wahl, die Browser, `git`
//!    und `curl` auf demselben Rechner treffen. In Netzen mit TLS-Prüfung fiele Lotse
//!    sonst aus, während alles andere läuft (`THREAT_MODEL.md` 6a).
//! 2. **Fehlerantworten sind Antworten, keine Fehlerwerte.** Nur so lassen sich ihre
//!    Kopfzeilen lesen: bei GitHub steckt im 403 der Unterschied zwischen „keine
//!    Berechtigung" und „Kontingent erschöpft". Der Aufrufer prüft den Status selbst.
//!
//! Dieses Modul greift nie auf den Tresor zu; Zugangsdaten reicht der Aufrufer herein.

use std::time::Duration;

use crate::{Error, Result};

/// Ein Agent mit dem gemeinsamen Zuschnitt und eigener Zeitgrenze.
pub fn agent(zeitgrenze: Duration) -> ureq::Agent {
    let tls = ureq::tls::TlsConfig::builder()
        .root_certs(ureq::tls::RootCerts::PlatformVerifier)
        .build();
    ureq::Agent::config_builder()
        .timeout_global(Some(zeitgrenze))
        .user_agent(concat!("lotse/", env!("CARGO_PKG_VERSION")))
        .http_status_as_error(false)
        .tls_config(tls)
        .build()
        .into()
}

/// Übersetzt einen Transportfehler in etwas, das ein Mensch lesen kann. `wohin` ist der
/// Name der Gegenstelle, damit in der Meldung steht, wer nicht antwortet.
pub fn fehler(e: ureq::Error, wohin: &str) -> Error {
    match e {
        ureq::Error::Timeout(_) => {
            Error::Netz(format!("{wohin} hat nicht rechtzeitig geantwortet."))
        }
        ureq::Error::HostNotFound => Error::Netz(format!("{wohin} ist nicht auffindbar.")),
        ureq::Error::ConnectionFailed => {
            Error::Netz(format!("Keine Verbindung zu {wohin}. Ist das Netz da?"))
        }
        ureq::Error::StatusCode(s) => Error::Netz(format!("{wohin} antwortete {s}")),
        andere => Error::Netz(andere.to_string()),
    }
}

/// Gibt es das, was unter dieser Adresse liegt?
///
/// Bewusst großzügig: 401 und 403 heißen »da, aber nicht für dich« – das ist erreichbar.
/// Nur 404/410 und ein Verbindungsfehler heißen »nicht da«. Ein `GET` statt `HEAD`, weil
/// etliche Server `HEAD` gar nicht beantworten.
pub fn erreichbar(url: &str) -> Result<bool> {
    let a = agent(std::time::Duration::from_secs(10));
    match a.get(url).call() {
        Ok(r) => Ok(!matches!(r.status().as_u16(), 404 | 410)),
        Err(e) => Err(fehler(e, url)),
    }
}
