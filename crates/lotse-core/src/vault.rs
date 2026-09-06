//! Tresor: verschlüsselte Einträge mit zwei Stufen.
//!
//! * `ueberall` – Eintragsschlüssel (DEK) ist mit dem Tresor-Schlüssel gewrappt, der aus
//!   dem Account-Schlüssel abgeleitet wird. Lesbar auf jedem Gerät mit Master-Passwort.
//! * `nur_desktop` – DEK ist mit einem Schlüssel gewrappt, der aus Account-Schlüssel **und**
//!   Desktop-Schlüssel abgeleitet wird. Der Desktop-Schlüssel liegt nur im OS-Schlüsselbund
//!   der Desktops und ist weder aus dem Passwort ableitbar noch auf dem Server.
//!
//! Löschen ist Crypto-Shredding: der gewrappte DEK wird entfernt, die Werte bleiben
//! unlesbar, auch in alten Sync-Kopien.
//!
//! Dieses Modul wird von `detect`, `export::mirror`, `watcher`, `mcp` und `ai` **nie**
//! importiert (siehe `docs/THREAT_MODEL.md`, Abschnitt 6).

use serde::{Deserialize, Serialize};
use ulid::Ulid;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::crypto::{self, Key32, RecoveryCode, Sealed};
use crate::model::Stufe;
use crate::{Error, Result};

/// Desktop-Schlüssel. Wird beim Setup einmal angezeigt (gleiche Darstellung wie der
/// Wiederherstellungscode, aber 32 Byte) und in jedem Desktop-Schlüsselbund abgelegt.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct DesktopKey(Key32);

impl DesktopKey {
    pub fn generate() -> Result<DesktopKey> {
        Ok(DesktopKey(Key32::random()?))
    }

    pub fn from_key(k: Key32) -> DesktopKey {
        DesktopKey(k)
    }

    pub fn key(&self) -> &Key32 {
        &self.0
    }

    /// Menschenlesbare Form zum Notieren in Proton Pass: zwei Wiederherstellungscode-Blöcke.
    pub fn display(&self) -> Zeroizing<String> {
        let b = self.0.as_bytes();
        let a = RecoveryCode::from_bytes(b[..16].try_into().unwrap()).display();
        let c = RecoveryCode::from_bytes(b[16..].try_into().unwrap()).display();
        Zeroizing::new(format!("{}-{}", a.as_str(), c.as_str()))
    }

    pub fn parse(s: &str) -> Result<DesktopKey> {
        let cleaned: String = s.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        if cleaned.len() != 52 {
            return Err(Error::Invalid(
                "Desktop-Schlüssel hat die falsche Länge".into(),
            ));
        }
        let a = RecoveryCode::parse(&cleaned[..26])?;
        let c = RecoveryCode::parse(&cleaned[26..])?;
        let mut bytes = [0u8; 32];
        bytes[..16].copy_from_slice(a.bytes());
        bytes[16..].copy_from_slice(c.bytes());
        Ok(DesktopKey(Key32::from_bytes(bytes)))
    }
}

/// Die Tresor-Schlüssel eines entsperrten Kontos.
pub struct VaultKeys {
    ueberall: Key32,
    nur_desktop: Option<Key32>,
}

impl VaultKeys {
    /// Ohne Desktop-Schlüssel (Browser, fremder Rechner).
    pub fn from_account_key(account_key: &Key32) -> VaultKeys {
        VaultKeys {
            ueberall: account_key.subkey(crypto::info::VAULT),
            nur_desktop: None,
        }
    }

    /// Mit Desktop-Schlüssel (eigener Desktop).
    pub fn with_desktop_key(account_key: &Key32, desktop: &DesktopKey) -> VaultKeys {
        let mut ikm = Zeroizing::new([0u8; 64]);
        ikm[..32].copy_from_slice(account_key.as_bytes());
        ikm[32..].copy_from_slice(desktop.key().as_bytes());
        let combined = Key32::from_bytes(hkdf_extract(&ikm[..]));
        VaultKeys {
            ueberall: account_key.subkey(crypto::info::VAULT),
            nur_desktop: Some(combined.subkey(crypto::info::VAULT_DESKTOP)),
        }
    }

    pub fn kann_nur_desktop(&self) -> bool {
        self.nur_desktop.is_some()
    }

    fn kek(&self, stufe: Stufe) -> Result<&Key32> {
        match stufe {
            Stufe::Ueberall => Ok(&self.ueberall),
            Stufe::NurDesktop => self.nur_desktop.as_ref().ok_or(Error::DesktopKeyMissing),
        }
    }
}

