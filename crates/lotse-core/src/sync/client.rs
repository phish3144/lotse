//! HTTP-Client für den Sync-Dienst (`docs/SYNC_PROTOCOL.md`) und der Abgleich-Ablauf.
//!
//! Der Client kennt nur Umschläge und Schlüssel-Wrappings. Klartext oder Schlüsselmaterial
//! gehen hier nie über die Leitung; `auth_key` ist der abgeleitete Auth-Schlüssel, den der
//! Dienst nochmals hasht.

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::crypto::{self, KdfParams, Key32, KontoHeader, Sealed};
use crate::store::{Angewendet, Store};
use crate::sync::{PullResponse, PushRequest, PushResponse, StatusResponse, MAX_PUSH_BATCH};
use crate::{Error, Result};

/// Meta-Schlüssel im lokalen Speicher, unter denen der Sync-Stand liegt.
/// Der Dienst, den Lotse benutzt, wenn nichts anderes gesagt wird.
///
/// Fest eingebaut, damit niemand eine Adresse abtippen muss, um auf einem zweiten Gerät
/// an die eigenen Daten zu kommen. Dass er überschreibbar bleibt, ist kein Widerspruch,
/// sondern der Fluchtweg: der Client läuft gegen jede Basis-Adresse, die dasselbe
/// Protokoll spricht (`docs/SYNC_PROTOCOL.md`) – Selbsthosten bleibt möglich, es ist nur
/// nicht mehr die Voraussetzung.
pub const STANDARD_DIENST: &str = "https://lotse-sync.sanctora.eu";

/// Die Adresse, die gelten soll: die angegebene, sonst der Standard.
pub fn dienst_oder_standard(angegeben: Option<&str>) -> String {
    angegeben
        .map(str::trim)
        .filter(|u| !u.is_empty())
        .unwrap_or(STANDARD_DIENST)
        .trim_end_matches('/')
        .to_string()
}

pub mod meta {
    pub const URL: &str = "sync_url";
    pub const EMAIL: &str = "sync_email";
    pub const TOKEN: &str = "sync_token";
    pub const ACCOUNT_ID: &str = "sync_account_id";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Geraet {
    pub id: Ulid,
    pub name: String,
    /// `desktop`, `web` oder `cli`.
    pub platform: String,
}

/// Was beim Löschen eines Kontos wirklich weg ist. Zahlen statt »erledigt«: der Mensch
/// soll sagen können, was gelöscht wurde, und nicht darauf vertrauen müssen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KontoGeloescht {
    /// Gelöschte Umschläge.
    pub records: usize,
    /// Gelöschte Anhänge im Objektspeicher.
    pub blobs: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeraetInfo {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub created_at: i64,
    pub last_seen_at: i64,
}

/// KDF-Parameter, wie der Dienst sie erwartet (`m` in KiB).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KdfWire {
    m: u32,
    t: u32,
    p: u32,
}

impl From<KdfParams> for KdfWire {
    fn from(k: KdfParams) -> Self {
        KdfWire {
            m: k.m_kib,
            t: k.t,
            p: k.p,
        }
    }
}

impl From<KdfWire> for KdfParams {
    fn from(k: KdfWire) -> Self {
        KdfParams {
            m_kib: k.m,
            t: k.t,
            p: k.p,
        }
    }
}

#[derive(Debug, Deserialize)]
struct PreloginWire {
    salt: String,
    kdf: KdfWire,
}

#[derive(Debug, Deserialize)]
struct RegisterWire {
    account_id: String,
    session_token: String,
}

#[derive(Debug, Deserialize)]
struct LoginWire {
    account_id: String,
    session_token: String,
    salt: String,
    kdf: KdfWire,
    wrapped_account_key: String,
}

#[derive(Debug, Deserialize)]
struct FehlerWire {
    #[serde(default)]
    error: String,
    #[serde(default)]
    message: String,
}

pub struct Prelogin {
    pub salt: [u8; crypto::SALT_LEN],
    pub kdf: KdfParams,
}

pub struct Login {
    pub account_id: String,
    pub session_token: String,
    pub header: KontoHeader,
}

