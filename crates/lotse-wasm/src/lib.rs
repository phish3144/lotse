//! WebAssembly-Anbindung des Lotse-Kerns.
//!
//! Hier steht **keine Logik**. Jede Funktion ist ein Adapter: JS-Werte hinein, Aufruf in
//! `lotse-core`, JSON oder ein undurchsichtiger Griff hinaus. Wer hier eine Entscheidung
//! trifft, die nicht auch nativ getroffen wird, baut die zweite Wahrheit, die
//! `docs/WEB_CLIENT.md` Abschnitt 4c verhindern soll.
//!
//! Zwei Regeln aus dem Bedrohungsmodell gelten wörtlich weiter:
//!
//! 1. **Schlüssel verlassen den WASM-Speicher nicht.** `Passwortschluessel` und
//!    `Kontoschluessel` sind Griffe ohne Auslesefunktion. Die einzige Ausnahme ist der
//!    `auth_key`: er geht per Konstruktion zum Dienst, der ihn nochmals hasht – genau wie
//!    beim nativen Client.
//! 2. **Die KDF-Parameter kommen von außen** (aus `prelogin`), nie aus diesem Code. Ein
//!    Gerät darf die Kosten nicht senken.

use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use lotse_core::crypto::{self, KdfParams, Key32, KontoHeader, RecoveryCode, Sealed};
use lotse_core::model::{Notiz, Projekt};
use lotse_core::sync::{RecordKind, Umschlag};
use lotse_core::{brief as brief_mod, hlc, vault};
use ulid::Ulid;
use wasm_bindgen::prelude::*;

/// Kernfehler in einen JS-`Error` mit der deutschen Meldung des Kerns.
fn js(e: lotse_core::Error) -> JsValue {
    JsValue::from(js_sys::Error::new(&e.to_string()))
}

fn js_text(s: &str) -> JsValue {
    JsValue::from(js_sys::Error::new(s))
}

fn json_aus<T: serde::Serialize>(v: &T) -> Result<String, JsValue> {
    serde_json::to_string(v).map_err(|e| js_text(&e.to_string()))
}

fn json_in<T: serde::de::DeserializeOwned>(s: &str, was: &str) -> Result<T, JsValue> {
    serde_json::from_str(s).map_err(|e| js_text(&format!("{was} konnte nicht gelesen werden: {e}")))
}

fn salt_aus_b64(salt_b64: &str) -> Result<[u8; crypto::SALT_LEN], JsValue> {
    let rohe = B64
        .decode(salt_b64.trim())
        .map_err(|_| js_text("Das Salt ist kein Base64."))?;
    rohe.try_into()
        .map_err(|_| js_text("Das Salt hat die falsche Länge."))
}

fn kdf(m_kib: u32, t: u32, p: u32) -> KdfParams {
    KdfParams { m_kib, t, p }
}

// --------------------------------------------------------------------------- Auskünfte

/// Version der Krypto-Komposition. Muss zum Dienst und zum Desktop passen.
#[wasm_bindgen(js_name = formatVersion)]
pub fn format_version() -> u16 {
    lotse_core::FORMAT_VERSION
}

/// Version des Datenschemas.
#[wasm_bindgen(js_name = schemaVersion)]
pub fn schema_version() -> u32 {
    lotse_core::SCHEMA_VERSION
}

/// Unix-Millisekunden aus der Uhr der Laufzeitumgebung – im Browser `Date.now`.
///
/// `f64` und nicht `i64`, damit auf der JS-Seite eine gewöhnliche `number` ankommt und
/// nicht ein `BigInt`. Millisekunden passen bis zum Jahr 287396 exakt in ein `f64`; die
/// Konvention im Kern (`i64`) bleibt davon unberührt.
#[wasm_bindgen(js_name = jetztMs)]
pub fn jetzt_ms() -> f64 {
    lotse_core::now_ms() as f64
}

/// Eine neue ULID. Der Zufall kommt aus WebCrypto.
#[wasm_bindgen(js_name = neueId)]
pub fn neue_id() -> String {
    Ulid::new().to_string()
}

// ------------------------------------------------------------------------- Schlüssel

/// Die beiden Schlüssel, die aus dem Master-Passwort entstehen. Der Wrap-Schlüssel bleibt
/// hier drin; nur der Auth-Schlüssel darf nach draußen, weil er zum Dienst muss.
#[wasm_bindgen]
pub struct Passwortschluessel {
    auth: Key32,
    wrap: Key32,
}

