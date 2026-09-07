//! Kryptografie-Komposition (siehe `docs/THREAT_MODEL.md`, Abschnitt 3).
//!
//! Ausschließlich geprüfte Bausteine aus RustCrypto: Argon2id, HKDF-SHA256,
//! XChaCha20-Poly1305. Dieses Modul erfindet keine Primitive, es setzt sie zusammen.
//! Jede Änderung an dieser Komposition erhöht `FORMAT_VERSION`.

use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use hkdf::Hkdf;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::{Error, Result, FORMAT_VERSION};

pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;
pub const SALT_LEN: usize = 16;

/// HKDF-`info`-Strings. Domain Separation zwischen allen Schlüsselverwendungen.
pub mod info {
    pub const AUTH: &[u8] = b"lotse/auth";
    pub const WRAP: &[u8] = b"lotse/wrap";
    pub const RECOVERY_AUTH: &[u8] = b"lotse/recovery-auth";
    pub const RECORDS: &[u8] = b"lotse/records";
    pub const LOCAL_DB: &[u8] = b"lotse/local-db";
    pub const VAULT: &[u8] = b"lotse/vault";
    pub const VAULT_DESKTOP: &[u8] = b"lotse/vault-desktop";
    pub const BLOBS: &[u8] = b"lotse/blobs";
}

/// 32-Byte-Schlüssel, der beim Verwerfen überschrieben wird.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Key32(pub(crate) [u8; KEY_LEN]);

impl Key32 {
    pub fn random() -> Result<Key32> {
        let mut k = [0u8; KEY_LEN];
        fill_random(&mut k)?;
        Ok(Key32(k))
    }

    pub fn from_bytes(b: [u8; KEY_LEN]) -> Key32 {
        Key32(b)
    }

    pub fn from_slice(b: &[u8]) -> Result<Key32> {
        let arr: [u8; KEY_LEN] = b
            .try_into()
            .map_err(|_| Error::Crypto("Schlüssel muss 32 Byte lang sein"))?;
        Ok(Key32(arr))
    }

    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }

    /// Leitet einen Unterschlüssel per HKDF-SHA256 ab.
    pub fn subkey(&self, info: &[u8]) -> Key32 {
        let hk = Hkdf::<Sha256>::new(None, &self.0);
        let mut out = [0u8; KEY_LEN];
        hk.expand(info, &mut out)
            .expect("32 Byte sind eine gültige HKDF-Länge");
        Key32(out)
    }

    /// Hex-Darstellung, z. B. für `PRAGMA key = "x'…'"` bei SQLCipher.
    pub fn to_hex(&self) -> Zeroizing<String> {
        let mut s = String::with_capacity(KEY_LEN * 2);
        for b in self.0 {
            s.push_str(&format!("{b:02x}"));
        }
        Zeroizing::new(s)
    }
}

impl std::fmt::Debug for Key32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Key32(…)")
    }
}

/// Argon2id-Parameter. Untergrenze m = 64 MiB, t = 3, p = 1. Der Desktop kalibriert
/// beim Setup nach oben; Browser und andere Geräte übernehmen die gespeicherten Werte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KdfParams {
    pub m_kib: u32,
    pub t: u32,
    pub p: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        KdfParams {
            m_kib: 64 * 1024,
            t: 3,
            p: 1,
        }
    }
}