pub struct Client {
    base: String,
    agent: ureq::Agent,
    token: Option<String>,
}

impl Client {
    pub fn new(base_url: &str) -> Client {
        Client {
            base: base_url.trim_end_matches('/').to_string(),
            agent: crate::netz::agent(std::time::Duration::from_secs(30)),
            token: None,
        }
    }

    pub fn with_token(mut self, token: &str) -> Client {
        self.token = Some(token.to_string());
        self
    }

    pub fn base_url(&self) -> &str {
        &self.base
    }

    fn url(&self, pfad: &str) -> String {
        format!("{}/v1{}", self.base, pfad)
    }

    /// Anfrage ohne Rumpf (GET, DELETE), mit Anmeldung.
    fn ohne_rumpf(
        &self,
        method: &str,
        pfad: &str,
    ) -> ureq::RequestBuilder<ureq::typestate::WithoutBody> {
        let url = self.url(pfad);
        let r = if method == "DELETE" {
            self.agent.delete(url.as_str())
        } else {
            self.agent.get(url.as_str())
        };
        self.anmelden(r)
    }

    /// Anfrage mit Rumpf (POST), mit Anmeldung.
    fn mit_rumpf(&self, pfad: &str) -> ureq::RequestBuilder<ureq::typestate::WithBody> {
        let url = self.url(pfad);
        self.anmelden(self.agent.post(url.as_str()))
    }

    fn anmelden<T>(&self, r: ureq::RequestBuilder<T>) -> ureq::RequestBuilder<T> {
        match &self.token {
            Some(t) => r.header("Authorization", &format!("Bearer {t}")),
            None => r,
        }
    }

    fn netzfehler(e: ureq::Error) -> Error {
        crate::netz::fehler(e, "Der Sync-Dienst")
    }

    /// Prüft den Status und macht aus einer Fehlerantwort einen `Error::Sync` mit Code
    /// und Meldung des Dienstes – die Oberfläche zeigt sie wörtlich an.
    ///
    /// Der Agent meldet Statuscodes bewusst nicht selbst als Fehler
    /// (`http_status_as_error(false)`), damit der Rumpf hier noch lesbar ist.
    fn geprueft(
        mut resp: ureq::http::Response<ureq::Body>,
    ) -> Result<ureq::http::Response<ureq::Body>> {
        let status = resp.status().as_u16();
        if (200..300).contains(&status) {
            return Ok(resp);
        }
        let body: FehlerWire = resp.body_mut().read_json().unwrap_or(FehlerWire {
            error: String::new(),
            message: String::new(),
        });
        Err(Error::Sync {
            status,
            code: body.error,
            message: body.message,
        })
    }

    fn json<T: for<'de> Deserialize<'de>>(resp: ureq::http::Response<ureq::Body>) -> Result<T> {
        Self::geprueft(resp)?
            .body_mut()
            .read_json::<T>()
            .map_err(|e| Error::Netz(e.to_string()))
    }

    // ------------------------------------------------------------------ Konto

    pub fn prelogin(&self, email: &str) -> Result<Prelogin> {
        let resp = self
            .ohne_rumpf("GET", "/auth/prelogin")
            .query("email", email)
            .call()
            .map_err(Self::netzfehler)?;
        let w: PreloginWire = Self::json(resp)?;
        let salt = B64.decode(w.salt).map_err(|_| Error::Base64)?;
        let salt: [u8; crypto::SALT_LEN] = salt
            .try_into()
            .map_err(|_| Error::Crypto("Salt vom Dienst hat die falsche Länge"))?;
        Ok(Prelogin {
            salt,
            kdf: w.kdf.into(),
        })
    }