#[wasm_bindgen]
impl Passwortschluessel {
    /// Master-Passwort + Salt + KDF-Parameter aus `prelogin` → Auth- und Wrap-Schlüssel.
    ///
    /// Dauert mit den Produktionsparametern (m = 64 MiB, t = 3) auch im Browser mehrere
    /// Sekunden. Gehört deshalb in einen Web Worker, nicht auf den Hauptthread.
    #[wasm_bindgen(js_name = ausPasswort)]
    pub fn aus_passwort(
        passwort: &str,
        salt_b64: &str,
        m_kib: u32,
        t: u32,
        p: u32,
    ) -> Result<Passwortschluessel, JsValue> {
        let salt = salt_aus_b64(salt_b64)?;
        let gestreckt =
            crypto::derive_stretched(passwort.as_bytes(), &salt, &kdf(m_kib, t, p)).map_err(js)?;
        let pk = crypto::split_password_keys(&gestreckt);
        Ok(Passwortschluessel {
            auth: pk.auth,
            wrap: pk.wrap,
        })
    }

    /// Der Nachweis für den Dienst. Er hasht ihn nochmals, bevor er ihn vergleicht.
    #[wasm_bindgen(js_name = authKeyB64)]
    pub fn auth_key_b64(&self) -> String {
        B64.encode(self.auth.as_bytes())
    }

    /// Entwrappt den Kontoschlüssel aus dem gewrappten Wert, den der Dienst beim Login
    /// liefert. Ein falsches Passwort scheitert hier – am Poly1305-Tag, nicht am Dienst.
    #[wasm_bindgen(js_name = kontoOeffnen)]
    pub fn konto_oeffnen(&self, wrapped_kompakt: &str) -> Result<Kontoschluessel, JsValue> {
        let sealed = Sealed::from_compact(wrapped_kompakt.trim()).map_err(js)?;
        let konto = crypto::unwrap_key(&self.wrap, &crypto::aad_account_key("password"), &sealed)
            .map_err(js)?;
        Ok(Kontoschluessel::neu(konto))
    }
}

/// Der Kontoschlüssel. Aus ihm leiten sich Datensatz- und Tresorschlüssel ab; er selbst
/// hat keine Auslesefunktion und lebt nur, solange die Sitzung lebt.
#[wasm_bindgen]
pub struct Kontoschluessel {
    account: Key32,
    records: Key32,
}

impl Kontoschluessel {
    fn neu(account: Key32) -> Kontoschluessel {
        let records = account.subkey(crypto::info::RECORDS);
        Kontoschluessel { account, records }
    }

    /// Ohne Desktop-Schlüssel – der kommt konstruktionsbedingt nie in einen Browser.
    fn tresorschluessel(&self) -> vault::VaultKeys {
        vault::VaultKeys::from_account_key(&self.account)
    }
}

#[wasm_bindgen]
impl Kontoschluessel {
    /// Öffnet einen Umschlag und gibt den Klartext-Datensatz als JSON zurück. Prüft dabei
    /// Formatversion und AAD – ein vom Dienst vertauschter Datensatz fällt hier auf.
    #[wasm_bindgen(js_name = umschlagOeffnen)]
    pub fn umschlag_oeffnen(&self, umschlag_json: &str) -> Result<String, JsValue> {
        let u: Umschlag = json_in(umschlag_json, "Der Umschlag")?;
        let wert: serde_json::Value = u.open(&self.records).map_err(js)?;
        json_aus(&wert)
    }

    /// Versiegelt einen Datensatz. `kind` ist die Art aus dem Protokoll und deshalb
    /// englisch: `project`, `note`, `reference`, `vault_entry`, `device`, `settings`
    /// (`SYNC_PROTOCOL.md`).
    #[wasm_bindgen(js_name = umschlagSiegeln)]
    pub fn umschlag_siegeln(
        &self,
        kind: &str,
        id: &str,
        hlc_wert: &str,
        device_id: &str,
        klartext_json: &str,
    ) -> Result<String, JsValue> {
        let art = RecordKind::parse(kind).ok_or_else(|| {
            let bekannt: Vec<&str> = RecordKind::ALLE.iter().map(|k| k.as_str()).collect();
            js_text(&format!(
                "»{kind}« ist keine Datensatzart. Bekannt sind: {}.",
                bekannt.join(", ")
            ))
        })?;
        let id: Ulid = id
            .parse()
            .map_err(|_| js_text("Die Datensatz-Kennung ist keine ULID."))?;
        let geraet: Ulid = device_id
            .parse()
            .map_err(|_| js_text("Die Gerätekennung ist keine ULID."))?;
        if !hlc::is_valid(hlc_wert) {
            return Err(js_text("Der HLC-Wert ist ungültig."));
        }
        let wert: serde_json::Value = json_in(klartext_json, "Der Datensatz")?;
        let u = Umschlag::seal(&self.records, art, id, hlc_wert.to_string(), geraet, &wert)
            .map_err(js)?;
        json_aus(&u)
    }

