//! Der MCP-Zugang über HTTP, einmal ganz durch: echter Speicher, echter Socket, echte
//! HTTP-Anfragen.
//!
//! Die Unit-Tests in `mcp::dienst` prüfen den Türsteher mit einem erfundenen Rückruf.
//! Hier hängt der Rückruf am selben Kern, den auch die Hülle benutzt – also wird geprüft,
//! was am Ende zählt: dass Werkzeuge wirken, dass Geschriebenes ankommt, und dass der
//! Tresor über diesen Weg nicht erreichbar ist.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use ulid::Ulid;

use lotse_core::crypto::Key32;
use lotse_core::mcp::dienst::{self, Bescheid};
use lotse_core::model::{Projekt, Stufe, Vorlage};
use lotse_core::store::Store;
use lotse_core::vault::{TresorEintrag, VaultKeys};

const TOKEN: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

/// Eine POST-Anfrage an den Dienst; liefert (Statuszeile, Rumpf).
fn post(port: u16, token: &str, rumpf: &str) -> (String, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
    s.set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .unwrap();
    let anfrage = format!(
        "POST {} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{rumpf}",
        dienst::PFAD,
        rumpf.len()
    );
    s.write_all(anfrage.as_bytes()).unwrap();
    let mut alles = String::new();
    s.read_to_string(&mut alles).unwrap();
    let (kopf, body) = alles.split_once("\r\n\r\n").unwrap_or((alles.as_str(), ""));
    (
        kopf.lines().next().unwrap_or("").to_string(),
        body.to_string(),
    )
}

fn rufen(port: u16, werkzeug: &str, args: Value) -> String {
    let (zeile, rumpf) = post(
        port,
        TOKEN,
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
                "params":{"name":werkzeug,"arguments":args}})
        .to_string(),
    );
    assert!(zeile.contains("200"), "{zeile}");
    let v: Value = serde_json::from_str(&rumpf).unwrap();
    v["result"]["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

#[test]
fn assistent_arbeitet_am_entsperrten_bestand_aber_nie_am_tresor() {
    // Ein Bestand, wie ihn die entsperrte App hält: ein Projekt mit Kurs und ein
    // Tresor-Eintrag mit einem Passwort darin.
    let ak = Key32::random().unwrap();
    let mut store = Store::open_in_memory(&ak, Ulid::new()).unwrap();
    let mut p = Projekt::neu("Gartenhaus", Vorlage::HausGarten, lotse_core::now_ms());
    p.kurs = "Fundament bis Oktober.".into();
    let projekt_id = p.id;
    store.projekt_speichern(&p).unwrap();

    let keys = VaultKeys::from_account_key(&ak);
    let e = TresorEintrag::neu(
        &keys,
        "Fritzbox Gartenhaus",
        vec![projekt_id],
        Stufe::Ueberall,
        &[("benutzer", "admin"), ("passwort", "Rollladen-Raupe-77")],
        lotse_core::now_ms(),
    )
    .unwrap();
    store.tresor_speichern(&e).unwrap();

    // Der Dienst, wie die Hülle ihn startet: Speicher hinter einer Sperre, je Anfrage
    // kurz genommen. Ein `None` darin heißt „gesperrt“ und beendet den Dienst.
    let sitzung = Arc::new(Mutex::new(Some(store)));
    let lauscher = dienst::binden(0).unwrap();
    let port = lauscher.local_addr().unwrap().port();
    let stop = Arc::new(AtomicBool::new(false));

    let sitzung_thread = sitzung.clone();
    let stop_thread = stop.clone();
    let server = std::thread::spawn(move || {
        dienst::bedienen(&lauscher, TOKEN, &stop_thread, |n| {
            let mut g = sitzung_thread.lock().unwrap();
            match g.as_mut() {
                Some(s) => Bescheid::Antwort(Box::new(lotse_core::mcp::anfrage(s, n))),
                None => Bescheid::Gesperrt,
            }
        });
    });

    // Handschlag.
    let (zeile, rumpf) = post(
        port,
        TOKEN,
        &json!({"jsonrpc":"2.0","id":0,"method":"initialize",
                "params":{"protocolVersion":"2024-11-05"}})
        .to_string(),
    );
    assert!(zeile.contains("200"), "{zeile}");
    let init: Value = serde_json::from_str(&rumpf).unwrap();
    assert_eq!(init["result"]["protocolVersion"], "2024-11-05");
    assert_eq!(init["result"]["serverInfo"]["name"], "lotse");

    // Werkzeugliste: fünf, und keines davon fasst den Tresor an.
    let (_, rumpf) = post(
        port,
        TOKEN,
        &json!({"jsonrpc":"2.0","id":1,"method":"tools/list"}).to_string(),
    );
    let liste: Value = serde_json::from_str(&rumpf).unwrap();
    let werkzeuge = liste["result"]["tools"].as_array().unwrap();
    assert_eq!(werkzeuge.len(), 5);
    // Kein Werkzeug, das den Tresor anfasst. Die Beschreibungen dürfen ihn nennen – sie
    // sagen dem Assistenten gerade, dass er dort nicht hinkommt.
    let namen: Vec<&str> = werkzeuge
        .iter()
        .map(|w| w["name"].as_str().unwrap())
        .collect();
    assert_eq!(
        namen,
        [
            "list_projects",
            "get_project_context",
            "log_activity",
            "list_open_threads",
            "search"
        ]
    );

    // Lesen.
    let ctx = rufen(
        port,
        "get_project_context",
        json!({"project": "Gartenhaus"}),
    );
    assert!(ctx.contains("Fundament"), "{ctx}");

    // Schreiben – und im echten Speicher nachsehen, nicht bloß der Antwort glauben.
    let quittung = rufen(
        port,
        "log_activity",
        json!({"project": "Gartenhaus", "text": "Schalung geprüft."}),
    );
    assert!(quittung.starts_with("Eingetragen"), "{quittung}");
    {
        let mut g = sitzung.lock().unwrap();
        let s = g.as_mut().unwrap();
        assert!(s
            .notizen(projekt_id)
            .unwrap()
            .iter()
            .any(|n| n.text.contains("Schalung")));
    }

    // Der Tresor: weder über die Suche noch über ein erfundenes Werkzeug.
    for wonach in ["Fritzbox", "admin", "Rollladen-Raupe-77"] {
        let treffer = rufen(port, "search", json!({ "query": wonach }));
        assert!(
            !treffer.contains("Fritzbox")
                && !treffer.contains("admin")
                && !treffer.contains("Rollladen"),
            "Suche nach {wonach} hat Tresor-Inhalt gezeigt: {treffer}"
        );
    }
    let (zeile, rumpf) = post(
        port,
        TOKEN,
        &json!({"jsonrpc":"2.0","id":7,"method":"tools/call",
                "params":{"name":"vault_list","arguments":{}}})
        .to_string(),
    );
    assert!(zeile.contains("200"), "{zeile}");
    let v: Value = serde_json::from_str(&rumpf).unwrap();
    assert_eq!(v["result"]["isError"], true);

    // Sperren: der Dienst endet und gibt den Port frei, ohne dass jemand ihn anhält.
    *sitzung.lock().unwrap() = None;
    let (zeile, _) = post(
        port,
        TOKEN,
        &json!({"jsonrpc":"2.0","id":8,"method":"ping"}).to_string(),
    );
    assert!(zeile.contains("503"), "{zeile}");
    server.join().unwrap();
    assert!(
        !stop.load(Ordering::SeqCst),
        "von selbst beendet, nicht abgeschaltet"
    );
}