    /// Registriert ein Konto mit dem lokalen Konto-Header. Liefert (account_id, session_token).
    pub fn register(
        &self,
        email: &str,
        auth_key: &Key32,
        recovery_auth_key: &Key32,
        header: &KontoHeader,
        geraet: &Geraet,
    ) -> Result<(String, String)> {
        let recovery = header
            .wrapped_account_key_recovery
            .as_ref()
            .ok_or_else(|| {
                Error::Invalid("Registrierung braucht das Wiederherstellungs-Wrapping".into())
            })?;
        let body = serde_json::json!({
            "email": email,
            "auth_key": B64.encode(auth_key.as_bytes()),
            "recovery_auth_key": B64.encode(recovery_auth_key.as_bytes()),
            "salt": B64.encode(&header.salt),
            "kdf": KdfWire::from(header.kdf),
            "wrapped_account_key": header.wrapped_account_key.to_compact(),
            "wrapped_account_key_recovery": recovery.to_compact(),
            "device": geraet,
        });
        let resp = self
            .mit_rumpf("/auth/register")
            .send_json(body)
            .map_err(Self::netzfehler)?;
        let w: RegisterWire = Self::json(resp)?;
        Ok((w.account_id, w.session_token))
    }

    /// Meldet ein Gerät an. Liefert den Konto-Header, mit dem sich der Account-Schlüssel
    /// lokal entsperren lässt.
    pub fn login(&self, email: &str, auth_key: &Key32, geraet: &Geraet) -> Result<Login> {
        let body = serde_json::json!({
            "email": email,
            "auth_key": B64.encode(auth_key.as_bytes()),
            "device": geraet,
        });
        let resp = self
            .mit_rumpf("/auth/login")
            .send_json(body)
            .map_err(Self::netzfehler)?;
        let w: LoginWire = Self::json(resp)?;
        let header = KontoHeader {
            format_version: crate::FORMAT_VERSION,
            salt: B64.decode(w.salt).map_err(|_| Error::Base64)?,
            kdf: w.kdf.into(),
            wrapped_account_key: Sealed::from_compact(&w.wrapped_account_key)?,
            wrapped_account_key_recovery: None,
        };
        Ok(Login {
            account_id: w.account_id,
            session_token: w.session_token,
            header,
        })
    }

    pub fn logout(&self) -> Result<()> {
        let resp = self
            .mit_rumpf("/auth/logout")
            .send_empty()
            .map_err(Self::netzfehler)?;
        Self::geprueft(resp)?;
        Ok(())
    }

    /// Schließt eine Wiederherstellung beim Dienst ab: neue Zugangsdaten, beglaubigt mit
    /// dem Recovery-Auth-Schlüssel statt mit dem alten Passwort. Braucht keine Sitzung –
    /// wer sein Passwort vergessen hat, hat in der Regel auch keine mehr.
    pub fn wiederherstellung_abschliessen(
        &self,
        email: &str,
        alter_recovery_auth: &Key32,
        neuer_auth: &Key32,
        neuer_recovery_auth: &Key32,
        header: &KontoHeader,
    ) -> Result<()> {
        let recovery = header
            .wrapped_account_key_recovery
            .as_ref()
            .ok_or_else(|| {
                Error::Invalid("Wiederherstellung braucht das Wiederherstellungs-Wrapping".into())
            })?;
        let body = serde_json::json!({
            "email": email,
            "recovery_auth_key": B64.encode(alter_recovery_auth.as_bytes()),
            "new_auth_key": B64.encode(neuer_auth.as_bytes()),
            "new_salt": B64.encode(&header.salt),
            "new_kdf": KdfWire::from(header.kdf),
            "wrapped_account_key": header.wrapped_account_key.to_compact(),
            "new_recovery_auth_key": B64.encode(neuer_recovery_auth.as_bytes()),
            "wrapped_account_key_recovery": recovery.to_compact(),
        });
        let resp = self
            .mit_rumpf("/auth/recover/complete")
            .send_json(body)
            .map_err(Self::netzfehler)?;
        Self::geprueft(resp)?;
        Ok(())
    }