impl KdfParams {
    /// Für Tests: schnell, aber deutlich unterhalb der Produktionsuntergrenze.
    pub fn schnell_fuer_tests() -> KdfParams {
        KdfParams {
            m_kib: 8 * 1024,
            t: 1,
            p: 1,
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.m_kib < 8 * 1024 || self.t == 0 || self.p == 0 {
            return Err(Error::Kdf(
                "Parameter unter der zulässigen Untergrenze".into(),
            ));
        }
        Ok(())
    }
}

pub fn fill_random(buf: &mut [u8]) -> Result<()> {
    getrandom::getrandom(buf).map_err(|_| Error::Crypto("Zufallsquelle nicht verfügbar"))
}

pub fn random_salt() -> Result<[u8; SALT_LEN]> {
    let mut s = [0u8; SALT_LEN];
    fill_random(&mut s)?;
    Ok(s)
}

/// Master-Passwort → Stretched Key (Argon2id).
pub fn derive_stretched(
    password: &[u8],
    salt: &[u8; SALT_LEN],
    params: &KdfParams,
) -> Result<Key32> {
    params.validate()?;
    let p = Params::new(params.m_kib, params.t, params.p, Some(KEY_LEN))
        .map_err(|e| Error::Kdf(e.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, p);
    let mut out = [0u8; KEY_LEN];
    argon
        .hash_password_into(password, salt, &mut out)
        .map_err(|e| Error::Kdf(e.to_string()))?;
    Ok(Key32(out))
}

/// Die beiden aus dem Passwort abgeleiteten Schlüssel.
pub struct PasswordKeys {
    /// Geht zum Server, wird dort nochmals gehasht.
    pub auth: Key32,
    /// Bleibt lokal, wrappt den Account-Schlüssel.
    pub wrap: Key32,
}

pub fn split_password_keys(stretched: &Key32) -> PasswordKeys {
    PasswordKeys {
        auth: stretched.subkey(info::AUTH),
        wrap: stretched.subkey(info::WRAP),
    }
}

/// Versiegelte Daten: Nonce + Ciphertext (inkl. Poly1305-Tag).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sealed {
    #[serde(with = "b64_bytes")]
    pub nonce: Vec<u8>,
    #[serde(with = "b64_bytes")]
    pub ciphertext: Vec<u8>,
}

impl Sealed {
    pub fn to_compact(&self) -> String {
        format!(
            "{}.{}",
            B64.encode(&self.nonce),
            B64.encode(&self.ciphertext)
        )
    }

    pub fn from_compact(s: &str) -> Result<Sealed> {
        let (n, c) = s.split_once('.').ok_or(Error::Base64)?;
        Ok(Sealed {
            nonce: B64.decode(n).map_err(|_| Error::Base64)?,
            ciphertext: B64.decode(c).map_err(|_| Error::Base64)?,
        })
    }
}

/// Additional Authenticated Data für einen Datensatz: `format_version ‖ kind ‖ id`,
/// getrennt durch `\x1f`.
pub fn aad(kind: &str, id: &str) -> Vec<u8> {
    aad_versioned(FORMAT_VERSION, kind, id)
}

pub fn aad_versioned(format_version: u16, kind: &str, id: &str) -> Vec<u8> {
    let mut v = Vec::with_capacity(8 + kind.len() + id.len());
    v.extend_from_slice(format_version.to_string().as_bytes());
    v.push(0x1f);
    v.extend_from_slice(kind.as_bytes());
    v.push(0x1f);
    v.extend_from_slice(id.as_bytes());
    v
}

/// XChaCha20-Poly1305 mit zufälliger 24-Byte-Nonce.
pub fn seal(key: &Key32, aad: &[u8], plaintext: &[u8]) -> Result<Sealed> {
    let cipher = XChaCha20Poly1305::new(key.0.as_ref().into());
    let mut nonce = [0u8; NONCE_LEN];
    fill_random(&mut nonce)?;
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| Error::Crypto("Verschlüsselung fehlgeschlagen"))?;
    Ok(Sealed {
        nonce: nonce.to_vec(),
        ciphertext,
    })
}

pub fn open(key: &Key32, aad: &[u8], sealed: &Sealed) -> Result<Zeroizing<Vec<u8>>> {
    if sealed.nonce.len() != NONCE_LEN {
        return Err(Error::Decrypt);
    }
    let cipher = XChaCha20Poly1305::new(key.0.as_ref().into());
    cipher
        .decrypt(
            XNonce::from_slice(&sealed.nonce),
            Payload {
                msg: &sealed.ciphertext,
                aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| Error::Decrypt)
}

/// Wrappt einen Schlüssel mit einem Key-Encryption-Key. Kein eigenes Key-Wrap-Schema,
/// schlicht AEAD über die 32 Schlüsselbytes.
pub fn wrap_key(kek: &Key32, aad: &[u8], key: &Key32) -> Result<Sealed> {
    seal(kek, aad, &key.0)
}

pub fn unwrap_key(kek: &Key32, aad: &[u8], sealed: &Sealed) -> Result<Key32> {
    let bytes = open(kek, aad, sealed)?;
    Key32::from_slice(&bytes)
}

/// AAD für das Wrapping des Account-Schlüssels.
pub fn aad_account_key(by: &str) -> Vec<u8> {
    aad("account_key", by)
}

/// Wiederherstellungscode: 128 Bit Zufall, Crockford-Base32 in Vierergruppen,
/// z. B. `K7Q2-M9XD-…`. Wird beim Setup einmal angezeigt.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct RecoveryCode {
    bytes: [u8; 16],
}

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

