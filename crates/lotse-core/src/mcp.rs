//! MCP-Server (Model Context Protocol) über stdin/stdout, damit KI-Assistenten wie Claude
//! Code Projektkontext lesen und ins Logbuch schreiben können (`docs/CONCEPT.md`, 9).
//!
//! JSON-RPC 2.0, eine Nachricht pro Zeile. Unterstützt `initialize`, `ping`, `tools/list`
//! und `tools/call`. Einträge, die hierüber entstehen, tragen Quelle `mcp` und sind in der
//! Oberfläche filter- und sammelweise löschbar.
//!
//! Dieses Modul importiert **nie** aus `vault`. Es gibt kein Werkzeug, das Tresor-Einträge
//! auch nur auflistet; das ist Modulgrenze, nicht Konvention (`THREAT_MODEL.md`, 6).

use std::io::{BufRead, Write};

use serde_json::{json, Value};
use ulid::Ulid;

use crate::brief;
use crate::model::{Art, Notiz, Quelle, Status};
use crate::store::Store;
use crate::{now_ms, Result};

pub const PROTOKOLL_VERSION: &str = "2025-06-18";

/// Werkzeugbeschreibungen, wie `tools/list` sie liefert.
pub fn werkzeuge() -> Value {
    json!([
        {
            "name": "list_projects",
            "description": "Alle Projekte mit Status, Kurs, Tagen seit letztem Kontakt und Zahl offener Fäden.",
            "inputSchema": { "type": "object", "properties": {
                "status": { "type": "string", "description": "Optional: nur Projekte mit diesem Status (idee, aktiv, pausiert, wartet, abgeschlossen, eingemottet)" }
            } }
        },
        {
            "name": "get_project_context",
            "description": "Der Wo-war-ich-Brief eines Projekts: Kurs, letzte Übergabenotiz, offene Fäden, Aktivität seit dem letzten Besuch, Referenzen und die letzten Logbuch-Einträge. Projekt per ID oder Titel.",
            "inputSchema": { "type": "object", "properties": {
                "project": { "type": "string", "description": "Projekt-ID (ULID) oder Titel" },
                "entries": { "type": "integer", "description": "Wie viele Logbuch-Einträge, Standard 20" }
            }, "required": ["project"] }
        },
        {
            "name": "log_activity",
            "description": "Schreibt einen Logbuch-Eintrag mit Quelle »mcp«. Für offene Fäden art = offen. Bitte kurz und konkret, am besten am Ende einer Sitzung: was wurde getan, was ist der nächste Schritt.",
            "inputSchema": { "type": "object", "properties": {
                "project": { "type": "string", "description": "Projekt-ID (ULID) oder Titel" },
                "text": { "type": "string" },
                "art": { "type": "string", "enum": ["log", "offen", "entscheidung"], "description": "Standard log" }
            }, "required": ["project", "text"] }
        },
        {
            "name": "list_open_threads",
            "description": "Alle offenen Fäden, optional nur eines Projekts.",
            "inputSchema": { "type": "object", "properties": {
                "project": { "type": "string", "description": "Optional: Projekt-ID oder Titel" }
            } }
        },
        {
            "name": "search",
            "description": "Volltextsuche über Projekte, Logbücher und Referenzen. Tresor-Inhalte sind nie enthalten.",
            "inputSchema": { "type": "object", "properties": {
                "query": { "type": "string" }
            }, "required": ["query"] }
        }
    ])
}

fn projekt_finden(store: &Store, s: &str) -> Result<Option<crate::model::Projekt>> {
    if let Ok(id) = Ulid::from_string(s) {
        if let Some(p) = store.projekt(id)? {
            return Ok(Some(p));
        }
    }
    if let Some(p) = store.projekt_nach_titel(s)? {
        return Ok(Some(p));
    }
    let s_l = s.to_lowercase();
    let mut treffer = store
        .projekte()?
        .into_iter()
        .filter(|p| p.titel.to_lowercase().starts_with(&s_l));
    let erster = treffer.next();
    Ok(if treffer.next().is_none() {
        erster
    } else {
        None
    })
}

