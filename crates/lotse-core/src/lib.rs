//! Lotse-Kern.
//!
//! Enthält alles, was zehn Jahre halten muss: Datenmodell, Kryptografie, Sync-Umschläge,
//! verschlüsselter lokaler Speicher, Projekterkennung und Export. Der Kern läuft nativ
//! (Desktop, CLI) und – ohne das Feature `native` – als WebAssembly im Browser.
//!
//! Modulgrenzen, die das Bedrohungsmodell verlangt (`docs/THREAT_MODEL.md`, Abschnitt 6):
//! `detect`, `export::mirror` und spätere Module `watcher`, `mcp`, `ai` importieren
//! niemals aus `vault`. Der Tresor wird ausschließlich von der Oberfläche und der CLI
//! angesprochen.

pub mod brief;
pub mod crypto;
pub mod error;
pub mod hlc;
pub mod model;
pub mod sync;
pub mod vault;

#[cfg(feature = "native")]
pub mod detect;
#[cfg(feature = "native")]
pub mod export;
#[cfg(feature = "native")]
pub mod git;
#[cfg(feature = "native")]
pub mod store;

pub use error::{Error, Result};

/// Version der Kryptografie-Komposition. Wird bei jeder Änderung der Schlüsselhierarchie
/// erhöht und in jedem Ciphertext als Additional Authenticated Data mitgeführt.
pub const FORMAT_VERSION: u16 = 1;

/// Version des Datenschemas im lokalen Speicher und in den Klartext-Datensätzen.
pub const SCHEMA_VERSION: u32 = 1;

/// Aktuelle Zeit in Unix-Millisekunden. Im Browser wird diese Funktion später über
/// `js_sys::Date::now` bereitgestellt; nativ über die Systemuhr.
pub fn now_ms() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        // Platzhalter bis zur WASM-Anbindung; die Uhr wird dort von außen gesetzt.
        0
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}