impl RecoveryCode {
    pub fn generate() -> Result<RecoveryCode> {
        let mut bytes = [0u8; 16];
        fill_random(&mut bytes)?;
        Ok(RecoveryCode { bytes })
    }

    pub fn from_bytes(bytes: [u8; 16]) -> RecoveryCode {
        RecoveryCode { bytes }
    }

    pub(crate) fn bytes(&self) -> &[u8; 16] {
        &self.bytes
    }

    /// Menschenlesbare Form (26 Zeichen, gruppiert).
    pub fn display(&self) -> Zeroizing<String> {
        let raw = crockford_encode(&self.bytes);
        let mut s = String::with_capacity(raw.len() + raw.len() / 4);
        for (i, c) in raw.chars().enumerate() {
            if i > 0 && i % 4 == 0 {
                s.push('-');
            }
            s.push(c);
        }
        Zeroizing::new(s)
    }

    /// Nimmt Eingaben mit oder ohne Bindestriche, in beliebiger Groß-/Kleinschreibung,
    /// und mit den üblichen Verwechslungen (O→0, I/L→1).
    pub fn parse(s: &str) -> Result<RecoveryCode> {
        let cleaned: String = s
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .map(|c| match c.to_ascii_uppercase() {
                'O' => '0',
                'I' | 'L' => '1',
                c => c,
            })
            .collect();
        let bytes = crockford_decode(&cleaned)?;
        let arr: [u8; 16] = bytes
            .try_into()
            .map_err(|_| Error::Invalid("Wiederherstellungscode hat die falsche Länge".into()))?;
        Ok(RecoveryCode { bytes: arr })
    }

    /// Recovery Key (Argon2id über die Rohbytes mit dem Konto-Salt).
    pub fn derive_key(&self, salt: &[u8; SALT_LEN], params: &KdfParams) -> Result<Key32> {
        derive_stretched(&self.bytes, salt, params)
    }
}

/// Aus dem Recovery Key abgeleiteter Auth-Schlüssel für `/auth/recover`.
pub fn recovery_auth_key(recovery_key: &Key32) -> Key32 {
    recovery_key.subkey(info::RECOVERY_AUTH)
}

fn crockford_encode(bytes: &[u8]) -> String {
    let mut out = String::new();
    let mut buffer: u32 = 0;
    let mut bits = 0;
    for &b in bytes {
        buffer = (buffer << 8) | b as u32;
        bits += 8;
        while bits >= 5 {
            bits -= 5;
            out.push(CROCKFORD[((buffer >> bits) & 31) as usize] as char);
        }
    }
    if bits > 0 {
        out.push(CROCKFORD[((buffer << (5 - bits)) & 31) as usize] as char);
    }
    out
}

fn crockford_decode(s: &str) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits = 0;
    for c in s.bytes() {
        let v =
            CROCKFORD.iter().position(|&x| x == c).ok_or_else(|| {
                Error::Invalid(format!("Ungültiges Zeichen im Code: {}", c as char))
            })? as u32;
        buffer = (buffer << 5) | v;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push(((buffer >> bits) & 0xff) as u8);
        }
    }
    Ok(out)
}