fn text_ergebnis(text: String) -> Value {
    json!({ "content": [{ "type": "text", "text": text }] })
}

fn fehler_ergebnis(text: String) -> Value {
    json!({ "content": [{ "type": "text", "text": text }], "isError": true })
}

/// Führt ein Werkzeug aus.
pub fn werkzeug_aufrufen(store: &mut Store, name: &str, args: &Value) -> Result<Value> {
    let jetzt = now_ms();
    let arg = |k: &str| args.get(k).and_then(Value::as_str).map(str::to_string);
    match name {
        "list_projects" => {
            let filter = arg("status").and_then(|s| Status::parse(&s));
            let mut zeilen = Vec::new();
            for k in store.hafen(jetzt)? {
                if filter.is_some_and(|f| k.projekt.status != f) {
                    continue;
                }
                zeilen.push(format!(
                    "- {} [{}] id={} · vor {} Tagen · {} offen · Kurs: {}",
                    k.projekt.titel,
                    k.projekt.status.as_str(),
                    k.projekt.id,
                    k.tage_seit,
                    k.offene_faeden,
                    if k.projekt.kurs.is_empty() {
                        "–"
                    } else {
                        &k.projekt.kurs
                    }
                ));
            }
            Ok(text_ergebnis(if zeilen.is_empty() {
                "Keine Projekte.".into()
            } else {
                zeilen.join("\n")
            }))
        }
        "get_project_context" => {
            let Some(s) = arg("project") else {
                return Ok(fehler_ergebnis("project fehlt".into()));
            };
            let Some(p) = projekt_finden(store, &s)? else {
                return Ok(fehler_ergebnis(format!("Kein Projekt »{s}«")));
            };
            let n = args.get("entries").and_then(Value::as_u64).unwrap_or(20) as usize;
            let b = store.brief(p.id, jetzt)?;
            let mut out = format!(
                "# {} [{}]\nid: {}\nKurs: {}\nVorlage: {} · Erwartungsintervall {} Tage · zuletzt vor {} Tagen\n",
                p.titel,
                p.status.as_str(),
                p.id,
                if p.kurs.is_empty() { "–" } else { &p.kurs },
                p.vorlage.anzeigename(),
                p.erwartungsintervall_tage,
                b.tage_seit
            );
            if let Some(w) = &p.wiedervorlage {
                out.push_str(&format!("Wiedervorlage: {w}\n"));
            }
            if let Some(u) = &b.letzte_uebergabe {
                out.push_str(&format!("\n## Letzte Übergabe\n{u}\n"));
            }
            if !b.offene_faeden.is_empty() {
                out.push_str("\n## Offene Fäden\n");
                for f in &b.offene_faeden {
                    out.push_str(&format!(
                        "- ({}) {}\n",
                        f.id,
                        f.text.lines().next().unwrap_or("")
                    ));
                }
            }
            if !b.aktivitaet_seit_letztem_besuch.is_empty() {
                out.push_str("\n## Seit dem letzten Besuch\n");
                for (q, n) in &b.aktivitaet_seit_letztem_besuch {
                    out.push_str(&format!("- {n} × {q}\n"));
                }
            }
            let refs = store.referenzen(p.id)?;
            if !refs.is_empty() {
                out.push_str("\n## Referenzen\n");
                for r in refs {
                    out.push_str(&format!(
                        "- {} ({}): {}\n",
                        r.typ.as_str(),
                        r.rolle.as_str(),
                        r.ziel
                    ));
                }
            }
            let notizen = store.notizen(p.id)?;
            out.push_str(&format!(
                "\n## Logbuch (letzte {} von {})\n",
                n.min(notizen.len()),
                notizen.len()
            ));
            for e in notizen.iter().rev().take(n) {
                out.push_str(&format!(
                    "- {} · {} · {}: {}\n",
                    crate::export::iso(e.ts),
                    e.art.as_str(),
                    e.quelle.as_str(),
                    e.text.trim()
                ));
            }
            Ok(text_ergebnis(out))
        }
        "log_activity" => {
            let (Some(s), Some(text)) = (arg("project"), arg("text")) else {
                return Ok(fehler_ergebnis("project und text sind Pflicht".into()));
            };
            let Some(p) = projekt_finden(store, &s)? else {
                return Ok(fehler_ergebnis(format!("Kein Projekt »{s}«")));
            };
            let art = match arg("art").as_deref() {
                Some("offen") => Art::Offen,
                Some("entscheidung") => Art::Entscheidung,
                _ => Art::Log,
            };
            if text.trim().is_empty() {
                return Ok(fehler_ergebnis("Leerer Text".into()));
            }
            let n = Notiz::neu(p.id, Quelle::Mcp, art, text.trim(), jetzt);
            store.notiz_speichern(&n)?;
            Ok(text_ergebnis(format!(
                "Eingetragen in »{}« ({}), id {}",
                p.titel,
                art.as_str(),
                n.id
            )))
        }
        "list_open_threads" => {
            let pid = match arg("project") {
                Some(s) => match projekt_finden(store, &s)? {
                    Some(p) => Some(p.id),
                    None => return Ok(fehler_ergebnis(format!("Kein Projekt »{s}«"))),
                },
                None => None,
            };
            let mut zeilen = Vec::new();
            for n in store.offene_faeden(pid)? {
                let titel = store
                    .projekt(n.projekt_id)?
                    .map(|p| p.titel)
                    .unwrap_or_default();
                zeilen.push(format!(
                    "- [{}] {} (id {}, vor {} Tagen)",
                    titel,
                    n.text.lines().next().unwrap_or(""),
                    n.id,
                    (jetzt - n.ts).max(0) / brief::MS_PRO_TAG
                ));
            }
            Ok(text_ergebnis(if zeilen.is_empty() {
                "Keine offenen Fäden.".into()
            } else {
                zeilen.join("\n")
            }))
        }
        "search" => {
            let Some(q) = arg("query") else {
                return Ok(fehler_ergebnis("query fehlt".into()));
            };
            let mut zeilen = Vec::new();
            for t in store.suche(&q)? {
                if t.kind == "tresor" {
                    continue; // nicht einmal Titel
                }
                let titel = t
                    .projekt_id
                    .and_then(|id| store.projekt(id).ok().flatten())
                    .map(|p| p.titel)
                    .unwrap_or_default();
                zeilen.push(format!(
                    "- {} · {}: {}",
                    t.kind,
                    titel,
                    t.ausschnitt.replace('\n', " ")
                ));
            }
            Ok(text_ergebnis(if zeilen.is_empty() {
                "Keine Treffer.".into()
            } else {
                zeilen.join("\n")
            }))
        }
        _ => Ok(fehler_ergebnis(format!("Unbekanntes Werkzeug {name}"))),
    }
}