/// HKDF-Extract über 64 Byte Eingabe zu 32 Byte, ohne zweiten Salt.
fn hkdf_extract(ikm: &[u8]) -> [u8; 32] {
    use hkdf::Hkdf;
    use sha2::Sha256;
    let (prk, _) = Hkdf::<Sha256>::extract(None, ikm);
    let mut out = [0u8; 32];
    out.copy_from_slice(&prk);
    out
}

/// Ein Feld eines Tresor-Eintrags: Name im Klartext, Wert verschlüsselt mit dem DEK.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TresorFeld {
    pub name: String,
    pub wert: Sealed,
}

/// Tresor-Eintrag. Titel, Projektzuordnung und Stufe sind im lokalen Index Klartext,
/// im Sync-Datensatz liegt der gesamte Eintrag innerhalb des Ciphertexts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TresorEintrag {
    pub id: Ulid,
    pub titel: String,
    #[serde(default)]
    pub projekt_ids: Vec<Ulid>,
    pub stufe: Stufe,
    /// Gewrappter DEK. `None` nach Crypto-Shredding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrapped_dek: Option<Sealed>,
    #[serde(default)]
    pub felder: Vec<TresorFeld>,
    pub angelegt: i64,
    pub geaendert: i64,
}

fn aad_dek(id: &Ulid, stufe: Stufe) -> Vec<u8> {
    crypto::aad("vault_dek", &format!("{id}\u{1f}{}", stufe.as_str()))
}

fn aad_feld(id: &Ulid, name: &str) -> Vec<u8> {
    crypto::aad("vault_field", &format!("{id}\u{1f}{name}"))
}

impl TresorEintrag {
    /// Legt einen Eintrag an und verschlüsselt alle Felder.
    pub fn neu(
        keys: &VaultKeys,
        titel: impl Into<String>,
        projekt_ids: Vec<Ulid>,
        stufe: Stufe,
        felder: &[(&str, &str)],
        jetzt_ms: i64,
    ) -> Result<TresorEintrag> {
        let id = Ulid::new();
        let dek = Key32::random()?;
        let wrapped = crypto::wrap_key(keys.kek(stufe)?, &aad_dek(&id, stufe), &dek)?;
        let mut eintrag = TresorEintrag {
            id,
            titel: titel.into(),
            projekt_ids,
            stufe,
            wrapped_dek: Some(wrapped),
            felder: Vec::new(),
            angelegt: jetzt_ms,
            geaendert: jetzt_ms,
        };
        for (name, wert) in felder {
            let sealed = crypto::seal(&dek, &aad_feld(&id, name), wert.as_bytes())?;
            eintrag.felder.push(TresorFeld {
                name: name.to_string(),
                wert: sealed,
            });
        }
        Ok(eintrag)
    }

    fn dek(&self, keys: &VaultKeys) -> Result<Key32> {
        let wrapped = self.wrapped_dek.as_ref().ok_or_else(|| {
            Error::NotFound("Eintrag wurde gelöscht (Schlüssel vernichtet)".into())
        })?;
        crypto::unwrap_key(
            keys.kek(self.stufe)?,
            &aad_dek(&self.id, self.stufe),
            wrapped,
        )
    }

    /// Entschlüsselt ein einzelnes Feld. Nur das angeklickte Feld, nie alle auf einmal.
    pub fn feld_lesen(&self, keys: &VaultKeys, name: &str) -> Result<Zeroizing<String>> {
        let feld = self
            .felder
            .iter()
            .find(|f| f.name == name)
            .ok_or_else(|| Error::NotFound(format!("Feld {name}")))?;
        let dek = self.dek(keys)?;
        let bytes = crypto::open(&dek, &aad_feld(&self.id, name), &feld.wert)?;
        String::from_utf8(bytes.to_vec())
            .map(Zeroizing::new)
            .map_err(|_| Error::Decrypt)
    }

    /// Setzt oder ersetzt ein Feld.
    pub fn feld_setzen(
        &mut self,
        keys: &VaultKeys,
        name: &str,
        wert: &str,
        jetzt_ms: i64,
    ) -> Result<()> {
        let dek = self.dek(keys)?;
        let sealed = crypto::seal(&dek, &aad_feld(&self.id, name), wert.as_bytes())?;
        match self.felder.iter_mut().find(|f| f.name == name) {
            Some(f) => f.wert = sealed,
            None => self.felder.push(TresorFeld {
                name: name.to_string(),
                wert: sealed,
            }),
        }
        self.geaendert = jetzt_ms;
        Ok(())
    }