/// Serde-Helfer: Bytes als Base64.
pub mod b64_bytes {
    use base64::{engine::general_purpose::STANDARD as B64, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(v: &[u8], s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&B64.encode(v))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<Vec<u8>, D::Error> {
        let s = String::deserialize(d)?;
        B64.decode(s).map_err(serde::de::Error::custom)
    }
}

/// Alles, was ein Konto lokal und auf dem Server über seine Schlüssel wissen muss.
/// Enthält keine geheimen Schlüssel, nur gewrappte.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KontoHeader {
    pub format_version: u16,
    #[serde(with = "b64_bytes")]
    pub salt: Vec<u8>,
    pub kdf: KdfParams,
    /// Account-Schlüssel, gewrappt mit dem Wrap-Schlüssel aus dem Passwort.
    pub wrapped_account_key: Sealed,
    /// Account-Schlüssel, gewrappt mit dem Recovery Key. Fehlt lokal auf Geräten, die
    /// per Login hinzukamen; der Dienst hält es und liefert es bei `/auth/recover`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wrapped_account_key_recovery: Option<Sealed>,
}

/// Ergebnis der Konto-Einrichtung.
pub struct NeuesKonto {
    pub header: KontoHeader,
    pub account_key: Key32,
    pub auth_key: Key32,
    pub recovery_auth_key: Key32,
    pub recovery_code: RecoveryCode,
}

/// Richtet ein neues Konto ein: Salt, Account-Schlüssel, Wiederherstellungscode,
/// beide Wrappings.
pub fn konto_einrichten(password: &[u8], kdf: KdfParams) -> Result<NeuesKonto> {
    let salt = random_salt()?;
    let stretched = derive_stretched(password, &salt, &kdf)?;
    let pk = split_password_keys(&stretched);
    let account_key = Key32::random()?;
    let recovery_code = RecoveryCode::generate()?;
    let recovery_key = recovery_code.derive_key(&salt, &kdf)?;

    let header = KontoHeader {
        format_version: FORMAT_VERSION,
        salt: salt.to_vec(),
        kdf,
        wrapped_account_key: wrap_key(&pk.wrap, &aad_account_key("password"), &account_key)?,
        wrapped_account_key_recovery: Some(wrap_key(
            &recovery_key,
            &aad_account_key("recovery"),
            &account_key,
        )?),
    };
    Ok(NeuesKonto {
        header,
        account_key,
        auth_key: pk.auth,
        recovery_auth_key: recovery_auth_key(&recovery_key),
        recovery_code,
    })
}

/// Entsperrt den Account-Schlüssel mit dem Master-Passwort. Liefert zusätzlich den
/// Auth-Schlüssel für den Login.
pub fn konto_entsperren(header: &KontoHeader, password: &[u8]) -> Result<(Key32, Key32)> {
    if header.format_version != FORMAT_VERSION {
        return Err(Error::FormatVersion(header.format_version, FORMAT_VERSION));
    }
    let salt: [u8; SALT_LEN] = header
        .salt
        .as_slice()
        .try_into()
        .map_err(|_| Error::Crypto("Salt hat die falsche Länge"))?;
    let stretched = derive_stretched(password, &salt, &header.kdf)?;
    let pk = split_password_keys(&stretched);
    let ak = unwrap_key(
        &pk.wrap,
        &aad_account_key("password"),
        &header.wrapped_account_key,
    )?;
    Ok((ak, pk.auth))
}

/// Entsperrt den Account-Schlüssel mit dem Wiederherstellungscode.
pub fn konto_wiederherstellen(header: &KontoHeader, code: &RecoveryCode) -> Result<Key32> {
    let salt: [u8; SALT_LEN] = header
        .salt
        .as_slice()
        .try_into()
        .map_err(|_| Error::Crypto("Salt hat die falsche Länge"))?;
    let rk = code.derive_key(&salt, &header.kdf)?;
    let wrapped = header
        .wrapped_account_key_recovery
        .as_ref()
        .ok_or_else(|| {
            Error::Invalid("Kein Wiederherstellungs-Wrapping auf diesem Gerät".into())
        })?;
    unwrap_key(&rk, &aad_account_key("recovery"), wrapped)
}

