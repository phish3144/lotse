//! Kalender lesen, nichts hineinschreiben.
//!
//! Lotse ist kein Kalender und will keiner werden (`NON_GOALS.md`). Was es beantworten
//! will, ist die zweite Hälfte von „wo stehe ich": *was steht für dieses Projekt an?*
//! Dafür genügt lesender Zugriff auf einen iCalendar-Datenstrom (RFC 5545) – die Datei,
//! die jeder Kalenderdienst als Abonnement-Adresse ausgibt.
//!
//! Zwei Entscheidungen, die den Rest erklären:
//!
//! 1. **Keine Zeitzonenrechnung.** Ein `DTSTART;TZID=Europe/Berlin:20260909T100000`
//!    steht für zehn Uhr, und genau das wird angezeigt. Ohne Zeitzonendatenbank wäre
//!    jede Umrechnung geraten; angezeigt wird deshalb, was im Kalender steht. Zeiten in
//!    UTC (`…Z`) werden als solche gekennzeichnet.
//! 2. **Keine Termine ins Logbuch.** Termine werden gezeigt, nicht kopiert. Sonst
//!    stünde derselbe Termin an zwei Orten und einer davon wäre irgendwann falsch.
//!
//! Das Modul liest keine Tresor-Werte (siehe `CLAUDE.md`).

/// Ein Termin, so wie er im Kalender steht.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Termin {
    pub titel: String,
    /// `JJJJ-MM-TT`.
    pub datum: String,
    /// `HH:MM`, oder `None` bei ganztägigen Terminen.
    pub uhrzeit: Option<String>,
    /// Die Uhrzeit steht im Kalender als UTC. Ohne Zeitzonendatenbank wird sie nicht
    /// umgerechnet, sondern gekennzeichnet.
    pub utc: bool,
    pub ort: Option<String>,
    /// Wiederholungsregel, falls der Termin eine hat.
    pub wiederholung: Option<Regel>,
}

/// Takt einer Wiederholung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Takt {
    Taeglich,
    Woechentlich,
    Monatlich,
    Jaehrlich,
}

/// Der Teil von `RRULE`, den Lotse ausrechnet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Regel {
    pub takt: Takt,
    pub intervall: u32,
    /// `COUNT`: so viele Vorkommen insgesamt, das erste eingeschlossen.
    pub anzahl: Option<u32>,
    /// `UNTIL` als `JJJJ-MM-TT`.
    pub bis: Option<String>,
    /// `BYDAY` bei wöchentlichem Takt: Wochentage 0 = Montag.
    pub wochentage: Vec<u8>,
    /// `WKST`: mit welchem Tag die Woche anfängt, 0 = Montag. Entscheidet bei
    /// `INTERVAL > 1`, welche Tage noch zur selben Woche gehören.
    pub wochenstart: u8,
    /// `false`, wenn die Regel Teile enthält, die Lotse nicht ausrechnet (etwa
    /// `BYDAY=2MO` oder `BYSETPOS`). Dann wird nichts hochgerechnet, sondern nur
    /// gesagt, dass sich der Termin wiederholt. Lieber keine Angabe als eine falsche.
    pub genau: bool,
}

impl Termin {
    /// Sortier- und Vergleichsschlüssel: `JJJJ-MM-TT HH:MM`. Ganztägige Termine kommen
    /// vor den Terminen mit Uhrzeit desselben Tages.
    pub fn schluessel(&self) -> String {
        format!("{} {}", self.datum, self.uhrzeit.as_deref().unwrap_or(""))
    }

    pub fn anzeige(&self) -> String {
        match (&self.uhrzeit, self.utc) {
            (Some(u), true) => format!("{} {u} UTC", self.datum),
            (Some(u), false) => format!("{} {u}", self.datum),
            (None, _) => self.datum.clone(),
        }
    }
}

// ------------------------------------------------------------------ Lesen

/// Hebt die Zeilenfaltung nach RFC 5545 auf: eine Fortsetzungszeile beginnt mit einem
/// Leerzeichen oder Tabulator. Google faltet mitten im Wort, deshalb wird ohne
/// Trennzeichen zusammengesetzt.
fn entfalten(ics: &str) -> Vec<String> {
    let mut zeilen: Vec<String> = Vec::new();
    for roh in ics.split('\n') {
        let z = roh.strip_suffix('\r').unwrap_or(roh);
        if let Some(rest) = z.strip_prefix([' ', '\t']) {
            if let Some(letzte) = zeilen.last_mut() {
                letzte.push_str(rest);
                continue;
            }
        }
        zeilen.push(z.to_string());
    }
    zeilen
}

