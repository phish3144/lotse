//! Testvektoren der Schlüsselableitung – die eine Seite.
//!
//! Die andere Seite ist `apps/web/scripts/wasm-browsertest.mjs`: dieselbe Datei, dieselben
//! Werte, ausgeführt in einem echten Browser über die WASM-Anbindung. Das ist das
//! Abschlusskriterium der Phase W1 aus `docs/WEB_CLIENT.md`.
//!
//! Warum überhaupt feste Werte: `seal`/`open` im Rundlauf beweist nur, dass der Code mit
//! sich selbst übereinstimmt. Er würde auch nach einer versehentlichen Änderung an der
//! Schlüsselhierarchie grün bleiben – und dann wäre jedes bestehende Konto ausgesperrt.

use lotse_core::crypto::{derive_stretched, split_password_keys, KdfParams, SALT_LEN};
use serde::Deserialize;

#[derive(Deserialize)]
struct Vektoren {
    format_version: u16,
    passwort: String,
    salt_b64: String,
    faelle: Vec<Fall>,
}

#[derive(Deserialize)]
struct Fall {
    was: String,
    m_kib: u32,
    t: u32,
    p: u32,
    stretched_hex: String,
    auth_hex: String,
    wrap_hex: String,
}

fn vektoren() -> Vektoren {
    let pfad = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/vektoren/kdf.json");
    let text = std::fs::read_to_string(pfad).expect("tests/vektoren/kdf.json fehlt");
    serde_json::from_str(&text).expect("tests/vektoren/kdf.json ist kein gültiges JSON")
}

#[test]
fn schluesselableitung_trifft_die_vektoren() {
    let v = vektoren();
    assert_eq!(
        v.format_version,
        lotse_core::FORMAT_VERSION,
        "Die Vektoren gehören zu einer anderen Formatversion. Wenn die Komposition \
         absichtlich geändert wurde: FORMAT_VERSION erhöhen, Vektoren neu erzeugen und \
         THREAT_MODEL.md nachziehen."
    );
    use base64::Engine;
    let salt = base64::engine::general_purpose::STANDARD
        .decode(&v.salt_b64)
        .expect("Salt ist kein Base64");
    let salt: [u8; SALT_LEN] = salt.try_into().expect("Salt hat die falsche Länge");

    assert!(!v.faelle.is_empty(), "Keine Fälle in den Vektoren");
    for f in &v.faelle {
        let params = KdfParams {
            m_kib: f.m_kib,
            t: f.t,
            p: f.p,
        };
        let gestreckt = derive_stretched(v.passwort.as_bytes(), &salt, &params)
            .unwrap_or_else(|e| panic!("{}: {e}", f.was));
        assert_eq!(
            gestreckt.to_hex().as_str(),
            f.stretched_hex,
            "{}: Argon2id liefert etwas anderes",
            f.was
        );
        let pk = split_password_keys(&gestreckt);
        assert_eq!(pk.auth.to_hex().as_str(), f.auth_hex, "{}: auth", f.was);
        assert_eq!(pk.wrap.to_hex().as_str(), f.wrap_hex, "{}: wrap", f.was);
    }
}

#[test]
fn auth_und_wrap_sind_verschieden() {
    // Domain Separation: derselbe gestreckte Schlüssel, zwei verschiedene Unterschlüssel.
    // Wären sie gleich, könnte der Dienst mit dem, was er ohnehin bekommt, den
    // Kontoschlüssel entwrappen.
    let v = vektoren();
    for f in &v.faelle {
        assert_ne!(f.auth_hex, f.wrap_hex, "{}", f.was);
        assert_ne!(f.auth_hex, f.stretched_hex, "{}", f.was);
        assert_ne!(f.wrap_hex, f.stretched_hex, "{}", f.was);
    }
}