/// Neues Passwort setzen: nur das Wrapping wird erneuert, keine Neuverschlüsselung.
/// Wie das Recovery-Wrapping beim Passwortwechsel neu verankert wird.
///
/// Es hängt am Salt, und der Salt wechselt mit dem Passwort. Deshalb muss es bei jedem
/// Wechsel neu berechnet werden – sonst öffnet der Wiederherstellungscode das Konto
/// danach nicht mehr, und das fällt erst auf, wenn das Passwort weg ist.
pub enum RecoveryWechsel<'a> {
    /// Bisherigen Code behalten. Der Aufrufer muss ihn abfragen; er wird gegen den alten
    /// Header geprüft, bevor damit neu gewrappt wird.
    Behalten(&'a RecoveryCode),
    /// Neuen Code erzeugen. Der bisherige gilt danach nicht mehr und der neue muss einmal
    /// angezeigt werden.
    Neu,
}

/// Ergebnis eines Passwortwechsels. `recovery_auth_key` ändert sich immer mit, weil er
/// aus dem salzabhängigen Recovery Key stammt – der Dienst muss ihn neu speichern.
pub struct Passwortwechsel {
    pub header: KontoHeader,
    pub auth_key: Key32,
    pub recovery_auth_key: Key32,
    /// Gesetzt, wenn ein neuer Code erzeugt wurde. Dann einmal anzeigen.
    pub neuer_code: Option<RecoveryCode>,
}