/// Trennt `NAME;PARAM=wert:Inhalt` in Name, Parameter und Inhalt.
fn zerlegen(zeile: &str) -> Option<(&str, &str, &str)> {
    let doppelpunkt = zeile.find(':')?;
    let (kopf, rest) = zeile.split_at(doppelpunkt);
    let inhalt = &rest[1..];
    match kopf.find(';') {
        Some(i) => Some((&kopf[..i], &kopf[i + 1..], inhalt)),
        None => Some((kopf, "", inhalt)),
    }
}

/// Macht die Maskierung nach RFC 5545 rückgängig (`\,` `\;` `\n` `\\`).
fn entmaskieren(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut zeichen = s.chars();
    while let Some(c) = zeichen.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match zeichen.next() {
            Some('n' | 'N') => out.push(' '),
            Some(x) => out.push(x),
            None => {}
        }
    }
    out
}

/// Ein Zeitwert: `20260909`, `20260909T100000` oder `20260909T080000Z`.
fn zeitpunkt(wert: &str) -> Option<(String, Option<String>, bool)> {
    let (datum_teil, zeit_teil) = match wert.split_once('T') {
        Some((d, z)) => (d, Some(z)),
        None => (wert, None),
    };
    if datum_teil.len() != 8 || !datum_teil.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let datum = format!(
        "{}-{}-{}",
        &datum_teil[0..4],
        &datum_teil[4..6],
        &datum_teil[6..8]
    );
    let Some(z) = zeit_teil else {
        return Some((datum, None, false));
    };
    let utc = z.ends_with('Z');
    let z = z.trim_end_matches('Z');
    // Über die Bytes prüfen, nicht schneiden: `z[..4]` würde bei „000ä" mitten in ein
    // Zeichen fassen und die Abfrage abbrechen lassen. Sind die ersten vier Bytes
    // Ziffern, sind die Schnitte darunter sicher.
    let b = z.as_bytes();
    if b.len() < 4 || !b[..4].iter().all(u8::is_ascii_digit) {
        return None;
    }
    Some((datum, Some(format!("{}:{}", &z[0..2], &z[2..4])), utc))
}

/// `RRULE:FREQ=WEEKLY;INTERVAL=2;COUNT=10` und Verwandte.
fn regel(wert: &str) -> Option<Regel> {
    let mut takt = None;
    let mut intervall = 1u32;
    let mut anzahl = None;
    let mut bis = None;
    let mut wochentage = Vec::new();
    let mut wochenstart = 0u8;
    let mut genau = true;

    for teil in wert.split(';') {
        let Some((name, v)) = teil.split_once('=') else {
            continue;
        };
        match name.to_ascii_uppercase().as_str() {
            "FREQ" => {
                takt = match v.to_ascii_uppercase().as_str() {
                    "DAILY" => Some(Takt::Taeglich),
                    "WEEKLY" => Some(Takt::Woechentlich),
                    "MONTHLY" => Some(Takt::Monatlich),
                    "YEARLY" => Some(Takt::Jaehrlich),
                    // SECONDLY, MINUTELY, HOURLY: in einem Projektkalender kommt das
                    // nicht vor, und ausrechnen würde es hier nichts Sinnvolles.
                    _ => {
                        genau = false;
                        None
                    }
                }
            }
            "INTERVAL" => intervall = v.parse().unwrap_or(1).max(1),
            "COUNT" => anzahl = v.parse().ok(),
            "UNTIL" => bis = zeitpunkt(v).map(|(d, _, _)| d),
            "BYDAY" => {
                for tag in v.split(',') {
                    match wochentag(tag) {
                        Some(n) => wochentage.push(n),
                        // Etwa `2MO` („zweiter Montag im Monat“): nicht ausgerechnet.
                        None => genau = false,
                    }
                }
            }
            "WKST" => match wochentag(v) {
                Some(n) => wochenstart = n,
                None => genau = false,
            },
            // Alles Weitere verschiebt oder filtert Termine auf eine Art, die hier
            // nicht nachgebaut wird.
            "BYMONTHDAY" | "BYMONTH" | "BYSETPOS" | "BYWEEKNO" | "BYYEARDAY" => genau = false,
            _ => {}
        }
    }
    let takt = takt?;
    if takt != Takt::Woechentlich && !wochentage.is_empty() {
        genau = false;
    }
    Some(Regel {
        takt,
        intervall,
        anzahl,
        bis,
        wochentage,
        wochenstart,
        genau,
    })
}

