//! MCP über HTTP auf `127.0.0.1`, damit ein Assistent die **entsperrte** Sitzung der App
//! mitbenutzen kann.
//!
//! Warum überhaupt: der Stdio-Transport verlangt, dass der Client den Server startet.
//! Der Client hätte dann kein Passwort und müsste den Tresorschlüssel selbst ableiten –
//! also entweder das Passwort in einer Konfigurationsdatei oder gar kein Zugang. Über
//! HTTP dreht sich die Richtung um: die App hält die Sitzung, der Assistent klopft an.
//! Sperrt sich die App, endet der Dienst.
//!
//! Absicherung, in dieser Reihenfolge geprüft:
//!
//! 1. Der Lauscher hängt an `127.0.0.1`, nie an `0.0.0.0` – nichts aus dem Netz kommt an.
//! 2. `Origin`, falls gesetzt, muss localhost sein. Das ist der Schutz gegen DNS-Rebinding,
//!    den die MCP-Spezifikation verlangt: eine Seite im Browser darf sich nicht als
//!    lokaler Client ausgeben.
//! 3. `Authorization: Bearer <token>`, verglichen ohne frühen Abbruch. Das Token steht im
//!    lokalen `meta`-Speicher und wird nie synchronisiert.
//!
//! Es gibt bewusst keine CORS-Kopfzeilen und keine Antwort auf `OPTIONS`: ein Browser
//! soll diesen Dienst gar nicht erst benutzen können.
//!
//! Der Dienst kennt den Tresor nicht. Er reicht JSON-RPC-Nachrichten an einen Rückruf
//! weiter, den die Hülle stellt – dieselben Werkzeuge wie über Stdio, dieselbe Grenze.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::Value;

use crate::{Error, Result};

/// Voreinstellung für den Port. Frei wählbar; steht im lokalen `meta`-Speicher, damit die
/// einmal eingetragene Adresse im Assistenten gültig bleibt.
pub const PORT_STANDARD: u16 = 7457;

/// Der einzige Pfad, den der Dienst bedient.
pub const PFAD: &str = "/mcp";

const MAX_KOPF_BYTES: usize = 8 * 1024;
const MAX_RUMPF_BYTES: usize = 1024 * 1024;
const ZEITGRENZE: Duration = Duration::from_secs(10);
const WARTE: Duration = Duration::from_millis(100);

/// Was die Hülle auf eine Nachricht hin zurückgibt.
pub enum Bescheid {
    /// Antwort, oder nichts – bei einer Benachrichtigung schweigt JSON-RPC.
    Antwort(Box<Option<Value>>),
    /// Die Sitzung ist gesperrt. Der Dienst beendet sich.
    Gesperrt,
}

/// Ein neues Zugangstoken (32 Byte Zufall, hexadezimal).
pub fn token_erzeugen() -> Result<String> {
    let mut b = [0u8; 32];
    crate::crypto::fill_random(&mut b)?;
    Ok(b.iter().fold(String::with_capacity(64), |mut s, x| {
        use std::fmt::Write as _;
        let _ = write!(s, "{x:02x}");
        s
    }))
}

/// Bindet den Lauscher an die Loopback-Adresse. `port` `0` lässt das System einen freien
/// wählen – dann sagt `local_addr()`, welcher es wurde.
pub fn binden(port: u16) -> Result<TcpListener> {
    let lauscher = TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port)))
        .map_err(|e| Error::Other(format!("Port {port} lässt sich nicht öffnen: {e}")))?;
    lauscher.set_nonblocking(true)?;
    Ok(lauscher)
}

/// Die Adresse, die im Assistenten einzutragen ist.
pub fn adresse(lauscher: &TcpListener) -> String {
    match lauscher.local_addr() {
        Ok(a) => format!("http://127.0.0.1:{}{PFAD}", a.port()),
        Err(_) => format!("http://127.0.0.1{PFAD}"),
    }
}