/// Verarbeitet eine JSON-RPC-Anfrage. `None` für Benachrichtigungen ohne Antwort.
pub fn anfrage(store: &mut Store, req: &Value) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);
    let antwort = |r: Value| Some(json!({ "jsonrpc": "2.0", "id": id, "result": r }));
    let fehler = |code: i64, msg: String| {
        Some(json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": msg } }))
    };
    match method {
        "initialize" => antwort(json!({
            "protocolVersion": PROTOKOLL_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "lotse", "version": env!("CARGO_PKG_VERSION") },
            "instructions": "Lotse ist das Logbuch für alle Vorhaben der Nutzerin. Hole zu Beginn den Kontext mit get_project_context und trage am Ende einer Sitzung mit log_activity kurz ein, was getan wurde und was der nächste Schritt ist. Tresor-Inhalte sind über diese Schnittstelle nie erreichbar."
        })),
        "ping" => antwort(json!({})),
        "tools/list" => antwort(json!({ "tools": werkzeuge() })),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            match werkzeug_aufrufen(store, name, &args) {
                Ok(r) => antwort(r),
                Err(e) => antwort(fehler_ergebnis(e.to_string())),
            }
        }
        m if m.starts_with("notifications/") => None,
        _ => {
            if id.is_none() {
                None
            } else {
                fehler(-32601, format!("Methode {method} nicht unterstützt"))
            }
        }
    }
}