pub fn passwort_wechseln(
    header: &KontoHeader,
    account_key: &Key32,
    new_password: &[u8],
    recovery: RecoveryWechsel<'_>,
) -> Result<Passwortwechsel> {
    // Einen behaltenen Code erst gegen den alten Header prüfen. Ein Tippfehler würde sonst
    // stillschweigend zum neuen Code – der aufgeschriebene wäre wertlos.
    let (code, neuer_code) = match recovery {
        RecoveryWechsel::Behalten(c) => {
            konto_wiederherstellen(header, c)?;
            (c.clone(), None)
        }
        RecoveryWechsel::Neu => {
            let c = RecoveryCode::generate()?;
            (c.clone(), Some(c))
        }
    };

    let salt = random_salt()?;
    let stretched = derive_stretched(new_password, &salt, &header.kdf)?;
    let pk = split_password_keys(&stretched);
    let recovery_key = code.derive_key(&salt, &header.kdf)?;

    let mut neu = header.clone();
    neu.salt = salt.to_vec();
    neu.wrapped_account_key = wrap_key(&pk.wrap, &aad_account_key("password"), account_key)?;
    neu.wrapped_account_key_recovery = Some(wrap_key(
        &recovery_key,
        &aad_account_key("recovery"),
        account_key,
    )?);

    Ok(Passwortwechsel {
        header: neu,
        auth_key: pk.auth,
        recovery_auth_key: recovery_auth_key(&recovery_key),
        neuer_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_open_roundtrip_und_aad() {
        let k = Key32::random().unwrap();
        let s = seal(&k, b"aad", b"hallo").unwrap();
        assert_eq!(open(&k, b"aad", &s).unwrap().as_slice(), b"hallo");
        assert!(open(&k, b"anders", &s).is_err());
        let k2 = Key32::random().unwrap();
        assert!(open(&k2, b"aad", &s).is_err());
    }

    #[test]
    fn konto_einrichten_und_entsperren() {
        let konto = konto_einrichten(b"korrekt batterie", KdfParams::schnell_fuer_tests()).unwrap();
        let (ak, auth) = konto_entsperren(&konto.header, b"korrekt batterie").unwrap();
        assert_eq!(ak.as_bytes(), konto.account_key.as_bytes());
        assert_eq!(auth.as_bytes(), konto.auth_key.as_bytes());
        assert!(konto_entsperren(&konto.header, b"falsch").is_err());
    }

    #[test]
    fn wiederherstellung_mit_code() {
        let konto = konto_einrichten(b"pw", KdfParams::schnell_fuer_tests()).unwrap();
        let shown = konto.recovery_code.display();
        let parsed = RecoveryCode::parse(&shown.to_lowercase()).unwrap();
        let ak = konto_wiederherstellen(&konto.header, &parsed).unwrap();
        assert_eq!(ak.as_bytes(), konto.account_key.as_bytes());
        let falsch = RecoveryCode::generate().unwrap();
        assert!(konto_wiederherstellen(&konto.header, &falsch).is_err());
    }

    #[test]
    fn wiederherstellung_ueberlebt_passwortwechsel() {
        // Der Wiederherstellungscode muss auch nach einem Passwortwechsel noch öffnen.
        // Sonst merkt man den Verlust erst, wenn das Passwort weg ist – im schlechtesten
        // denkbaren Moment.
        let konto = konto_einrichten(b"alt", KdfParams::schnell_fuer_tests()).unwrap();
        let code = RecoveryCode::parse(&konto.recovery_code.display()).unwrap();
        let w = passwort_wechseln(
            &konto.header,
            &konto.account_key,
            b"neues passwort",
            RecoveryWechsel::Behalten(&code),
        )
        .unwrap();
        assert!(w.neuer_code.is_none());
        let ak = konto_wiederherstellen(&w.header, &code).unwrap();
        assert_eq!(ak.as_bytes(), konto.account_key.as_bytes());
    }

    #[test]
    fn passwortwechsel_mit_neuem_code_entwertet_den_alten() {
        let konto = konto_einrichten(b"alt", KdfParams::schnell_fuer_tests()).unwrap();
        let alt = RecoveryCode::parse(&konto.recovery_code.display()).unwrap();
        let w = passwort_wechseln(
            &konto.header,
            &konto.account_key,
            b"neues passwort",
            RecoveryWechsel::Neu,
        )
        .unwrap();
        let frisch = w
            .neuer_code
            .as_ref()
            .expect("neuer Code muss geliefert werden");
        assert_eq!(
            konto_wiederherstellen(&w.header, frisch)
                .unwrap()
                .as_bytes(),
            konto.account_key.as_bytes()
        );
        assert!(konto_wiederherstellen(&w.header, &alt).is_err());
    }

    #[test]
    fn passwortwechsel_lehnt_falschen_code_ab() {
        // Ein Tippfehler darf nicht stillschweigend zum neuen Code werden.
        let konto = konto_einrichten(b"alt", KdfParams::schnell_fuer_tests()).unwrap();
        let falsch = RecoveryCode::generate().unwrap();
        assert!(passwort_wechseln(
            &konto.header,
            &konto.account_key,
            b"neu",
            RecoveryWechsel::Behalten(&falsch)
        )
        .is_err());
    }

    #[test]
    fn passwortwechsel_behaelt_account_key() {
        let konto = konto_einrichten(b"alt", KdfParams::schnell_fuer_tests()).unwrap();
        let code = RecoveryCode::parse(&konto.recovery_code.display()).unwrap();
        let w = passwort_wechseln(
            &konto.header,
            &konto.account_key,
            b"neu",
            RecoveryWechsel::Behalten(&code),
        )
        .unwrap();
        let (ak, _) = konto_entsperren(&w.header, b"neu").unwrap();
        assert_eq!(ak.as_bytes(), konto.account_key.as_bytes());
        assert!(konto_entsperren(&w.header, b"alt").is_err());
        // Der Auth-Schlüssel für den Dienst wandert mit dem Salt mit.
        assert_ne!(
            w.recovery_auth_key.as_bytes(),
            recovery_auth_key(
                &code
                    .derive_key(
                        konto.header.salt.as_slice().try_into().unwrap(),
                        &konto.header.kdf
                    )
                    .unwrap()
            )
            .as_bytes()
        );
    }

    #[test]
    fn crockford_roundtrip() {
        let bytes = [0u8, 255, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
        let enc = crockford_encode(&bytes);
        assert_eq!(enc.len(), 26);
        assert_eq!(crockford_decode(&enc).unwrap(), bytes);
    }

    #[test]
    fn subkeys_sind_verschieden() {
        let k = Key32::random().unwrap();
        assert_ne!(
            k.subkey(info::RECORDS).as_bytes(),
            k.subkey(info::VAULT).as_bytes()
        );
        assert_eq!(
            k.subkey(info::RECORDS).as_bytes(),
            k.subkey(info::RECORDS).as_bytes()
        );
    }

    #[test]
    fn sealed_compact() {
        let k = Key32::random().unwrap();
        let s = seal(&k, b"", b"x").unwrap();
        let c = s.to_compact();
        assert_eq!(Sealed::from_compact(&c).unwrap(), s);
    }
}