    /// Crypto-Shredding: der DEK wird vernichtet, die Werte bleiben als unlesbarer Ballast.
    pub fn schreddern(&mut self, jetzt_ms: i64) {
        self.wrapped_dek = None;
        self.felder.clear();
        self.geaendert = jetzt_ms;
    }

    pub fn ist_geschreddert(&self) -> bool {
        self.wrapped_dek.is_none()
    }

    /// Kann dieser Eintrag mit den vorhandenen Schlüsseln gelesen werden?
    pub fn lesbar_mit(&self, keys: &VaultKeys) -> bool {
        !self.ist_geschreddert() && keys.kek(self.stufe).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> (Key32, DesktopKey) {
        (Key32::random().unwrap(), DesktopKey::generate().unwrap())
    }

    #[test]
    fn ueberall_ist_ohne_desktop_key_lesbar() {
        let (ak, _) = keys();
        let vk = VaultKeys::from_account_key(&ak);
        let e = TresorEintrag::neu(
            &vk,
            "Fritzbox",
            vec![],
            Stufe::Ueberall,
            &[("passwort", "geheim")],
            1,
        )
        .unwrap();
        assert_eq!(e.feld_lesen(&vk, "passwort").unwrap().as_str(), "geheim");
    }

    #[test]
    fn nur_desktop_braucht_desktop_key() {
        let (ak, dk) = keys();
        let voll = VaultKeys::with_desktop_key(&ak, &dk);
        let e = TresorEintrag::neu(
            &voll,
            "Bank",
            vec![],
            Stufe::NurDesktop,
            &[("pin", "1234")],
            1,
        )
        .unwrap();
        assert_eq!(e.feld_lesen(&voll, "pin").unwrap().as_str(), "1234");

        let browser = VaultKeys::from_account_key(&ak);
        assert!(matches!(
            e.feld_lesen(&browser, "pin"),
            Err(Error::DesktopKeyMissing)
        ));
        assert!(!e.lesbar_mit(&browser));

        let falscher_dk = DesktopKey::generate().unwrap();
        let falsch = VaultKeys::with_desktop_key(&ak, &falscher_dk);
        assert!(matches!(e.feld_lesen(&falsch, "pin"), Err(Error::Decrypt)));
    }

    #[test]
    fn schreddern_macht_unlesbar() {
        let (ak, _) = keys();
        let vk = VaultKeys::from_account_key(&ak);
        let mut e =
            TresorEintrag::neu(&vk, "x", vec![], Stufe::Ueberall, &[("a", "b")], 1).unwrap();
        e.schreddern(2);
        assert!(e.ist_geschreddert());
        assert!(e.feld_lesen(&vk, "a").is_err());
    }

    #[test]
    fn feld_setzen_und_aad_bindung() {
        let (ak, _) = keys();
        let vk = VaultKeys::from_account_key(&ak);
        let mut e = TresorEintrag::neu(&vk, "x", vec![], Stufe::Ueberall, &[], 1).unwrap();
        e.feld_setzen(&vk, "user", "anna", 2).unwrap();
        e.feld_setzen(&vk, "user", "berta", 3).unwrap();
        assert_eq!(e.feld_lesen(&vk, "user").unwrap().as_str(), "berta");
        assert_eq!(e.felder.len(), 1);
        // Ein Feld unter fremdem Namen einschleusen scheitert an der AAD.
        e.felder[0].name = "pass".into();
        assert!(e.feld_lesen(&vk, "pass").is_err());
    }

    #[test]
    fn desktop_key_display_parse() {
        let dk = DesktopKey::generate().unwrap();
        let shown = dk.display();
        let parsed = DesktopKey::parse(&shown).unwrap();
        assert_eq!(parsed.key().as_bytes(), dk.key().as_bytes());
    }

    #[test]
    fn eintrag_serialisiert_ohne_klartext() {
        let (ak, _) = keys();
        let vk = VaultKeys::from_account_key(&ak);
        let e = TresorEintrag::neu(
            &vk,
            "t",
            vec![],
            Stufe::Ueberall,
            &[("k", "SEHR_GEHEIM")],
            1,
        )
        .unwrap();
        let json = serde_json::to_string(&e).unwrap();
        assert!(!json.contains("SEHR_GEHEIM"));
    }
}