    /// Liest ein Tresorfeld der Stufe `ueberall`. Die Stufe `nur_desktop` ist im Browser
    /// konstruktionsbedingt nicht lesbar – der Desktop-Schlüssel kommt nie hierher.
    #[wasm_bindgen(js_name = tresorFeldLesen)]
    pub fn tresor_feld_lesen(&self, eintrag_json: &str, feld: &str) -> Result<String, JsValue> {
        let eintrag: vault::TresorEintrag = json_in(eintrag_json, "Der Tresoreintrag")?;
        let wert = eintrag
            .feld_lesen(&self.tresorschluessel(), feld)
            .map_err(js)?;
        Ok(wert.to_string())
    }
}

/// Ergebnis der Konto-Einrichtung im Browser. Der Wiederherstellungscode steht genau
/// einmal hier – wird er nicht abgetippt, entsteht kein Konto (`WEB_CLIENT.md` 4d).
#[wasm_bindgen]
pub struct NeuesKonto {
    header: KontoHeader,
    konto: Option<Kontoschluessel>,
    auth_key: Key32,
    recovery_auth_key: Key32,
    recovery_code: String,
}

#[wasm_bindgen]
impl NeuesKonto {
    /// Der Konto-Header, wie er beim Registrieren mitgeht: Salt, KDF-Parameter und beide
    /// Wrappings des Kontoschlüssels. Enthält keinen offenen Schlüssel.
    #[wasm_bindgen(js_name = headerJson)]
    pub fn header_json(&self) -> Result<String, JsValue> {
        json_aus(&self.header)
    }

    #[wasm_bindgen(js_name = authKeyB64)]
    pub fn auth_key_b64(&self) -> String {
        B64.encode(self.auth_key.as_bytes())
    }

    #[wasm_bindgen(js_name = recoveryAuthKeyB64)]
    pub fn recovery_auth_key_b64(&self) -> String {
        B64.encode(self.recovery_auth_key.as_bytes())
    }

    /// Der Wiederherstellungscode in Anzeigeform. Muss abgetippt werden, bevor es
    /// weitergeht.
    #[wasm_bindgen(js_name = wiederherstellungscode)]
    pub fn wiederherstellungscode(&self) -> String {
        self.recovery_code.clone()
    }

    /// Prüft eine Abtippung gegen den erzeugten Code – Trennzeichen und Groß- und
    /// Kleinschreibung sind dabei gleichgültig, der Kern normalisiert beides.
    #[wasm_bindgen(js_name = codeStimmt)]
    pub fn code_stimmt(&self, abgetippt: &str) -> bool {
        let Ok(a) = RecoveryCode::parse(abgetippt) else {
            return false;
        };
        let Ok(b) = RecoveryCode::parse(&self.recovery_code) else {
            return false;
        };
        a.display().as_str() == b.display().as_str()
    }

    /// Gibt den Kontoschlüssel heraus – genau einmal, damit er nicht zweimal in JS liegt.
    #[wasm_bindgen(js_name = kontoschluessel)]
    pub fn kontoschluessel(&mut self) -> Result<Kontoschluessel, JsValue> {
        self.konto
            .take()
            .ok_or_else(|| js_text("Der Kontoschlüssel wurde schon entnommen."))
    }
}

/// Richtet ein Konto im Browser ein. Die KDF-Parameter kommen von außen, damit ein Gerät
/// die Kosten nicht senken kann.
#[wasm_bindgen(js_name = kontoEinrichten)]
pub fn konto_einrichten(passwort: &str, m_kib: u32, t: u32, p: u32) -> Result<NeuesKonto, JsValue> {
    let neu = crypto::konto_einrichten(passwort.as_bytes(), kdf(m_kib, t, p)).map_err(js)?;
    Ok(NeuesKonto {
        header: neu.header,
        konto: Some(Kontoschluessel::neu(neu.account_key)),
        auth_key: neu.auth_key,
        recovery_auth_key: neu.recovery_auth_key,
        recovery_code: neu.recovery_code.display().to_string(),
    })
}