/// Bedient einen Strom von JSON-RPC-Zeilen bis EOF.
pub fn bedienen<R: BufRead, W: Write>(store: &mut Store, reader: R, mut writer: W) -> Result<()> {
    for zeile in reader.lines() {
        let zeile = zeile?;
        if zeile.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&zeile) {
            Ok(v) => v,
            Err(e) => {
                let err = json!({ "jsonrpc": "2.0", "id": null, "error": { "code": -32700, "message": format!("Parse error: {e}") } });
                writeln!(writer, "{err}")?;
                writer.flush()?;
                continue;
            }
        };
        if let Some(antwort) = anfrage(store, &req) {
            writeln!(writer, "{antwort}")?;
            writer.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Key32;
    use crate::model::{Projekt, Vorlage};

    #[test]
    fn werkzeuge_ueber_jsonrpc() {
        let ak = Key32::random().unwrap();
        let mut s = Store::open_in_memory(&ak, Ulid::new()).unwrap();
        let mut p = Projekt::neu("Gartenhaus", Vorlage::HausGarten, now_ms());
        p.kurs = "Fundament bis Oktober.".into();
        s.projekt_speichern(&p).unwrap();
        s.notiz_speichern(&Notiz::neu(
            p.id,
            Quelle::Cli,
            Art::Offen,
            "Bewehrung?",
            now_ms(),
        ))
        .unwrap();

        let init = anfrage(
            &mut s,
            &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        )
        .unwrap();
        assert_eq!(init["result"]["protocolVersion"], PROTOKOLL_VERSION);
        assert!(anfrage(
            &mut s,
            &json!({"jsonrpc":"2.0","method":"notifications/initialized"})
        )
        .is_none());

        let liste = anfrage(
            &mut s,
            &json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        )
        .unwrap();
        let namen: Vec<&str> = liste["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
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

        let ctx = anfrage(&mut s, &json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"get_project_context","arguments":{"project":"garten"}}})).unwrap();
        let text = ctx["result"]["content"][0]["text"].as_str().unwrap();
        assert!(
            text.contains("Gartenhaus")
                && text.contains("Bewehrung?")
                && text.contains("Fundament")
        );

        let log = anfrage(&mut s, &json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"log_activity","arguments":{"project":"Gartenhaus","text":"Schalung geprüft. Nächster Schritt: Beton."}}})).unwrap();
        assert!(log["result"]["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("Eingetragen"));
        let n = s.notizen(p.id).unwrap();
        assert!(n
            .iter()
            .any(|x| x.quelle == Quelle::Mcp && x.text.contains("Schalung")));

        let unbekannt = anfrage(&mut s, &json!({"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"vault_list","arguments":{}}})).unwrap();
        assert_eq!(unbekannt["result"]["isError"], true);
        let fehl = anfrage(
            &mut s,
            &json!({"jsonrpc":"2.0","id":6,"method":"resources/list"}),
        )
        .unwrap();
        assert_eq!(fehl["error"]["code"], -32601);
    }

    #[test]
    fn strom_bis_eof() {
        let ak = Key32::random().unwrap();
        let mut s = Store::open_in_memory(&ak, Ulid::new()).unwrap();
        let eingabe = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\nkaputt\n\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"list_projects\",\"arguments\":{}}}\n";
        let mut out = Vec::new();
        bedienen(&mut s, eingabe.as_bytes(), &mut out).unwrap();
        let zeilen: Vec<Value> = String::from_utf8(out)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(zeilen.len(), 3);
        assert_eq!(zeilen[0]["id"], 1);
        assert_eq!(zeilen[1]["error"]["code"], -32700);
        assert_eq!(zeilen[2]["result"]["content"][0]["text"], "Keine Projekte.");
    }
}
