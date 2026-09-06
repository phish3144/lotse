//! Sync-Umschläge und Konfliktregel (siehe `docs/SYNC_PROTOCOL.md`).
//!
//! Der Dienst sieht nur diese Umschläge. Alles darin außer `id`, `kind`, `hlc`,
//! `device_id`, `deleted` und `format_version` ist Ciphertext. Der HTTP-Transport selbst
//! liegt außerhalb dieses Moduls (Desktop: `reqwest`, Browser: `fetch`); hier stehen die
//! Typen, die Versiegelung und die Regel, wer gewinnt.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use ulid::Ulid;

use crate::crypto::{self, b64_bytes, Key32, Sealed};
use crate::{Error, Result, FORMAT_VERSION};

#[cfg(feature = "native")]
pub mod client;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecordKind {
    Project,
    Note,
    Reference,
    VaultEntry,
    Device,
    Settings,
}

impl RecordKind {
    pub const ALLE: [RecordKind; 6] = [
        RecordKind::Project,
        RecordKind::Note,
        RecordKind::Reference,
        RecordKind::VaultEntry,
        RecordKind::Device,
        RecordKind::Settings,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            RecordKind::Project => "project",
            RecordKind::Note => "note",
            RecordKind::Reference => "reference",
            RecordKind::VaultEntry => "vault_entry",
            RecordKind::Device => "device",
            RecordKind::Settings => "settings",
        }
    }

    pub fn parse(s: &str) -> Option<RecordKind> {
        RecordKind::ALLE.into_iter().find(|k| k.as_str() == s)
    }
}

/// Der Umschlag, wie er über die Leitung geht.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Umschlag {
    pub id: Ulid,
    pub kind: RecordKind,
    pub hlc: String,
    pub device_id: Ulid,
    pub deleted: bool,
    pub format_version: u16,
    #[serde(with = "b64_bytes")]
    pub nonce: Vec<u8>,
    #[serde(with = "b64_bytes")]
    pub ciphertext: Vec<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_seq: Option<u64>,
}

impl Umschlag {
    /// Versiegelt einen Klartext-Datensatz.
    pub fn seal<T: Serialize>(
        records_key: &Key32,
        kind: RecordKind,
        id: Ulid,
        hlc: String,
        device_id: Ulid,
        payload: &T,
    ) -> Result<Umschlag> {
        let plaintext = serde_json::to_vec(payload)?;
        let sealed = crypto::seal(
            records_key,
            &crypto::aad(kind.as_str(), &id.to_string()),
            &plaintext,
        )?;
        Ok(Umschlag {
            id,
            kind,
            hlc,
            device_id,
            deleted: false,
            format_version: FORMAT_VERSION,
            nonce: sealed.nonce,
            ciphertext: sealed.ciphertext,
            server_seq: None,
        })
    }

    /// Tombstone: Ciphertext leer, bleibt dauerhaft im Dienst.
    pub fn tombstone(kind: RecordKind, id: Ulid, hlc: String, device_id: Ulid) -> Umschlag {
        Umschlag {
            id,
            kind,
            hlc,
            device_id,
            deleted: true,
            format_version: FORMAT_VERSION,
            nonce: Vec::new(),
            ciphertext: Vec::new(),
            server_seq: None,
        }
    }