    /// Meldet einen Passwortwechsel beim Dienst. Das Recovery-Wrapping muss mitgehen:
    /// es hängt am Salt, und der wechselt hier. Der Dienst widerruft alle anderen
    /// Sitzungen, diese bleibt bestehen.
    pub fn passwort_wechseln(
        &self,
        alter_auth_key: &Key32,
        neuer_auth_key: &Key32,
        neuer_recovery_auth_key: &Key32,
        header: &KontoHeader,
    ) -> Result<()> {
        let recovery = header
            .wrapped_account_key_recovery
            .as_ref()
            .ok_or_else(|| {
                Error::Invalid("Passwortwechsel braucht das Wiederherstellungs-Wrapping".into())
            })?;
        let body = serde_json::json!({
            "old_auth_key": B64.encode(alter_auth_key.as_bytes()),
            "new_auth_key": B64.encode(neuer_auth_key.as_bytes()),
            "new_salt": B64.encode(&header.salt),
            "new_kdf": KdfWire::from(header.kdf),
            "wrapped_account_key": header.wrapped_account_key.to_compact(),
            "recovery_auth_key": B64.encode(neuer_recovery_auth_key.as_bytes()),
            "wrapped_account_key_recovery": recovery.to_compact(),
        });
        let resp = self
            .mit_rumpf("/auth/password")
            .send_json(body)
            .map_err(Self::netzfehler)?;
        Self::geprueft(resp)?;
        Ok(())
    }

    pub fn geraete(&self) -> Result<Vec<GeraetInfo>> {
        let resp = self
            .ohne_rumpf("GET", "/devices")
            .call()
            .map_err(Self::netzfehler)?;
        Self::json(resp)
    }

    /// Löscht das Konto beim Dienst – unwiderruflich, samt aller Umschläge und Anhänge.
    ///
    /// Verlangt den `auth_key` im Rumpf und nicht nur die Sitzung: ein gestohlenes
    /// Sitzungstoken darf kein Konto ausradieren. Den Wiederherstellungscode braucht es
    /// dagegen **nicht** – wer nicht mehr hineinkommt, hat trotzdem das Recht, seine
    /// Daten loszuwerden.
    ///
    /// Die lokale Datenbank bleibt davon unberührt. Das ist zwei getrennte Entscheidungen
    /// wert: »nicht mehr abgleichen« und »hier alles weg« (`konto::zuruecksetzen`).
    pub fn konto_loeschen(&self, auth_key: &Key32) -> Result<KontoGeloescht> {
        let resp = self
            .mit_rumpf("/account/delete")
            .send_json(serde_json::json!({ "auth_key": B64.encode(auth_key.as_bytes()) }))
            .map_err(Self::netzfehler)?;
        Self::json(resp)
    }

    pub fn geraet_widerrufen(&self, id: &str) -> Result<()> {
        let resp = self
            .ohne_rumpf("DELETE", &format!("/devices/{id}"))
            .call()
            .map_err(Self::netzfehler)?;
        Self::geprueft(resp)?;
        Ok(())
    }

    // ------------------------------------------------------------------- Sync

    pub fn push(&self, req: &PushRequest) -> Result<PushResponse> {
        let resp = self
            .mit_rumpf("/sync/push")
            .send_json(serde_json::to_value(req)?)
            .map_err(Self::netzfehler)?;
        Self::json(resp)
    }

    pub fn pull(&self, since: u64, limit: usize) -> Result<PullResponse> {
        let resp = self
            .ohne_rumpf("GET", "/sync/pull")
            .query("since", since.to_string())
            .query("limit", limit.to_string())
            .call()
            .map_err(Self::netzfehler)?;
        Self::json(resp)
    }

    pub fn status(&self) -> Result<StatusResponse> {
        let resp = self
            .ohne_rumpf("GET", "/sync/status")
            .call()
            .map_err(Self::netzfehler)?;
        Self::json(resp)
    }
}

/// Ergebnis eines Abgleichs.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Abgleich {
    pub gepusht: usize,
    pub abgelehnt: usize,
    pub uebernommen: usize,
    pub verworfen: usize,
    pub konflikte: usize,
    pub server_seq: u64,
}

/// Verbindung aus dem lokalen Speicher herstellen (URL und Token liegen dort).
pub fn client_aus_store(store: &Store) -> Result<Client> {
    // Leere Werte gelten als »nicht gesetzt«. Sonst entsteht ein Client mit leerer
    // Adresse, und der Fehler beim ersten Aufruf klingt nach Netzproblem statt nach
    // »noch nicht eingerichtet«.
    let url = store
        .meta_get(meta::URL)?
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| {
            Error::Invalid("Kein Sync eingerichtet (`lotse sync register` oder `login`)".into())
        })?;
    let token = store
        .meta_get(meta::TOKEN)?
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| Error::Invalid("Keine Sitzung; erneut anmelden".into()))?;
    Ok(Client::new(&url).with_token(&token))
}

