//! Fehlertypen des Kerns.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Kryptografie: {0}")]
    Crypto(&'static str),

    #[error("Entschlüsselung fehlgeschlagen (falscher Schlüssel oder manipulierte Daten)")]
    Decrypt,

    #[error("Schlüsselableitung fehlgeschlagen: {0}")]
    Kdf(String),

    #[error("Ungültige Eingabe: {0}")]
    Invalid(String),

    #[error("Nicht gefunden: {0}")]
    NotFound(String),

    #[error(
        "Der Desktop-Schlüssel fehlt; Einträge der Stufe »nur Desktop« sind hier nicht lesbar"
    )]
    DesktopKeyMissing,

    #[error("Formatversion {0} wird von diesem Kern (Version {1}) nicht unterstützt")]
    FormatVersion(u16, u16),

    #[error("Serialisierung: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Ungültige Base64-Kodierung")]
    Base64,

    #[cfg(feature = "native")]
    #[error("Speicher: {0}")]
    Store(#[from] rusqlite::Error),

    #[cfg(feature = "native")]
    #[error("Dateisystem: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;