    /// Öffnet den Umschlag. Prüft Formatversion und AAD (verhindert Vertauschen von
    /// Datensätzen durch den Dienst).
    pub fn open<T: DeserializeOwned>(&self, records_key: &Key32) -> Result<T> {
        if self.deleted {
            return Err(Error::Invalid("Tombstone hat keinen Inhalt".into()));
        }
        if self.format_version != FORMAT_VERSION {
            return Err(Error::FormatVersion(self.format_version, FORMAT_VERSION));
        }
        let sealed = Sealed {
            nonce: self.nonce.clone(),
            ciphertext: self.ciphertext.clone(),
        };
        let aad = crypto::aad_versioned(
            self.format_version,
            self.kind.as_str(),
            &self.id.to_string(),
        );
        let bytes = crypto::open(records_key, &aad, &sealed)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Plausibilitätsprüfung, wie sie auch der Dienst vornimmt.
    pub fn validate(&self) -> Result<()> {
        if !crate::hlc::is_valid(&self.hlc) {
            return Err(Error::Invalid("HLC ungültig".into()));
        }
        if self.deleted {
            if !self.ciphertext.is_empty() || !self.nonce.is_empty() {
                return Err(Error::Invalid("Tombstone muss leer sein".into()));
            }
        } else {
            if self.nonce.len() != crypto::NONCE_LEN {
                return Err(Error::Invalid("Nonce muss 24 Byte lang sein".into()));
            }
            if self.ciphertext.len() > MAX_CIPHERTEXT {
                return Err(Error::Invalid("Datensatz zu groß".into()));
            }
        }
        Ok(())
    }
}

pub const MAX_CIPHERTEXT: usize = 256 * 1024;
pub const MAX_PUSH_BATCH: usize = 500;

/// Last-Writer-Wins: der lexikografisch größere HLC gewinnt. Bei Gleichstand (derselbe
/// Umschlag zweimal) gewinnt der bereits vorhandene, damit Push idempotent bleibt.
pub fn gewinnt(eingehend_hlc: &str, vorhanden_hlc: Option<&str>) -> bool {
    match vorhanden_hlc {
        None => true,
        Some(v) => eingehend_hlc > v,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushRequest {
    pub records: Vec<Umschlag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rejected {
    pub id: Ulid,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResponse {
    pub accepted: Vec<Ulid>,
    pub rejected: Vec<Rejected>,
    pub server_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullResponse {
    pub records: Vec<Umschlag>,
    pub next_seq: u64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub server_seq: u64,
    pub record_count: u64,
    pub blob_bytes: u64,
}

/// Was ein Client über seinen Sync-Stand persistiert.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncState {
    /// Höchste lokale Änderungsnummer, die erfolgreich gepusht wurde.
    pub last_pushed_local_seq: u64,
    /// Höchste `server_seq`, die per Pull verarbeitet wurde.
    pub last_server_seq: u64,
    pub last_sync_ms: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Art, Notiz, Quelle};

    #[test]
    fn seal_open_roundtrip() {
        let key = Key32::random().unwrap();
        let dev = Ulid::new();
        let n = Notiz::neu(Ulid::new(), Quelle::Cli, Art::Log, "Motor läuft", 1000);
        let hlc = crate::hlc::format(1000, 0, &dev);
        let u = Umschlag::seal(&key, RecordKind::Note, n.id, hlc, dev, &n).unwrap();
        u.validate().unwrap();
        let json = serde_json::to_string(&u).unwrap();
        assert!(!json.contains("Motor"));
        let back: Umschlag = serde_json::from_str(&json).unwrap();
        let n2: Notiz = back.open(&key).unwrap();
        assert_eq!(n, n2);
    }

    #[test]
    fn vertauschte_id_wird_erkannt() {
        let key = Key32::random().unwrap();
        let dev = Ulid::new();
        let n = Notiz::neu(Ulid::new(), Quelle::Cli, Art::Log, "x", 1);
        let mut u = Umschlag::seal(
            &key,
            RecordKind::Note,
            n.id,
            crate::hlc::format(1, 0, &dev),
            dev,
            &n,
        )
        .unwrap();
        u.id = Ulid::new();
        assert!(u.open::<Notiz>(&key).is_err());
        u.id = n.id;
        u.kind = RecordKind::Project;
        assert!(u.open::<Notiz>(&key).is_err());
    }

    #[test]
    fn tombstone_und_gewinnt() {
        let dev = Ulid::new();
        let t = Umschlag::tombstone(
            RecordKind::Note,
            Ulid::new(),
            crate::hlc::format(5, 0, &dev),
            dev,
        );
        t.validate().unwrap();
        assert!(t.open::<Notiz>(&Key32::random().unwrap()).is_err());
        let a = crate::hlc::format(5, 0, &dev);
        let b = crate::hlc::format(5, 1, &dev);
        assert!(gewinnt(&b, Some(&a)));
        assert!(!gewinnt(&a, Some(&b)));
        assert!(!gewinnt(&a, Some(&a)));
        assert!(gewinnt(&a, None));
    }
}