/// Nimmt Verbindungen an, bis `stop` gesetzt wird oder die Sitzung sich sperrt.
///
/// Eine Verbindung nach der anderen, jede mit `Connection: close`: die Anfragen sind kurz,
/// und der Rückruf greift ohnehin auf einen Speicher zu, der nur einem gehört. Zeitgrenzen
/// auf Lesen und Schreiben sorgen dafür, dass ein hängender Client die Reihe nicht anhält.
pub fn bedienen<F>(lauscher: &TcpListener, token: &str, stop: &AtomicBool, mut beantworten: F)
where
    F: FnMut(&Value) -> Bescheid,
{
    while !stop.load(Ordering::SeqCst) {
        match lauscher.accept() {
            Ok((strom, _)) => {
                if !verbindung(strom, token, &mut beantworten) {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(WARTE);
            }
            Err(_) => break,
        }
    }
}

/// Bedient eine Verbindung. `false` heißt: die Sitzung ist gesperrt, der Dienst endet.
fn verbindung<F>(strom: TcpStream, token: &str, beantworten: &mut F) -> bool
where
    F: FnMut(&Value) -> Bescheid,
{
    let _ = strom.set_nonblocking(false);
    let _ = strom.set_read_timeout(Some(ZEITGRENZE));
    let _ = strom.set_write_timeout(Some(ZEITGRENZE));
    let Ok(klon) = strom.try_clone() else {
        return true;
    };
    let mut leser = BufReader::new(klon);
    let mut schreiber = strom;

    let Some(kopf) = kopf_lesen(&mut leser) else {
        antworten(&mut schreiber, 400, "Bad Request", "Kaputte Anfrage.");
        return true;
    };

    if kopf.pfad != PFAD {
        antworten(&mut schreiber, 404, "Not Found", "Unbekannter Pfad.");
        return true;
    }
    if let Some(o) = kopf.feld("origin") {
        if !origin_ist_lokal(o) {
            antworten(&mut schreiber, 403, "Forbidden", "Fremde Herkunft.");
            return true;
        }
    }
    let erlaubt = kopf
        .feld("authorization")
        .and_then(|v| {
            v.strip_prefix("Bearer ")
                .or_else(|| v.strip_prefix("bearer "))
        })
        .is_some_and(|t| gleich_konstant(t.trim(), token));
    if !erlaubt {
        kopfzeile_antwort(
            &mut schreiber,
            401,
            "Unauthorized",
            "Token fehlt oder stimmt nicht.",
            Some("WWW-Authenticate: Bearer\r\n"),
        );
        return true;
    }
    if kopf.methode != "POST" {
        antworten(
            &mut schreiber,
            405,
            "Method Not Allowed",
            "Nur POST. Dieser Dienst hält keinen SSE-Strom offen.",
        );
        return true;
    }

    let laenge: usize = kopf
        .feld("content-length")
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(0);
    if laenge > MAX_RUMPF_BYTES {
        antworten(
            &mut schreiber,
            413,
            "Payload Too Large",
            "Nachricht zu groß.",
        );
        return true;
    }
    let mut rumpf = vec![0u8; laenge];
    if leser.read_exact(&mut rumpf).is_err() {
        antworten(&mut schreiber, 400, "Bad Request", "Rumpf unvollständig.");
        return true;
    }

    let Ok(nachricht) = serde_json::from_slice::<Value>(&rumpf) else {
        // Parse-Fehler ist ein JSON-RPC-Fehler, kein HTTP-Fehler: der Client soll ihn in
        // seiner eigenen Sprache lesen können.
        json_senden(
            &mut schreiber,
            &serde_json::json!({"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":"Kein gültiges JSON"}}),
        );
        return true;
    };

    // Ein Stapel (Array) gehört zu Fassung 2025-03-26; einzelne Nachrichten zu allen.
    let mut antworten_liste = Vec::new();
    let nachrichten: Vec<&Value> = match nachricht.as_array() {
        Some(a) => a.iter().collect(),
        None => vec![&nachricht],
    };
    for n in nachrichten {
        match beantworten(n) {
            Bescheid::Gesperrt => {
                antworten(
                    &mut schreiber,
                    503,
                    "Service Unavailable",
                    "Lotse ist gesperrt.",
                );
                return false;
            }
            Bescheid::Antwort(a) => {
                if let Some(a) = *a {
                    antworten_liste.push(a);
                }
            }
        }
    }

    match antworten_liste.len() {
        // Nur Benachrichtigungen: die Spezifikation will 202 ohne Rumpf.
        0 => antworten(&mut schreiber, 202, "Accepted", ""),
        1 if !nachricht.is_array() => json_senden(&mut schreiber, &antworten_liste[0]),
        _ => json_senden(&mut schreiber, &Value::Array(antworten_liste)),
    }
    true
}

