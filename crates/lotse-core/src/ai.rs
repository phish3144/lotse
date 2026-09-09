//! Verdichtung des Briefs durch ein Sprachmodell, gegen eine OpenAI-kompatible
//! Schnittstelle.
//!
//! Eine Schnittstelle, austauschbares Ziel: Ollama (lokal, ohne Schlüssel), Gemini über
//! seinen OpenAI-kompatiblen Endpunkt, ebenso Groq, Mistral oder OpenRouter. Der
//! Unterschied ist eine Adresse, ein Modellname und ein optionaler Schlüssel.
//!
//! Drei Regeln aus `CONCEPT.md` Abschnitt 9 und `THREAT_MODEL.md`:
//!
//! 1. **Kein Tresor-Zugriff.** Wie `detect`, `forge`, `git` und `watcher` importiert
//!    dieses Modul nie aus `vault`. Zugangsdaten reicht der Aufrufer herein.
//! 2. **Nichts fließt still ab.** Was gesendet würde, ist als Text abrufbar
//!    (`anfrage_text`), bevor irgendetwas das Gerät verlässt. Die Oberfläche zeigt es.
//! 3. **KI ergänzt das Logbuch, sie ersetzt es nicht.** Das Ergebnis ist ein Vorschlag,
//!    den der Mensch übernimmt oder verwirft – keine automatisch erzeugte Aufgabe.

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Wohin gefragt wird. `basis_url` endet vor `/chat/completions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ziel {
    pub basis_url: String,
    pub modell: String,
    /// Fehlt bei Ollama; Pflicht bei allen gehosteten Anbietern.
    pub schluessel: Option<String>,
}

/// Die übliche Adresse einer lokalen Ollama-Installation.
pub const OLLAMA_URL: &str = "http://localhost:11434/v1";
/// Gemini spricht dieselbe Schnittstelle wie OpenAI unter diesem Pfad.
pub const GEMINI_URL: &str = "https://generativelanguage.googleapis.com/v1beta/openai";

impl Ziel {
    pub fn ollama(modell: &str) -> Ziel {
        Ziel {
            basis_url: OLLAMA_URL.to_string(),
            modell: modell.to_string(),
            schluessel: None,
        }
    }

    #[cfg(feature = "native")]
    fn url(&self, pfad: &str) -> String {
        format!("{}{pfad}", self.basis_url.trim_end_matches('/'))
    }

    /// Läuft das Ziel auf diesem Rechner? Dann verlässt nichts das Gerät, und es
    /// braucht keinen Schlüssel.
    pub fn lokal(&self) -> bool {
        self.basis_url.starts_with("http://localhost")
            || self.basis_url.starts_with("http://127.0.0.1")
    }

    /// Lokale Ziele brauchen keinen Schlüssel; entfernte schon. Das früh zu prüfen
    /// erspart eine Fehlermeldung des Anbieters, die niemand versteht.
    pub fn pruefen(&self) -> Result<()> {
        if self.modell.trim().is_empty() {
            return Err(Error::Invalid("Kein Modell gewählt".into()));
        }
        if !self.lokal() && self.schluessel.as_deref().unwrap_or("").trim().is_empty() {
            return Err(Error::Invalid(
                "Für dieses Ziel wird ein Schlüssel gebraucht".into(),
            ));
        }
        Ok(())
    }
}

// ------------------------------------------------------------------ Anfrage

/// Wozu gefragt wird. Jede Fähigkeit bekommt hier einen Eintrag; die Anweisung an das
/// Modell hängt am Zweck, nicht an der Aufrufstelle. Sonst wäre jede neue Fähigkeit eine
/// Kopie von `verdichten()` mit eigenem Zustimmungsfluss.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Zweck {
    /// Den „Wo war ich"-Brief auf wenige Sätze bringen.
    BriefVerdichten,
    /// Eine einzelne, ausdrücklich hergegebene Datei deuten.
    DateiDeuten,
}

impl Zweck {
    pub fn as_str(self) -> &'static str {
        match self {
            Zweck::BriefVerdichten => "brief_verdichten",
            Zweck::DateiDeuten => "datei_deuten",
        }
    }

    pub fn parse(s: &str) -> Option<Zweck> {
        [Zweck::BriefVerdichten]
            .into_iter()
            .find(|z| z.as_str() == s)
    }

    /// Wie der Zweck in der Oberfläche und im Protokoll heißt.
    pub fn anzeige(self) -> &'static str {
        match self {
            Zweck::BriefVerdichten => "Brief verdichten",
            Zweck::DateiDeuten => "Datei deuten",
        }
    }

    /// Die Anweisung an das Modell.
    pub fn anweisung(self) -> &'static str {
        match self {
            Zweck::BriefVerdichten => {
                "Du fasst für eine Person zusammen, die nach längerer Pause in ihr eigenes \
                 Projekt zurückkehrt. Schreibe höchstens fünf Sätze auf Deutsch: wo das \
                 Projekt steht und was der nächste Schritt wäre. Nenne nur, was in der \
                 Eingabe steht; erfinde nichts dazu. Keine Anrede, keine Überschrift, keine \
                 Aufzählung."
            }
            Zweck::DateiDeuten => {
                "Du liest ein Dokument für jemanden, der wenig Zeit hat. Schreibe auf \
                 Deutsch: worum es geht, die wichtigsten Zahlen, Fristen und Namen, und \
                 was daraus zu tun wäre. Höchstens zehn Sätze. Nenne nur, was im Text \
                 steht; wenn etwas fehlt oder unklar ist, sage das, statt es zu ergänzen. \
                 Keine Anrede, keine Überschrift."
            }
        }
    }
}

/// Obergrenze für eine einzelne Anfrage.
///
/// Heute schützt sie davor, versehentlich einen riesigen Text zu senden – bei bezahlten
/// Zielen kostet jedes Zeichen. Später ist das die Stelle, an der das Kontingent eines
/// Abos geprüft wird. Ein Deckel, der von Anfang an da ist, ist billig; einer, der nach
/// dem ersten Kostenschock nachgerüstet wird, ist es nicht.
pub const MAX_EINGABE_ZEICHEN: usize = 40_000;

/// Was ein Aufruf gekostet hat, so wie das Ziel es meldet.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Verbrauch {
    pub eingabe_token: u64,
    pub ausgabe_token: u64,
}

/// Antwort des Modells samt Verbrauch, soweit das Ziel ihn nennt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Antwort {
    pub text: String,
    /// Fehlt, wenn das Ziel keine Zahlen mitschickt.
    pub verbrauch: Option<Verbrauch>,
}

/// Baut den Text, der gesendet würde. Getrennt vom Senden, damit die Oberfläche ihn
/// zeigen kann, bevor etwas das Gerät verlässt.
pub fn anfrage_text(brief: &str, offene_faeden: &[String], titel: &str) -> String {
    let mut s = format!("Projekt: {titel}\n\n");
    if !brief.trim().is_empty() {
        s.push_str("Was seit dem letzten Besuch passiert ist:\n");
        s.push_str(brief.trim());
        s.push_str("\n\n");
    }
    if !offene_faeden.is_empty() {
        s.push_str("Offene Fäden:\n");
        for f in offene_faeden {
            s.push_str("- ");
            s.push_str(f.trim());
            s.push('\n');
        }
    }
    s.trim_end().to_string()
}

#[cfg(feature = "native")]
#[derive(Serialize)]
struct Nachricht<'a> {
    role: &'a str,
    content: &'a str,
}

#[cfg(feature = "native")]
#[derive(Serialize)]
struct AnfrageWire<'a> {
    model: &'a str,
    messages: Vec<Nachricht<'a>>,
    /// Wenig Streuung: das hier ist eine Zusammenfassung, keine Erfindung.
    temperature: f32,
    stream: bool,
}

// ------------------------------------------------------------------ Antwort

#[cfg(feature = "native")]
#[derive(Deserialize)]
struct AntwortWire {
    choices: Vec<Wahl>,
    /// Die meisten Ziele melden hier die Token-Zahlen; Pflicht ist es nicht.
    #[serde(default)]
    usage: Option<VerbrauchWire>,
}

#[cfg(feature = "native")]
#[derive(Deserialize)]
struct VerbrauchWire {
    #[serde(default)]
    prompt_tokens: u64,
    #[serde(default)]
    completion_tokens: u64,
}

#[cfg(feature = "native")]
#[derive(Deserialize)]
struct Wahl {
    message: AntwortNachricht,
}

#[cfg(feature = "native")]
#[derive(Deserialize)]
struct AntwortNachricht {
    content: Option<String>,
}

/// Liest Text und Verbrauch aus einer OpenAI-kompatiblen Antwort. Getrennt vom Holen,
/// damit es ohne Netz prüfbar ist.
#[cfg(feature = "native")]
fn antwort_lesen(json: &str) -> Result<Antwort> {
    let a: AntwortWire =
        serde_json::from_str(json).map_err(|e| Error::Other(format!("Antwort unlesbar: {e}")))?;
    let verbrauch = a.usage.map(|u| Verbrauch {
        eingabe_token: u.prompt_tokens,
        ausgabe_token: u.completion_tokens,
    });
    let text = a
        .choices
        .into_iter()
        .next()
        .and_then(|w| w.message.content)
        .unwrap_or_default();
    let text = text.trim().to_string();
    if text.is_empty() {
        return Err(Error::Other("Das Modell hat nichts geantwortet".into()));
    }
    Ok(Antwort { text, verbrauch })
}

#[cfg(feature = "native")]
#[derive(Deserialize)]
struct ModelleWire {
    data: Vec<ModellEintrag>,
}

#[cfg(feature = "native")]
#[derive(Deserialize)]
struct ModellEintrag {
    id: String,
}

#[cfg(feature = "native")]
fn modelle_lesen(json: &str) -> Result<Vec<String>> {
    let m: ModelleWire = serde_json::from_str(json)
        .map_err(|e| Error::Other(format!("Modell-Liste unlesbar: {e}")))?;
    Ok(m.data.into_iter().map(|e| e.id).collect())
}

// ------------------------------------------------------------------- Netz

#[cfg(feature = "native")]
fn agent() -> ureq::Agent {
    // Lokale Modelle brauchen auf schwacher Hardware Zeit.
    crate::netz::agent(std::time::Duration::from_secs(120))
}

/// Deutet den Statuscode einer Antwort. `None`, wenn alles in Ordnung ist.
#[cfg(feature = "native")]
fn status_deuten(status: u16, ziel: &Ziel) -> Option<Error> {
    match status {
        200..=299 => None,
        401 | 403 => Some(Error::Invalid("Der Schlüssel wird abgelehnt.".into())),
        404 => Some(Error::NotFound(format!(
            "Modell »{}« kennt dieses Ziel nicht.",
            ziel.modell
        ))),
        429 => Some(Error::Invalid(
            "Kontingent erschöpft. Bei den kostenlosen Zugängen ist das eine Grenze pro \
             Minute – kurz warten genügt meist."
                .into(),
        )),
        s => Some(Error::Netz(format!("Das Ziel antwortete {s}"))),
    }
}

/// Deutet einen Transportfehler. Bei einem lokalen Ziel ist die wahrscheinlichste
/// Ursache eine andere als bei einem entfernten – das gehört in die Meldung.
#[cfg(feature = "native")]
fn fehler_deuten(e: ureq::Error, ziel: &Ziel) -> Error {
    if ziel.lokal() {
        return Error::Netz(
            "Keine Antwort von Ollama. Läuft es? Sonst starten und ein Modell \
             laden: `ollama pull llama3.2`."
                .into(),
        );
    }
    crate::netz::fehler(e, "Das KI-Ziel")
}

/// Kurzer Blick, ob unter der Adresse überhaupt etwas antwortet. Eigener, knapper
/// Zeitrahmen: das hier läuft beim Öffnen der Einstellungen und darf nicht hängen.
#[cfg(feature = "native")]
pub fn erreichbar(basis_url: &str) -> bool {
    let a = crate::netz::agent(std::time::Duration::from_secs(2));
    a.get(format!("{}/models", basis_url.trim_end_matches('/')).as_str())
        .call()
        .is_ok_and(|r| r.status().is_success())
}

/// Verfügbare Modelle des Ziels. Damit muss niemand einen Modellnamen abtippen.
#[cfg(feature = "native")]
pub fn modelle(ziel: &Ziel) -> Result<Vec<String>> {
    let a = agent();
    let mut r = a.get(ziel.url("/models").as_str());
    if let Some(k) = ziel.schluessel.as_deref().filter(|k| !k.trim().is_empty()) {
        r = r.header("Authorization", &format!("Bearer {k}"));
    }
    let mut resp = r.call().map_err(|e| fehler_deuten(e, ziel))?;
    if let Some(f) = status_deuten(resp.status().as_u16(), ziel) {
        return Err(f);
    }
    let text = resp
        .body_mut()
        .read_to_string()
        .map_err(|e| Error::Other(e.to_string()))?;
    modelle_lesen(&text)
}

/// Schickt genau den Text, den der Aufrufer gezeigt hat, und liefert die Antwort samt
/// Verbrauch. Über dem Deckel wird gar nicht erst gesendet.
#[cfg(feature = "native")]
pub fn verdichten(ziel: &Ziel, zweck: Zweck, eingabe: &str) -> Result<Antwort> {
    ziel.pruefen()?;
    if eingabe.chars().count() > MAX_EINGABE_ZEICHEN {
        return Err(Error::Invalid(format!(
            "Die Anfrage ist zu lang ({} Zeichen, erlaubt sind {MAX_EINGABE_ZEICHEN}). \
             Kürze den Text oder wähle einen kleineren Ausschnitt.",
            eingabe.chars().count()
        )));
    }
    let a = agent();
    let koerper = AnfrageWire {
        model: &ziel.modell,
        messages: vec![
            Nachricht {
                role: "system",
                content: zweck.anweisung(),
            },
            Nachricht {
                role: "user",
                content: eingabe,
            },
        ],
        temperature: 0.2,
        stream: false,
    };
    let mut r = a.post(ziel.url("/chat/completions").as_str());
    if let Some(k) = ziel.schluessel.as_deref().filter(|k| !k.trim().is_empty()) {
        r = r.header("Authorization", &format!("Bearer {k}"));
    }
    let mut resp = r.send_json(&koerper).map_err(|e| fehler_deuten(e, ziel))?;
    if let Some(f) = status_deuten(resp.status().as_u16(), ziel) {
        return Err(f);
    }
    let text = resp
        .body_mut()
        .read_to_string()
        .map_err(|e| Error::Other(e.to_string()))?;
    antwort_lesen(&text)
}

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;

    #[test]
    fn anfrage_zeigt_nur_was_uebergeben_wurde() {
        let t = anfrage_text(
            "3 Commits, 12 Dateien geändert.",
            &["Bewehrung nötig?".into(), "Statik prüfen".into()],
            "Gartenhaus",
        );
        assert!(t.starts_with("Projekt: Gartenhaus"));
        assert!(t.contains("3 Commits"));
        assert!(t.contains("- Bewehrung nötig?"));
        assert!(t.contains("- Statik prüfen"));
    }

    #[test]
    fn anfrage_ohne_faeden_und_ohne_brief() {
        let t = anfrage_text("", &[], "Leer");
        assert_eq!(t, "Projekt: Leer");
    }

    #[test]
    fn liest_openai_kompatible_antwort() {
        // Gestalt, die Ollama, Gemini, Groq und OpenRouter gleichermaßen liefern.
        let json = r#"{
          "id": "chatcmpl-1",
          "object": "chat.completion",
          "model": "llama3.2",
          "choices": [
            {
              "index": 0,
              "message": { "role": "assistant", "content": "  Das Fundament wartet auf Beton.  " },
              "finish_reason": "stop"
            }
          ],
          "usage": { "prompt_tokens": 40, "completion_tokens": 12, "total_tokens": 52 }
        }"#;
        let a = antwort_lesen(json).unwrap();
        assert_eq!(a.text, "Das Fundament wartet auf Beton.");
        // Die Token-Zahlen kommen mit und werden nicht mehr weggeworfen: ohne sie
        // gäbe es später keine Grundlage für eine faire Grenze.
        assert_eq!(
            a.verbrauch,
            Some(Verbrauch {
                eingabe_token: 40,
                ausgabe_token: 12
            })
        );
    }

    #[test]
    fn antwort_ohne_verbrauchsangabe_ist_kein_fehler() {
        // Nicht jedes Ziel meldet Token-Zahlen. Fehlen sie, fehlt die Angabe – die
        // Antwort bleibt gültig.
        let json = r#"{"choices":[{"message":{"role":"assistant","content":"Kurz."}}]}"#;
        let a = antwort_lesen(json).unwrap();
        assert_eq!(a.text, "Kurz.");
        assert_eq!(a.verbrauch, None);
    }

    #[test]
    fn zweck_traegt_seine_anweisung() {
        assert_eq!(
            Zweck::parse("brief_verdichten"),
            Some(Zweck::BriefVerdichten)
        );
        assert_eq!(Zweck::parse("erfunden"), None);
        assert!(Zweck::BriefVerdichten.anweisung().contains("fünf Sätze"));
        assert_eq!(Zweck::BriefVerdichten.anzeige(), "Brief verdichten");
    }

    #[test]
    fn zu_lange_eingabe_wird_gar_nicht_erst_gesendet() {
        // Der Deckel greift vor dem Netz: ein versehentlich riesiger Text kostet bei
        // bezahlten Zielen sonst echtes Geld.
        let ziel = Ziel::ollama("llama3.2");
        let zu_lang = "ä".repeat(MAX_EINGABE_ZEICHEN + 1);
        let e = verdichten(&ziel, Zweck::BriefVerdichten, &zu_lang).unwrap_err();
        assert!(e.to_string().contains("zu lang"), "{e}");
    }

    #[test]
    fn leere_antwort_ist_ein_fehler_kein_leerer_brief() {
        let leer = r#"{"choices":[{"message":{"role":"assistant","content":""}}]}"#;
        assert!(antwort_lesen(leer).is_err());
        let ohne = r#"{"choices":[]}"#;
        assert!(antwort_lesen(ohne).is_err());
        let null = r#"{"choices":[{"message":{"role":"assistant","content":null}}]}"#;
        assert!(antwort_lesen(null).is_err());
    }

    #[test]
    fn liest_modell_liste() {
        let json = r#"{
          "object": "list",
          "data": [
            { "id": "llama3.2", "object": "model", "owned_by": "library" },
            { "id": "qwen2.5-coder", "object": "model", "owned_by": "library" }
          ]
        }"#;
        assert_eq!(
            modelle_lesen(json).unwrap(),
            vec!["llama3.2", "qwen2.5-coder"]
        );
    }

    #[test]
    fn lokales_ziel_braucht_keinen_schluessel_entferntes_schon() {
        assert!(Ziel::ollama("llama3.2").pruefen().is_ok());

        let ohne = Ziel {
            basis_url: GEMINI_URL.into(),
            modell: "gemini-2.0-flash".into(),
            schluessel: None,
        };
        assert!(ohne.pruefen().is_err());

        let mit = Ziel {
            schluessel: Some("abc".into()),
            ..ohne
        };
        assert!(mit.pruefen().is_ok());
    }

    #[test]
    fn ohne_modell_gar_nicht_erst_senden() {
        let z = Ziel::ollama("");
        assert!(z.pruefen().is_err());
    }

    #[test]
    fn url_haengt_pfad_sauber_an() {
        let z = Ziel::ollama("m");
        assert_eq!(z.url("/models"), "http://localhost:11434/v1/models");
        let mit_schraegstrich = Ziel {
            basis_url: "http://localhost:11434/v1/".into(),
            ..z
        };
        assert_eq!(
            mit_schraegstrich.url("/chat/completions"),
            "http://localhost:11434/v1/chat/completions"
        );
    }
}