fn wochentag(s: &str) -> Option<u8> {
    match s.trim().to_ascii_uppercase().as_str() {
        "MO" => Some(0),
        "TU" => Some(1),
        "WE" => Some(2),
        "TH" => Some(3),
        "FR" => Some(4),
        "SA" => Some(5),
        "SU" => Some(6),
        _ => None,
    }
}

/// Liest die Termine eines iCalendar-Datenstroms. Unbekannte Eigenschaften, fremde
/// Komponenten (`VTIMEZONE`, `VTODO`, `VALARM`) und Einträge ohne Beginn werden
/// übergangen – ein Kalender voller Sonderfälle soll nicht am Lesen scheitern.
pub fn lesen(ics: &str) -> Vec<Termin> {
    let mut out: Vec<Termin> = Vec::new();
    let mut tiefe_fremd = 0usize;
    let mut offen: Option<Roh> = None;

    for zeile in entfalten(ics) {
        let Some((name, params, wert)) = zerlegen(&zeile) else {
            continue;
        };
        match (name, wert) {
            ("BEGIN", "VEVENT") if tiefe_fremd == 0 && offen.is_none() => {
                offen = Some(Roh::default())
            }
            // Ein VALARM steckt im VEVENT; seine Felder gehören nicht zum Termin.
            ("BEGIN", _) if offen.is_some() => tiefe_fremd += 1,
            ("END", _) if tiefe_fremd > 0 => tiefe_fremd -= 1,
            ("END", "VEVENT") => {
                if let Some(t) = offen.take().and_then(Roh::fertig) {
                    out.push(t);
                }
            }
            _ if tiefe_fremd > 0 => {}
            _ => {
                if let Some(r) = offen.as_mut() {
                    r.feld(name, params, wert);
                }
            }
        }
    }
    out.sort_by_key(|t| t.schluessel());
    out
}

#[derive(Default)]
struct Roh {
    titel: Option<String>,
    beginn: Option<String>,
    ort: Option<String>,
    rrule: Option<String>,
}

impl Roh {
    fn feld(&mut self, name: &str, _params: &str, wert: &str) {
        match name {
            "SUMMARY" => self.titel = Some(entmaskieren(wert)),
            "DTSTART" => self.beginn = Some(wert.to_string()),
            "LOCATION" if !wert.trim().is_empty() => self.ort = Some(entmaskieren(wert)),
            "RRULE" => self.rrule = Some(wert.to_string()),
            _ => {}
        }
    }

    fn fertig(self) -> Option<Termin> {
        let (datum, uhrzeit, utc) = zeitpunkt(self.beginn.as_deref()?)?;
        Some(Termin {
            titel: self
                .titel
                .filter(|t| !t.trim().is_empty())
                .unwrap_or_else(|| "Ohne Titel".to_string()),
            datum,
            uhrzeit,
            utc,
            ort: self.ort,
            wiederholung: self.rrule.as_deref().and_then(regel),
        })
    }
}

// ------------------------------------------------------------- Ausrechnen

/// Sicherheitsnetze gegen Kalender, die täglich bis in alle Ewigkeit wiederholen.
/// `MAX_VORKOMMEN` begrenzt die Ausgabe, `MAX_SCHRITTE` die Rechnerei je Termin.
const MAX_VORKOMMEN: usize = 400;
const MAX_SCHRITTE: i64 = 2000;

fn datum_lesen(d: &str) -> Option<time::Date> {
    let format = time::macros::format_description!("[year]-[month]-[day]");
    time::Date::parse(d, &format).ok()
}

fn datum_schreiben(d: time::Date) -> String {
    format!("{:04}-{:02}-{:02}", d.year(), d.month() as u8, d.day())
}

/// Verschiebt ein Datum um `n` Monate. Gibt es den Tag im Zielmonat nicht (31. Februar),
/// fällt das Vorkommen aus – so steht es auch in RFC 5545.
fn plus_monate(d: time::Date, n: i64) -> Option<time::Date> {
    let gesamt = i64::from(d.year()) * 12 + i64::from(d.month() as u8 - 1) + n;
    let jahr = i32::try_from(gesamt.div_euclid(12)).ok()?;
    let monat = u8::try_from(gesamt.rem_euclid(12) + 1).ok()?;
    time::Date::from_calendar_date(jahr, time::Month::try_from(monat).ok()?, d.day()).ok()
}

/// Anfang der Woche, in der `d` liegt, gemessen an `wochenstart` (0 = Montag).
fn wochen_anker(d: time::Date, wochenstart: u8) -> Option<time::Date> {
    let versatz =
        (i64::from(d.weekday().number_days_from_monday()) + 7 - i64::from(wochenstart)) % 7;
    d.checked_sub(time::Duration::days(versatz))
}