/// Anfragezeile und Kopfzeilen einer HTTP-Anfrage.
struct Kopf {
    methode: String,
    pfad: String,
    /// Name (klein geschrieben) und Wert, in der Reihenfolge des Eingangs.
    felder: Vec<(String, String)>,
}

impl Kopf {
    fn feld(&self, name: &str) -> Option<&str> {
        self.felder
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }
}

/// Liest Anfragezeile und Kopfzeilen. `None`, wenn die Anfrage nicht lesbar ist oder der
/// Kopf die Obergrenze überschreitet.
fn kopf_lesen(leser: &mut impl BufRead) -> Option<Kopf> {
    let mut gelesen = 0usize;
    let mut zeile = String::new();
    if leser.read_line(&mut zeile).ok()? == 0 {
        return None;
    }
    gelesen += zeile.len();
    let mut teile = zeile.trim_end().split(' ');
    let methode = teile.next()?.to_string();
    let ziel = teile.next()?.to_string();
    // Abfrageteil abschneiden: der Dienst kennt keine Parameter.
    let pfad = ziel.split(['?', '#']).next().unwrap_or("").to_string();

    let mut felder = Vec::new();
    loop {
        let mut z = String::new();
        if leser.read_line(&mut z).ok()? == 0 {
            return None;
        }
        gelesen += z.len();
        if gelesen > MAX_KOPF_BYTES {
            return None;
        }
        let z = z.trim_end_matches(['\r', '\n']);
        if z.is_empty() {
            break;
        }
        if let Some((name, wert)) = z.split_once(':') {
            felder.push((name.trim().to_ascii_lowercase(), wert.trim().to_string()));
        }
    }
    Some(Kopf {
        methode,
        pfad,
        felder,
    })
}

/// `Origin` zählt als lokal, wenn Schema und Rechnername auf diesen Rechner zeigen.
/// Alles andere – auch `null` – wird abgewiesen.
fn origin_ist_lokal(origin: &str) -> bool {
    let ohne_schema = match origin.split_once("://") {
        Some(("http" | "https", rest)) => rest,
        _ => return false,
    };
    let rechner = ohne_schema.split('/').next().unwrap_or("");
    let rechner = match rechner.rsplit_once(':') {
        // IPv6 in Klammern: der letzte Doppelpunkt kann zur Adresse gehören.
        Some((vorn, hinten)) if hinten.chars().all(|c| c.is_ascii_digit()) => vorn,
        _ => rechner,
    };
    matches!(rechner, "127.0.0.1" | "localhost" | "[::1]" | "::1")
}