/// Verbindung im Speicher merken.
pub fn verbindung_merken(
    store: &Store,
    url: &str,
    email: &str,
    account_id: &str,
    token: &str,
) -> Result<()> {
    store.meta_set(meta::URL, url.trim_end_matches('/'))?;
    store.meta_set(meta::EMAIL, email)?;
    store.meta_set(meta::ACCOUNT_ID, account_id)?;
    store.meta_set(meta::TOKEN, token)
}

/// Vergisst die gemerkte Verbindung: Adresse, Adresse, Kennung, Token.
///
/// Nach dem Löschen des Kontos nötig – ein gemerkter Dienst ohne Konto dahinter ist eine
/// Einladung, beim nächsten Abgleich in eine 401 zu laufen und sie für einen Fehler zu
/// halten. Die **Daten** bleiben unberührt; das hier trennt nur die Leitung.
pub fn verbindung_vergessen(store: &mut Store) -> Result<()> {
    for schluessel in [meta::URL, meta::EMAIL, meta::ACCOUNT_ID, meta::TOKEN] {
        store.meta_loeschen(schluessel)?;
    }
    // Auch der Abgleichstand muss weg. Bliebe `last_server_seq` stehen, würde ein später
    // angelegtes Konto erst ab dieser Nummer pullen und alles darunter überspringen – und
    // `last_local_seq` würde verhindern, dass der eigene Bestand noch einmal hochgeht.
    store.sync_state_setzen(&crate::sync::SyncState::default())
}

/// Der Abgleich nach `SYNC_PROTOCOL.md`, Abschnitt 5: erst alle ausstehenden Umschläge in
/// Bündeln pushen, dann alles seit der letzten Server-Sequenz pullen und lokal anwenden.
pub fn abgleichen(store: &mut Store, client: &Client) -> Result<Abgleich> {
    let mut ergebnis = Abgleich::default();

    loop {
        let (umschlaege, bis_seq) = store.ausstehende_umschlaege(MAX_PUSH_BATCH)?;
        if umschlaege.is_empty() {
            break;
        }
        let anzahl = umschlaege.len();
        let antwort = client.push(&PushRequest {
            records: umschlaege,
        })?;
        ergebnis.gepusht += antwort.accepted.len();
        ergebnis.abgelehnt += antwort.rejected.len();
        ergebnis.server_seq = antwort.server_seq;
        store.gepusht_bis(bis_seq)?;
        if anzahl < MAX_PUSH_BATCH {
            break;
        }
    }

    loop {
        let seit = store.sync_state()?.last_server_seq;
        let antwort = client.pull(seit, MAX_PUSH_BATCH)?;
        for u in &antwort.records {
            // Eigene Umschläge kommen zurück; die Konfliktregel verwirft sie still.
            match store.umschlag_anwenden(u)? {
                Angewendet::Uebernommen => ergebnis.uebernommen += 1,
                Angewendet::Verworfen => ergebnis.verworfen += 1,
                Angewendet::Konflikt => {
                    ergebnis.uebernommen += 1;
                    ergebnis.konflikte += 1;
                }
            }
        }
        store.gepullt_bis(antwort.next_seq)?;
        ergebnis.server_seq = ergebnis.server_seq.max(antwort.next_seq);
        if !antwort.has_more {
            break;
        }
    }

    // Konfliktnotizen sind neue lokale Änderungen; ein zweiter kurzer Push räumt sie weg.
    if ergebnis.konflikte > 0 {
        let (umschlaege, bis_seq) = store.ausstehende_umschlaege(MAX_PUSH_BATCH)?;
        if !umschlaege.is_empty() {
            let antwort = client.push(&PushRequest {
                records: umschlaege,
            })?;
            ergebnis.gepusht += antwort.accepted.len();
            store.gepusht_bis(bis_seq)?;
        }
    }
    Ok(ergebnis)
}
