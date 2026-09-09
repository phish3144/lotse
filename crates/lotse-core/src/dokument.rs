//! Text aus einer Datei ziehen, damit ein Mensch ihn deuten lassen kann.
//!
//! Das ist bewusst etwas anderes als der Ordner-Beobachter. Der sieht nur Namen und
//! Zeitstempel und liest nie Inhalte – diese Zusage steht so auf der Landing Page und
//! bleibt gültig. Hier gibt jemand ausdrücklich **eine** Datei her, sieht den Auszug vor
//! sich und entscheidet dann, ob er ihn an ein Modell schickt.
//!
//! Vier Regeln, die dieses Modul einhält:
//!
//! 1. **Kein Tresor-Zugriff.** Wie `ai`, `detect`, `forge`, `git` und `watcher`
//!    importiert es nie aus `vault` (`scripts/modulgrenzen.sh`).
//! 2. **Geheimnisse bleiben zu.** Dieselbe Ausschlussliste wie beim Beobachter
//!    (`detect::NIE_LESEN`): `.env`, Schlüssel, Zertifikate, Passwortdatenbanken.
//! 3. **Nichts von allein.** Es gibt keine Schleife über Ordner; der Aufrufer nennt
//!    genau eine Datei.
//! 4. **Nichts wird behalten.** Der Auszug wird zurückgegeben, nicht gespeichert.

use std::path::Path;

#[cfg(feature = "native")]
use crate::{Error, Result};

/// Was Lotse aufmachen kann.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Text,
    Markdown,
    Csv,
    Json,
    Pdf,
}

impl Format {
    pub fn as_str(self) -> &'static str {
        match self {
            Format::Text => "text",
            Format::Markdown => "markdown",
            Format::Csv => "csv",
            Format::Json => "json",
            Format::Pdf => "pdf",
        }
    }

    pub fn anzeige(self) -> &'static str {
        match self {
            Format::Text => "Textdatei",
            Format::Markdown => "Markdown",
            Format::Csv => "Tabelle (CSV)",
            Format::Json => "JSON",
            Format::Pdf => "PDF",
        }
    }
}

/// Was aus der Datei herausgekommen ist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Auszug {
    pub format: Format,
    /// Der Text, gegebenenfalls gekürzt.
    pub text: String,
    /// Zeichen im vollständigen Text, vor dem Kürzen.
    pub zeichen_gesamt: usize,
    pub gekuerzt: bool,
    /// Nur bei PDF: wie viele Seiten die Datei hat.
    pub seiten: Option<usize>,
}

/// Größer als das wird gar nicht erst aufgemacht. Wer ein Handbuch deuten lassen will,
/// schneidet das Kapitel heraus; alles andere sprengt ohnehin jedes Kontextfenster.
pub const MAX_DATEI_BYTES: u64 = 20 * 1024 * 1024;

/// Erkennt am Namen, ob Lotse die Datei aufmachen kann. `None` heißt schlicht: dieses
/// Format nicht – kein Fehler, nur nichts zu holen.
pub fn format_von(pfad: &Path) -> Option<Format> {
    let endung = pfad.extension()?.to_str()?.to_ascii_lowercase();
    Some(match endung.as_str() {
        "txt" | "log" | "text" => Format::Text,
        "md" | "markdown" => Format::Markdown,
        "csv" | "tsv" => Format::Csv,
        "json" => Format::Json,
        "pdf" => Format::Pdf,
        _ => return None,
    })
}

/// Kürzt auf `max_zeichen`, ohne mitten in ein Zeichen zu schneiden.
#[cfg(any(feature = "native", test))]
fn kuerzen(text: &str, max_zeichen: usize) -> (String, usize, bool) {
    let gesamt = text.chars().count();
    if gesamt <= max_zeichen {
        return (text.to_string(), gesamt, false);
    }
    (text.chars().take(max_zeichen).collect(), gesamt, true)
}

/// Räumt den Text auf, den ein PDF hergibt: viele Zeilenumbrüche mitten im Satz, doppelte
/// Leerzeichen, Seitenreste. Absätze bleiben erhalten.
#[cfg(any(feature = "native", test))]
fn aufraeumen(roh: &str) -> String {
    let mut out = String::with_capacity(roh.len());
    let mut leerzeilen = 0usize;
    for zeile in roh.lines() {
        let z = zeile.trim();
        if z.is_empty() {
            leerzeilen += 1;
            continue;
        }
        if !out.is_empty() {
            out.push(if leerzeilen > 0 { '\n' } else { ' ' });
            if leerzeilen > 0 {
                out.push('\n');
            }
        }
        leerzeilen = 0;
        let mut letztes_leer = false;
        for c in z.chars() {
            let leer = c.is_whitespace();
            if leer && letztes_leer {
                continue;
            }
            out.push(if leer { ' ' } else { c });
            letztes_leer = leer;
        }
    }
    out
}