/// Ganze Monate zwischen zwei Daten, für den Sprung ans Fenster.
fn monate_zwischen(a: time::Date, b: time::Date) -> i64 {
    (i64::from(b.year()) - i64::from(a.year())) * 12
        + (i64::from(b.month() as u8) - i64::from(a.month() as u8))
}

/// Alle Vorkommen eines Termins im Fenster `[von, bis]`, beide `JJJJ-MM-TT` und
/// einschließlich. Ohne Wiederholungsregel ist das der Termin selbst.
///
/// Regeln, die Lotse nicht ausrechnet (`genau == false`), liefern nur das erste
/// Vorkommen – die Oberfläche sagt dann, dass sich der Termin wiederholt, statt ein
/// falsches Datum zu behaupten.
pub fn vorkommen(t: &Termin, von: &str, bis: &str) -> Vec<Termin> {
    fn nimm(out: &mut Vec<Termin>, t: &Termin, tag: time::Date, von: &str, bis: &str) {
        let datum = datum_schreiben(tag);
        if datum.as_str() >= von && datum.as_str() <= bis {
            out.push(Termin { datum, ..t.clone() });
        }
    }

    let mut out: Vec<Termin> = Vec::new();
    let einzeln = |out: &mut Vec<Termin>| {
        if t.datum.as_str() >= von && t.datum.as_str() <= bis {
            out.push(t.clone());
        }
    };

    let Some(r) = t.wiederholung.as_ref().filter(|r| r.genau) else {
        einzeln(&mut out);
        return out;
    };
    let (Some(start), Some(fenster_anfang), Some(fenster_ende)) =
        (datum_lesen(&t.datum), datum_lesen(von), datum_lesen(bis))
    else {
        einzeln(&mut out);
        return out;
    };
    // `UNTIL` und das Fensterende – was zuerst kommt.
    let ende = match r.bis.as_deref().and_then(datum_lesen) {
        Some(u) if u < fenster_ende => u,
        _ => fenster_ende,
    };
    let mut wochentage = r.wochentage.clone();
    wochentage.sort_unstable();
    wochentage.dedup();

    let tage = |n: i64| time::Duration::days(n);
    let intervall = i64::from(r.intervall.max(1));

    // Ohne `COUNT` wird bis ans Fenster vorgesprungen. Sonst zählte eine tägliche Reihe,
    // die vor Jahren begann, ihr Sicherheitsnetz auf, lange bevor sie beim Fenster
    // ankäme – und der Termin verschwände ganz.
    //
    // Mit `COUNT` wird von vorn gezählt, denn `COUNT` zählt ab dem ersten Vorkommen
    // (RFC 5545) und muss deshalb auch die Vorkommen vor dem Fenster mitzählen.
    let erster_schritt = if r.anzahl.is_some() {
        0
    } else {
        let abstand = match r.takt {
            Takt::Taeglich => (fenster_anfang - start).whole_days() / intervall,
            Takt::Woechentlich => (fenster_anfang - start).whole_days() / (7 * intervall),
            Takt::Monatlich => monate_zwischen(start, fenster_anfang) / intervall,
            Takt::Jaehrlich => monate_zwischen(start, fenster_anfang) / (12 * intervall),
        };
        // Einen Schritt zurück, damit an der Kante nichts verloren geht.
        (abstand - 1).max(0)
    };

    let grenze = r.anzahl.map(|n| n as usize).unwrap_or(usize::MAX);
    let mut gezaehlt = 0usize;

    for schritt in erster_schritt..erster_schritt.saturating_add(MAX_SCHRITTE) {
        if gezaehlt >= grenze || out.len() >= MAX_VORKOMMEN {
            break;
        }
        let versatz = intervall * schritt;
        // `anker` entscheidet über den Abbruch, `treffer` sind die Tage dieses Schritts.
        // Bei Monats- und Jahrestakt kann ein Schritt ausfallen (31. Februar); dann ist
        // die Reihe nicht zu Ende, nur dieser Schritt.
        let (anker, treffer): (Option<time::Date>, Vec<time::Date>) = match r.takt {
            Takt::Taeglich => {
                let d = start.checked_add(tage(versatz));
                (d, d.into_iter().collect())
            }
            Takt::Woechentlich if wochentage.is_empty() => {
                let d = start.checked_add(tage(versatz * 7));
                (d, d.into_iter().collect())
            }
            Takt::Woechentlich => {
                // Mit BYDAY zählt die Woche, nicht der einzelne Tag: erst zum Anfang
                // dieser Woche (WKST), dann die genannten Wochentage.
                let woche = start
                    .checked_add(tage(versatz * 7))
                    .and_then(|d| wochen_anker(d, r.wochenstart));
                let treffer = woche
                    .map(|w| {
                        wochentage
                            .iter()
                            .filter_map(|n| {
                                let ab_wochenstart =
                                    (i64::from(*n) + 7 - i64::from(r.wochenstart)) % 7;
                                w.checked_add(tage(ab_wochenstart))
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                (woche, treffer)
            }
            Takt::Monatlich => {
                let d = plus_monate(start, versatz);
                (d, d.into_iter().collect())
            }
            Takt::Jaehrlich => {
                let d = plus_monate(start, versatz * 12);
                (d, d.into_iter().collect())
            }
        };
        if anker.is_some_and(|a| a > ende) {
            break;
        }
        // Innerhalb eines Schritts der Reihe nach: mit WKST kann der erste Wochentag
        // ein Sonntag sein, der vor dem Montag liegt.
        let mut treffer = treffer;
        treffer.sort_unstable();
        for tag in treffer {
            // BYDAY kann Tage vor dem Beginn erzeugen; die zählen nicht mit.
            if tag < start || tag > ende {
                continue;
            }
            if gezaehlt >= grenze {
                break;
            }
            gezaehlt += 1;
            nimm(&mut out, t, tag, von, bis);
        }
    }
    out
}

/// `JJJJ-MM-TT` um `tage` verschoben. Bei unlesbarem Datum bleibt es, wie es war –
/// ein Fenster, das zusammenfällt, ist besser als ein erfundenes Datum.
pub fn tage_spaeter(datum: &str, tage: i64) -> String {
    match datum_lesen(datum).and_then(|d| d.checked_add(time::Duration::days(tage))) {
        Some(d) => datum_schreiben(d),
        None => datum.to_string(),
    }
}

/// Alle Termine im Fenster, Wiederholungen ausgerechnet, nach Zeit sortiert.
pub fn kommende(termine: &[Termin], von: &str, bis: &str, hoechstens: usize) -> Vec<Termin> {
    let mut out: Vec<Termin> = termine
        .iter()
        .flat_map(|t| vorkommen(t, von, bis))
        .collect();
    out.sort_by_key(|t| t.schluessel());
    out.truncate(hoechstens);
    out
}

// ------------------------------------------------------------------ Holen

/// Erkennt eine Kalenderadresse: `webcal://…`, `…​.ics` oder ein lokaler Pfad auf eine
/// `.ics`-Datei. Alles andere ist kein Kalender.
pub fn ist_kalender(ziel: &str) -> bool {
    let z = ziel.trim().to_ascii_lowercase();
    let ohne_frage = z.split(['?', '#']).next().unwrap_or(&z);
    z.starts_with("webcal://") || ohne_frage.ends_with(".ics")
}

/// Holt einen Kalender. `webcal://` ist https, nur mit anderem Namen. Lokale Pfade
/// werden gelesen, nicht geholt.
#[cfg(feature = "native")]
pub fn holen(ziel: &str) -> crate::Result<String> {
    let z = ziel.trim();
    if !z.starts_with("http://") && !z.starts_with("https://") && !z.starts_with("webcal://") {
        return std::fs::read_to_string(z).map_err(crate::Error::Io);
    }
    let url = match z.strip_prefix("webcal://") {
        Some(rest) => format!("https://{rest}"),
        None => z.to_string(),
    };
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent(concat!("lotse/", env!("CARGO_PKG_VERSION")))
        .build();
    match agent.get(&url).call() {
        Ok(r) => r
            .into_string()
            .map_err(|e| crate::Error::Netz(e.to_string())),
        Err(ureq::Error::Status(401 | 403, _)) => Err(crate::Error::Invalid(
            "Der Kalender ist nicht öffentlich. Bei den meisten Diensten gibt es dafür \
             eine geheime Abonnement-Adresse."
                .into(),
        )),
        Err(ureq::Error::Status(404, _)) => {
            Err(crate::Error::NotFound("Kalender nicht gefunden".into()))
        }
        Err(ureq::Error::Status(s, _)) => Err(crate::Error::Netz(format!("Antwort {s}"))),
        Err(e) => Err(crate::Error::Netz(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn daten(name: &str) -> String {
        std::fs::read_to_string(format!("{}/tests/daten/{name}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("{name}: {e}"))
    }

    #[test]
    fn liest_ganztaegige_termine_eines_echten_kalenders() {
        // Unveränderte Einträge von officeholidays.com: ganztägig, mit Parametern am
        // SUMMARY (`;LANGUAGE=en-us`) und maskierten Zeilenumbrüchen in der Beschreibung.
        let t = lesen(&daten("ics_officeholidays.ics"));
        assert_eq!(t.len(), 3);
        assert_eq!(t[0].datum, "2026-01-01");
        assert!(t[0].titel.contains("New Year's Day"), "{}", t[0].titel);
        assert_eq!(t[0].uhrzeit, None);
        assert_eq!(t[0].ort.as_deref(), Some("Germany"));
    }

    #[test]
    fn haelt_gefaltete_zeilen_zusammen() {
        // Google faltet lange Zeilen mitten im Wort. Falsch zusammengesetzt stünde in
        // der Beschreibung „Setting s“ – hier zählt, dass die Termine trotzdem stimmen.
        let ics = daten("ics_google.ics");
        assert!(ics.contains("\r\n "), "Fixture ohne gefaltete Zeile");
        let t = lesen(&ics);
        assert_eq!(t.len(), 3);
        assert_eq!(t[0].datum, "2021-02-17");
        assert_eq!(t[0].titel, "Carnival / Ash Wednesday");
    }

    #[test]
    fn liest_uhrzeit_ort_und_maskierung() {
        let t = lesen(&daten("ics_beispiel.ics"));
        let erster = t.iter().find(|t| t.titel.starts_with("Fundament")).unwrap();
        assert_eq!(erster.datum, "2026-09-10");
        assert_eq!(erster.uhrzeit.as_deref(), Some("10:00"));
        // TZID wird nicht umgerechnet: zehn Uhr bleibt zehn Uhr.
        assert!(!erster.utc);
        assert_eq!(erster.titel, "Fundament abnehmen, mit Statiker");
        assert_eq!(erster.ort.as_deref(), Some("Baustelle, hinten"));
        assert_eq!(erster.anzeige(), "2026-09-10 10:00");

        // Der VALARM im Termin darf den Titel nicht überschreiben.
        assert!(t.iter().all(|t| t.titel != "Erinnerung"));
        // Und die VTIMEZONE-Komponente ist kein Termin.
        assert!(t.iter().all(|t| t.titel != "Sommerzeit"));
    }

    #[test]
    fn kennzeichnet_utc_statt_zu_raten() {
        let t = lesen(&daten("ics_beispiel.ics"));
        let steuer = t.iter().find(|t| t.titel.starts_with("Steuer")).unwrap();
        assert!(steuer.utc);
        assert_eq!(steuer.anzeige(), "2026-09-15 06:30 UTC");
    }

    #[test]
    fn rechnet_zweiwoechentlich_mit_count_aus() {
        let t = lesen(&daten("ics_beispiel.ics"));
        let kurs = t.iter().find(|t| t.titel == "Werkstattkurs").unwrap();
        let v: Vec<String> = vorkommen(kurs, "2026-01-01", "2027-12-31")
            .into_iter()
            .map(|t| t.datum)
            .collect();
        assert_eq!(v, ["2026-09-19", "2026-10-03", "2026-10-17"]);
    }

    #[test]
    fn rechnet_wochentage_bis_zum_stichtag_aus() {
        let t = lesen(&daten("ics_beispiel.ics"));
        let probe = t.iter().find(|t| t.titel == "Bandprobe").unwrap();
        let v: Vec<String> = vorkommen(probe, "2026-01-01", "2027-12-31")
            .into_iter()
            .map(|t| t.datum)
            .collect();
        // 1.9.2026 ist ein Dienstag; UNTIL ist der 30.9.
        assert_eq!(v.first().map(String::as_str), Some("2026-09-01"));
        assert_eq!(v.last().map(String::as_str), Some("2026-09-29"));
        assert_eq!(v.len(), 9);
        assert!(v.contains(&"2026-09-03".to_string()));
    }

    #[test]
    fn rechnet_jaehrliches_nicht_ueber_das_fenster_hinaus() {
        let t = lesen(&daten("ics_beispiel.ics"));
        let steuer = t.iter().find(|t| t.titel.starts_with("Steuer")).unwrap();
        let v: Vec<String> = vorkommen(steuer, "2026-01-01", "2029-01-01")
            .into_iter()
            .map(|t| t.datum)
            .collect();
        assert_eq!(v, ["2026-09-15", "2027-09-15", "2028-09-15"]);
    }

    #[test]
    fn behauptet_nichts_bei_regeln_die_es_nicht_ausrechnet() {
        let t = lesen(&daten("ics_beispiel.ics"));
        let unklar = t.iter().find(|t| t.titel.starts_with("Zweiter")).unwrap();
        let r = unklar.wiederholung.as_ref().unwrap();
        assert!(!r.genau, "BYDAY=2MO darf nicht als ausgerechnet gelten");
        // Nur das erste Vorkommen, kein geratenes Datum.
        let v = vorkommen(unklar, "2026-01-01", "2027-12-31");
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].datum, "2026-09-07");
    }

    #[test]
    fn monatlich_ueberspringt_fehlende_tage() {
        let t = Termin {
            titel: "Miete".into(),
            datum: "2026-01-31".into(),
            uhrzeit: None,
            utc: false,
            ort: None,
            wiederholung: Some(Regel {
                takt: Takt::Monatlich,
                intervall: 1,
                anzahl: None,
                bis: None,
                wochentage: vec![],
                wochenstart: 0,
                genau: true,
            }),
        };
        let v: Vec<String> = vorkommen(&t, "2026-01-01", "2026-05-31")
            .into_iter()
            .map(|t| t.datum)
            .collect();
        // Februar hat keinen 31.; die Reihe läuft trotzdem weiter.
        assert_eq!(v, ["2026-01-31", "2026-03-31", "2026-05-31"]);
    }

    #[test]
    fn schaltjahr_faellt_aus_statt_zu_verrutschen() {
        let t = Termin {
            titel: "Schalttag".into(),
            datum: "2024-02-29".into(),
            uhrzeit: None,
            utc: false,
            ort: None,
            wiederholung: Some(Regel {
                takt: Takt::Jaehrlich,
                intervall: 1,
                anzahl: None,
                bis: None,
                wochentage: vec![],
                wochenstart: 0,
                genau: true,
            }),
        };
        let v: Vec<String> = vorkommen(&t, "2024-01-01", "2029-12-31")
            .into_iter()
            .map(|t| t.datum)
            .collect();
        assert_eq!(v, ["2024-02-29", "2028-02-29"]);
    }

    /// Eine tägliche Reihe ohne COUNT, die vor Jahren begann, muss heute noch
    /// auftauchen. Vorher zählte das Sicherheitsnetz die Vorkommen vor dem Fenster mit
    /// und brach ab, bevor es überhaupt beim Fenster ankam – der Termin verschwand.
    #[test]
    fn alte_reihe_ohne_count_taucht_noch_auf() {
        let t = lesen("BEGIN:VEVENT\r\nDTSTART:20180101T090000\r\nSUMMARY:Täglich\r\nRRULE:FREQ=DAILY\r\nEND:VEVENT\r\n");
        let v = vorkommen(&t[0], "2026-09-07", "2026-09-09");
        let daten: Vec<String> = v.into_iter().map(|t| t.datum).collect();
        assert_eq!(daten, ["2026-09-07", "2026-09-08", "2026-09-09"]);

        // Auch wöchentlich, auch monatlich, auch mit Intervall.
        let w = lesen("BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20180101\r\nSUMMARY:Woche\r\nRRULE:FREQ=WEEKLY;INTERVAL=2\r\nEND:VEVENT\r\n");
        assert!(!vorkommen(&w[0], "2026-09-01", "2026-09-30").is_empty());
        let m = lesen("BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20000115\r\nSUMMARY:Monat\r\nRRULE:FREQ=MONTHLY\r\nEND:VEVENT\r\n");
        assert_eq!(
            vorkommen(&m[0], "2026-09-01", "2026-09-30")
                .into_iter()
                .map(|t| t.datum)
                .collect::<Vec<_>>(),
            ["2026-09-15"]
        );
    }

    /// `COUNT` zählt ab dem ersten Vorkommen, auch über das Fenster hinaus: eine Reihe
    /// mit `COUNT=3` ist nach dem dritten Termin zu Ende und darf später nicht wieder
    /// auftauchen.
    #[test]
    fn count_zaehlt_ab_dem_ersten_vorkommen() {
        let t = lesen("BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20260101\r\nSUMMARY:Dreimal\r\nRRULE:FREQ=DAILY;COUNT=3\r\nEND:VEVENT\r\n");
        assert_eq!(vorkommen(&t[0], "2026-01-01", "2026-12-31").len(), 3);
        // Das Fenster fängt nach dem dritten Vorkommen an: nichts mehr.
        assert!(vorkommen(&t[0], "2026-02-01", "2026-12-31").is_empty());
    }

    /// `WKST` entscheidet bei `INTERVAL > 1`, welche Tage noch zur selben Woche gehören.
    /// Ohne Beachtung lagen die Termine hinter dem Wochenstart eine Woche zu spät.
    #[test]
    fn beachtet_den_wochenstart() {
        // 2026-09-06 ist ein Sonntag. Mit WKST=SU gehören Sonntag und der folgende
        // Montag in dieselbe Woche.
        let t = lesen("BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20260906\r\nSUMMARY:Wkst\r\nRRULE:FREQ=WEEKLY;INTERVAL=2;BYDAY=SU,MO;WKST=SU\r\nEND:VEVENT\r\n");
        let daten: Vec<String> = vorkommen(&t[0], "2026-09-01", "2026-10-15")
            .into_iter()
            .map(|t| t.datum)
            .collect();
        assert_eq!(
            daten,
            [
                "2026-09-06",
                "2026-09-07",
                "2026-09-20",
                "2026-09-21",
                "2026-10-04",
                "2026-10-05"
            ]
        );

        // Ohne WKST bleibt es beim Montag als Wochenanfang.
        let ohne = lesen("BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20260907\r\nSUMMARY:Ohne\r\nRRULE:FREQ=WEEKLY;INTERVAL=2;BYDAY=MO,WE\r\nEND:VEVENT\r\n");
        let daten: Vec<String> = vorkommen(&ohne[0], "2026-09-01", "2026-09-30")
            .into_iter()
            .map(|t| t.datum)
            .collect();
        assert_eq!(
            daten,
            ["2026-09-07", "2026-09-09", "2026-09-21", "2026-09-23"]
        );
    }

    #[test]
    fn kommende_sortiert_und_begrenzt() {
        let alle = lesen(&daten("ics_beispiel.ics"));
        let k = kommende(&alle, "2026-09-01", "2026-09-30", 5);
        assert_eq!(k.len(), 5);
        assert_eq!(k[0].datum, "2026-09-01");
        // Aufsteigend, ganztägig vor Uhrzeit desselben Tages.
        let mut sortiert = k.clone();
        sortiert.sort_by_key(|t| t.schluessel());
        assert_eq!(k, sortiert);
        // Nichts außerhalb des Fensters.
        assert!(k.iter().all(|t| t.datum.as_str() <= "2026-09-30"));
    }

    #[test]
    fn verschiebt_daten_ueber_monatsgrenzen() {
        assert_eq!(tage_spaeter("2026-09-07", 90), "2026-12-06");
        assert_eq!(tage_spaeter("2026-12-31", 1), "2027-01-01");
        assert_eq!(tage_spaeter("kein Datum", 5), "kein Datum");
    }

    #[test]
    fn erkennt_kalenderadressen() {
        for ziel in [
            "https://example.invalid/kalender.ics",
            "webcal://example.invalid/feed",
            "/home/du/termine.ics",
            "https://example.invalid/export.ics?token=abc",
        ] {
            assert!(ist_kalender(ziel), "{ziel}");
        }
        for ziel in [
            "https://github.com/phish3144/lotse",
            "/home/du/Projekte",
            "Keller, Regal 3",
        ] {
            assert!(!ist_kalender(ziel), "{ziel}");
        }
    }

    #[test]
    fn haelt_muell_aus() {
        // Kein Absturz, keine Panik, keine erfundenen Termine.
        assert!(lesen("").is_empty());
        assert!(lesen("BEGIN:VCALENDAR\r\nEND:VCALENDAR\r\n").is_empty());
        assert!(lesen("BEGIN:VEVENT\r\nSUMMARY:Ohne Beginn\r\nEND:VEVENT\r\n").is_empty());
        assert!(lesen("nur eine Zeile ohne Doppelpunkt").is_empty());
        let ohne_titel = lesen("BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20260101\r\nEND:VEVENT\r\n");
        assert_eq!(ohne_titel.len(), 1);
        assert_eq!(ohne_titel[0].titel, "Ohne Titel");
        // Unsinniges Datum: kein Termin, kein Fehler.
        assert!(lesen("BEGIN:VEVENT\r\nDTSTART:morgen\r\nSUMMARY:x\r\nEND:VEVENT\r\n").is_empty());
        // Auch nicht bei Zeichen, die über mehrere Bytes gehen: ein Schnitt nach dem
        // vierten Byte läge sonst mitten im „ä".
        for wert in [
            "20260101T000ä00",
            "20260101Tä",
            "2026ä101",
            "20260101T",
            "20260101TäääZ",
        ] {
            let ics = format!("BEGIN:VEVENT\r\nDTSTART:{wert}\r\nSUMMARY:x\r\nEND:VEVENT\r\n");
            assert!(lesen(&ics).is_empty(), "{wert}");
        }
        // Emoji im Titel bleibt heil.
        let mit_emoji = lesen(
            "BEGIN:VEVENT\r\nDTSTART;VALUE=DATE:20260101\r\nSUMMARY:Neujahr 🎉\r\nEND:VEVENT\r\n",
        );
        assert_eq!(mit_emoji[0].titel, "Neujahr 🎉");
    }
}