/// Vergleicht ohne beim ersten Unterschied abzubrechen, damit die Laufzeit nichts über
/// das Token verrät.
fn gleich_konstant(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

fn json_senden(w: &mut impl Write, wert: &Value) {
    let rumpf = serde_json::to_vec(wert).unwrap_or_else(|_| b"{}".to_vec());
    schicken(w, 200, "OK", "application/json", &rumpf, None);
}

fn antworten(w: &mut impl Write, code: u16, grund: &str, text: &str) {
    kopfzeile_antwort(w, code, grund, text, None);
}

fn kopfzeile_antwort(w: &mut impl Write, code: u16, grund: &str, text: &str, extra: Option<&str>) {
    schicken(
        w,
        code,
        grund,
        "text/plain; charset=utf-8",
        text.as_bytes(),
        extra,
    );
}

fn schicken(
    w: &mut impl Write,
    code: u16,
    grund: &str,
    typ: &str,
    rumpf: &[u8],
    extra: Option<&str>,
) {
    let mut kopf = format!(
        "HTTP/1.1 {code} {grund}\r\nContent-Length: {}\r\nConnection: close\r\n",
        rumpf.len()
    );
    if !rumpf.is_empty() {
        kopf.push_str(&format!("Content-Type: {typ}\r\n"));
    }
    if let Some(e) = extra {
        kopf.push_str(e);
    }
    kopf.push_str("\r\n");
    let _ = w.write_all(kopf.as_bytes());
    let _ = w.write_all(rumpf);
    let _ = w.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Startet den Dienst mit einem Rückruf, der `initialize` beantwortet und sonst
    /// schweigt, und liefert Adresse plus Stop-Schalter.
    fn dienst() -> (
        u16,
        std::sync::Arc<AtomicBool>,
        std::thread::JoinHandle<usize>,
    ) {
        let lauscher = binden(0).unwrap();
        let port = lauscher.local_addr().unwrap().port();
        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let stop_thread = stop.clone();
        let h = std::thread::spawn(move || {
            let mut gezaehlt = 0usize;
            bedienen(&lauscher, "geheim", &stop_thread, |n| {
                gezaehlt += 1;
                if n["method"] == "sperren" {
                    return Bescheid::Gesperrt;
                }
                Bescheid::Antwort(Box::new(n.get("id").map(
                    |id| json!({"jsonrpc":"2.0","id":id,"result":{"echo":n["method"].clone()}}),
                )))
            });
            gezaehlt
        });
        (port, stop, h)
    }

    /// Schickt eine rohe Anfrage und liefert (Statuszeile, Rumpf).
    fn roh(port: u16, anfrage: &str) -> (String, String) {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        s.write_all(anfrage.as_bytes()).unwrap();
        let mut alles = String::new();
        s.read_to_string(&mut alles).unwrap();
        let (kopf, rumpf) = alles.split_once("\r\n\r\n").unwrap_or((alles.as_str(), ""));
        (
            kopf.lines().next().unwrap_or("").to_string(),
            rumpf.to_string(),
        )
    }

    fn post(port: u16, token: Option<&str>, origin: Option<&str>, rumpf: &str) -> (String, String) {
        let mut a = format!("POST {PFAD} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\nContent-Type: application/json\r\n", rumpf.len());
        if let Some(t) = token {
            a.push_str(&format!("Authorization: Bearer {t}\r\n"));
        }
        if let Some(o) = origin {
            a.push_str(&format!("Origin: {o}\r\n"));
        }
        a.push_str("\r\n");
        a.push_str(rumpf);
        roh(port, &a)
    }

    #[test]
    fn beantwortet_nur_mit_gueltigem_token() {
        let (port, stop, h) = dienst();

        // Ohne Token: 401, und der Rückruf wird gar nicht erst gefragt.
        let (zeile, _) = post(
            port,
            None,
            None,
            r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
        );
        assert!(zeile.contains("401"), "{zeile}");
        // Falsches Token, gleiche Länge: ebenfalls 401.
        let (zeile, _) = post(
            port,
            Some("geheiM"),
            None,
            r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
        );
        assert!(zeile.contains("401"), "{zeile}");

        // Mit Token: Antwort vom Rückruf.
        let (zeile, rumpf) = post(
            port,
            Some("geheim"),
            None,
            r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
        );
        assert!(zeile.contains("200"), "{zeile}");
        let v: Value = serde_json::from_str(&rumpf).unwrap();
        assert_eq!(v["result"]["echo"], "ping");

        stop.store(true, Ordering::SeqCst);
        // Eine letzte Verbindung, damit die Schleife aus dem Warten kommt.
        let _ = TcpStream::connect(("127.0.0.1", port));
        assert_eq!(h.join().unwrap(), 1, "nur die berechtigte Anfrage zählt");
    }

    #[test]
    fn weist_fremde_herkunft_und_fremde_wege_ab() {
        let (port, stop, h) = dienst();

        // DNS-Rebinding: Seite im Browser, Token geraten – die Herkunft verrät sie.
        let (zeile, _) = post(
            port,
            Some("geheim"),
            Some("https://boese.example"),
            r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
        );
        assert!(zeile.contains("403"), "{zeile}");
        // Ein lokaler Client mit Origin darf.
        let (zeile, _) = post(
            port,
            Some("geheim"),
            Some("http://localhost:5173"),
            r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
        );
        assert!(zeile.contains("200"), "{zeile}");
        // Anderer Pfad: 404, noch vor der Prüfung des Tokens.
        let (zeile, _) = roh(
            port,
            "GET /admin HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\n\r\n",
        );
        assert!(zeile.contains("404"), "{zeile}");
        // GET auf den richtigen Pfad: kein SSE-Strom.
        let (zeile, _) = roh(
            port,
            &format!("GET {PFAD} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer geheim\r\nContent-Length: 0\r\n\r\n"),
        );
        assert!(zeile.contains("405"), "{zeile}");

        stop.store(true, Ordering::SeqCst);
        let _ = TcpStream::connect(("127.0.0.1", port));
        h.join().unwrap();
    }

    #[test]
    fn benachrichtigung_kaputtes_json_und_sperre() {
        let (port, stop, h) = dienst();

        // Ohne `id` ist es eine Benachrichtigung: 202, kein Rumpf.
        let (zeile, rumpf) = post(
            port,
            Some("geheim"),
            None,
            r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#,
        );
        assert!(zeile.contains("202"), "{zeile}");
        assert!(rumpf.is_empty());

        // Kaputtes JSON kommt als JSON-RPC-Fehler zurück, nicht als HTTP-Fehler.
        let (zeile, rumpf) = post(port, Some("geheim"), None, "kein json");
        assert!(zeile.contains("200"), "{zeile}");
        let v: Value = serde_json::from_str(&rumpf).unwrap();
        assert_eq!(v["error"]["code"], -32700);

        // Sperrt sich die App, endet der Dienst von selbst.
        let (zeile, _) = post(
            port,
            Some("geheim"),
            None,
            r#"{"jsonrpc":"2.0","id":9,"method":"sperren"}"#,
        );
        assert!(zeile.contains("503"), "{zeile}");
        // `join` kehrt zurück, ohne dass jemand den Schalter umgelegt hat: der Dienst hat
        // sich wegen der gesperrten Sitzung selbst beendet.
        h.join().unwrap();
        assert!(!stop.load(Ordering::SeqCst));
    }

    #[test]
    fn erkennt_lokale_herkunft() {
        for gut in [
            "http://127.0.0.1:5173",
            "http://localhost",
            "https://localhost:1420",
            "http://[::1]:8080",
        ] {
            assert!(origin_ist_lokal(gut), "{gut}");
        }
        for schlecht in [
            "null",
            "https://boese.example",
            "http://127.0.0.1.boese.example",
            "file://",
            "http://localhost.boese.example:80",
        ] {
            assert!(!origin_ist_lokal(schlecht), "{schlecht}");
        }
    }

    #[test]
    fn token_sind_verschieden_und_lang_genug() {
        let a = token_erzeugen().unwrap();
        let b = token_erzeugen().unwrap();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
        assert!(gleich_konstant(&a, &a) && !gleich_konstant(&a, &b));
    }
}