/// Zieht den Text aus einer PDF-Datei.
///
/// Läuft in `catch_unwind`: ein PDF kommt von außen, und ein Parser, der über eine
/// kaputte Datei stolpert, soll die Anwendung nicht mitreißen.
#[cfg(feature = "native")]
fn pdf_text(pfad: &Path) -> Result<(String, usize)> {
    let pfad = pfad.to_path_buf();
    let ergebnis = std::panic::catch_unwind(move || -> Result<(String, usize)> {
        let doc = lopdf::Document::load(&pfad)
            .map_err(|e| Error::Invalid(format!("PDF nicht lesbar: {e}")))?;
        let seiten: Vec<u32> = doc.get_pages().keys().copied().collect();
        let anzahl = seiten.len();
        let text = doc
            .extract_text(&seiten)
            .map_err(|e| Error::Invalid(format!("Kein Text aus dem PDF: {e}")))?;
        Ok((text, anzahl))
    });
    match ergebnis {
        Ok(r) => r,
        Err(_) => Err(Error::Invalid(
            "Diese PDF-Datei lässt sich nicht auslesen; sie scheint beschädigt zu sein.".into(),
        )),
    }
}

/// Liest eine Datei und gibt ihren Text zurück, höchstens `max_zeichen` lang.
///
/// Fehler statt Panik in allen Fällen, die von außen kommen: Datei fehlt, zu groß, auf
/// der Ausschlussliste, kein Text darin.
#[cfg(feature = "native")]
pub fn lesen(pfad: &Path, max_zeichen: usize) -> Result<Auszug> {
    let name = pfad
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::Invalid("Kein Dateiname".into()))?;
    if crate::detect::nie_lesen(name) {
        return Err(Error::Invalid(format!(
            "»{name}« steht auf der Ausschlussliste: Schlüssel, Zertifikate und \
             Konfigurationsdateien mit Geheimnissen liest Lotse nicht."
        )));
    }
    let format = format_von(pfad).ok_or_else(|| {
        Error::Invalid(format!(
            "»{name}« ist ein Format, das Lotse nicht aufmacht. Es gehen Text, Markdown, \
             CSV, JSON und PDF."
        ))
    })?;

    let daten = std::fs::metadata(pfad)?;
    if !daten.is_file() {
        return Err(Error::Invalid(format!("»{name}« ist keine Datei.")));
    }
    if daten.len() > MAX_DATEI_BYTES {
        return Err(Error::Invalid(format!(
            "»{name}« ist {} MB groß; mehr als {} MB macht Lotse nicht auf.",
            daten.len() / 1_048_576,
            MAX_DATEI_BYTES / 1_048_576
        )));
    }

    let (roh, seiten) = match format {
        Format::Pdf => {
            let (t, n) = pdf_text(pfad)?;
            (aufraeumen(&t), Some(n))
        }
        _ => {
            let t = std::fs::read_to_string(pfad).map_err(|_| {
                Error::Invalid(format!(
                    "»{name}« ist kein lesbarer Text (vermutlich keine UTF-8-Datei)."
                ))
            })?;
            (t, None)
        }
    };

    if roh.trim().is_empty() {
        return Err(Error::Invalid(match format {
            Format::Pdf => format!(
                "In »{name}« steht kein auslesbarer Text. Vermutlich ein Scan – dafür \
                 bräuchte es Texterkennung, die Lotse nicht hat."
            ),
            _ => format!("»{name}« ist leer."),
        }));
    }

    let (text, zeichen_gesamt, gekuerzt) = kuerzen(&roh, max_zeichen);
    Ok(Auszug {
        format,
        text,
        zeichen_gesamt,
        gekuerzt,
        seiten,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erkennt_formate_am_namen() {
        let f = |n: &str| format_von(Path::new(n));
        assert_eq!(f("angebot.pdf"), Some(Format::Pdf));
        assert_eq!(f("ANGEBOT.PDF"), Some(Format::Pdf));
        assert_eq!(f("notizen.md"), Some(Format::Markdown));
        assert_eq!(f("stückliste.csv"), Some(Format::Csv));
        assert_eq!(f("export.json"), Some(Format::Json));
        assert_eq!(f("liesmich.txt"), Some(Format::Text));
        // Kein Format, kein Fehler: dazu gibt es schlicht nichts zu holen.
        assert_eq!(f("bild.png"), None);
        assert_eq!(f("archiv.zip"), None);
        assert_eq!(f("ohne_endung"), None);
    }

    #[test]
    fn kuerzt_ohne_zeichen_zu_zerschneiden() {
        let (t, gesamt, gekuerzt) = kuerzen("äöüß", 2);
        assert_eq!(t, "äö");
        assert_eq!(gesamt, 4);
        assert!(gekuerzt);
        let (t, gesamt, gekuerzt) = kuerzen("kurz", 99);
        assert_eq!(t, "kurz");
        assert_eq!(gesamt, 4);
        assert!(!gekuerzt);
    }

    #[test]
    fn raeumt_pdf_umbrueche_auf() {
        let roh = "Angebot   Nr. 42\nfür das Gartenhaus\n\n\nPosition 1:  Beton\n";
        assert_eq!(
            aufraeumen(roh),
            "Angebot Nr. 42 für das Gartenhaus\n\nPosition 1: Beton"
        );
    }

    #[cfg(feature = "native")]
    #[test]
    fn liest_text_und_haelt_sich_an_die_ausschlussliste() {
        let t = tempfile::tempdir().unwrap();

        let md = t.path().join("notiz.md");
        std::fs::write(&md, "# Gartenhaus\n\nBeton bestellt.").unwrap();
        let a = lesen(&md, 1000).unwrap();
        assert_eq!(a.format, Format::Markdown);
        assert!(a.text.contains("Beton bestellt"));
        assert!(!a.gekuerzt);
        assert_eq!(a.seiten, None);

        // Geheimnis-Dateien bleiben zu, auch wenn jemand sie ausdrücklich wählt.
        let env = t.path().join(".env");
        std::fs::write(&env, "SECRET=1").unwrap();
        let e = lesen(&env, 1000).unwrap_err();
        assert!(e.to_string().contains("Ausschlussliste"), "{e}");

        let key = t.path().join("server.pem");
        std::fs::write(&key, "-----BEGIN").unwrap();
        assert!(lesen(&key, 1000).is_err());

        // Fremdes Format: klare Auskunft, kein Absturz.
        let png = t.path().join("bild.png");
        std::fs::write(&png, [0u8, 1, 2]).unwrap();
        assert!(lesen(&png, 1000)
            .unwrap_err()
            .to_string()
            .contains("Format"));

        // Leere Datei ist ein Fehler, kein leerer Auszug.
        let leer = t.path().join("leer.txt");
        std::fs::write(&leer, "   \n").unwrap();
        assert!(lesen(&leer, 1000).is_err());
    }

    /// `beispiel.pdf` ist ein von Hand gebautes, gültiges PDF mit drei Textzeilen –
    /// der Beweis, dass der Auszug nicht nur Fehler abfängt, sondern auch etwas liefert.
    #[cfg(feature = "native")]
    #[test]
    fn liest_text_aus_einem_echten_pdf() {
        let pfad = format!("{}/tests/daten/beispiel.pdf", env!("CARGO_MANIFEST_DIR"));
        let a = lesen(Path::new(&pfad), 5000).unwrap();
        assert_eq!(a.format, Format::Pdf);
        assert_eq!(a.seiten, Some(1));
        assert!(a.text.contains("Angebot Nr. 42"), "{}", a.text);
        assert!(a.text.contains("Beton"), "{}", a.text);
        assert!(a.text.contains("30. September"), "{}", a.text);
        assert!(!a.gekuerzt);

        // Und mit engem Deckel wird gekürzt, nicht abgebrochen.
        let kurz = lesen(Path::new(&pfad), 20).unwrap();
        assert!(kurz.gekuerzt);
        assert_eq!(kurz.text.chars().count(), 20);
        assert!(kurz.zeichen_gesamt > 20);
    }

    #[cfg(feature = "native")]
    #[test]
    fn kaputtes_pdf_gibt_einen_fehler_statt_abzustuerzen() {
        let t = tempfile::tempdir().unwrap();
        let pdf = t.path().join("kaputt.pdf");
        std::fs::write(&pdf, b"%PDF-1.7\nnichts als Unsinn").unwrap();
        let e = lesen(&pdf, 1000).unwrap_err();
        assert!(e.to_string().to_lowercase().contains("pdf"), "{e}");
    }
}