/// Entsperrt ein Konto aus einem vollständigen Konto-Header (der Weg über `prelogin`
/// läuft über `Passwortschluessel`; dieser hier ist der Weg über einen mitgebrachten
/// Header).
#[wasm_bindgen(js_name = kontoEntsperren)]
pub fn konto_entsperren(header_json: &str, passwort: &str) -> Result<Kontoschluessel, JsValue> {
    let header: KontoHeader = json_in(header_json, "Der Konto-Header")?;
    let (account_key, _auth) =
        crypto::konto_entsperren(&header, passwort.as_bytes()).map_err(js)?;
    Ok(Kontoschluessel::neu(account_key))
}

// --------------------------------------------------------------------------- Umschläge

/// Plausibilitätsprüfung eines Umschlags – dieselbe, die der Dienst vornimmt.
#[wasm_bindgen(js_name = umschlagPruefen)]
pub fn umschlag_pruefen(umschlag_json: &str) -> Result<(), JsValue> {
    let u: Umschlag = json_in(umschlag_json, "Der Umschlag")?;
    u.validate().map_err(js)
}

/// Die Konfliktregel des Protokolls: gewinnt der eingehende Stand?
#[wasm_bindgen(js_name = gewinnt)]
pub fn gewinnt(eingehend_hlc: &str, vorhanden_hlc: Option<String>) -> bool {
    lotse_core::sync::gewinnt(eingehend_hlc, vorhanden_hlc.as_deref())
}

/// Die Logikuhr dieses Browsers. Ein Browser ist ein Gerät (`WEB_CLIENT.md` 4e), also
/// hat er seine eigene Gerätekennung und seinen eigenen Zähler.
#[wasm_bindgen]
pub struct Logikuhr {
    inner: hlc::Hlc,
}

#[wasm_bindgen]
impl Logikuhr {
    #[wasm_bindgen(constructor)]
    pub fn new(device_id: &str, groesster_gesehener: Option<String>) -> Result<Logikuhr, JsValue> {
        let geraet: Ulid = device_id
            .parse()
            .map_err(|_| js_text("Die Gerätekennung ist keine ULID."))?;
        let mut inner = hlc::Hlc::new(geraet);
        inner.restore(groesster_gesehener.as_deref()).map_err(js)?;
        Ok(Logikuhr { inner })
    }

    /// Nächster HLC-Wert für einen eigenen Schreibvorgang.
    #[wasm_bindgen(js_name = weiter)]
    pub fn weiter(&mut self, jetzt_ms: f64) -> String {
        self.inner.next(jetzt_ms as i64)
    }

    /// Nimmt einen fremden HLC-Wert zur Kenntnis, damit die eigene Uhr nicht zurückfällt.
    #[wasm_bindgen(js_name = gesehen)]
    pub fn gesehen(&mut self, fremd: &str) -> Result<(), JsValue> {
        self.inner.observe(fremd).map_err(js)
    }
}

// ------------------------------------------------------------------------------ Brief

/// Der Wo-war-ich-Brief: eine reine Funktion über Projekt und Notizen.
#[wasm_bindgen(js_name = brief)]
pub fn brief(projekt_json: &str, notizen_json: &str, jetzt_ms: f64) -> Result<String, JsValue> {
    let projekt: Projekt = json_in(projekt_json, "Das Vorhaben")?;
    let notizen: Vec<Notiz> = json_in(notizen_json, "Die Notizen")?;
    json_aus(&brief_mod::brief(&projekt, &notizen, jetzt_ms as i64))
}

/// Auffälligkeit eines Vorhabens: `ruhig`, `wartet`, `ueberfaellig`, `frist_nah`.
#[wasm_bindgen(js_name = auffaelligkeit)]
pub fn auffaelligkeit(
    projekt_json: &str,
    letzte_notiz_ts: Option<f64>,
    jetzt_ms: f64,
) -> Result<String, JsValue> {
    let projekt: Projekt = json_in(projekt_json, "Das Vorhaben")?;
    json_aus(&brief_mod::auffaelligkeit(
        &projekt,
        letzte_notiz_ts.map(|t| t as i64),
        jetzt_ms as i64,
    ))
}

/// Vorbefüllung für die Übergabenotiz.
#[wasm_bindgen(js_name = uebergabeVorschlag)]
pub fn uebergabe_vorschlag(brief_json: &str) -> Result<String, JsValue> {
    let b: brief_mod::Brief = json_in(brief_json, "Der Brief")?;
    Ok(brief_mod::uebergabe_vorschlag(&b))
}
