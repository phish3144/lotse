//! Der "Wo war ich"-Brief und die Auffälligkeit eines Projekts (`docs/CONCEPT.md`, 4 und 5).
//!
//! Reine Funktionen über Modelldaten, damit Desktop, Web und CLI dieselbe Logik nutzen.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{Art, Notiz, Projekt, Quelle, Status};

pub const MS_PRO_TAG: i64 = 24 * 60 * 60 * 1000;

/// Ruhende Projekte sind ruhig. Rot wird nur, wer sein eigenes Erwartungsintervall reißt
/// oder dessen Wiedervorlage verstrichen ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Auffaelligkeit {
    Ruhig,
    Auffaellig,
    Ueberfaellig,
}

/// Tage seit dem letzten Kontakt (letzte Notiz oder `zuletzt_beruehrt`).
pub fn tage_seit(projekt: &Projekt, letzte_notiz_ts: Option<i64>, jetzt_ms: i64) -> i64 {
    let letzter = letzte_notiz_ts
        .unwrap_or(projekt.zuletzt_beruehrt)
        .max(projekt.angelegt);
    ((jetzt_ms - letzter).max(0)) / MS_PRO_TAG
}

/// `JJJJ-MM-TT` → Unix-Millisekunden um Mitternacht UTC.
pub fn datum_zu_ms(datum: &str) -> Option<i64> {
    let format = time::macros::format_description!("[year]-[month]-[day]");
    let d = time::Date::parse(datum, &format).ok()?;
    Some(d.midnight().assume_utc().unix_timestamp() * 1000)
}

pub fn auffaelligkeit(
    projekt: &Projekt,
    letzte_notiz_ts: Option<i64>,
    jetzt_ms: i64,
) -> Auffaelligkeit {
    // Wiedervorlage verstrichen: immer überfällig, egal welcher Status.
    if let Some(wv) = projekt.wiedervorlage.as_deref().and_then(datum_zu_ms) {
        if jetzt_ms >= wv + MS_PRO_TAG {
            return Auffaelligkeit::Ueberfaellig;
        }
    }
    match projekt.status {
        Status::Aktiv => {
            let tage = tage_seit(projekt, letzte_notiz_ts, jetzt_ms);
            let intervall = projekt.erwartungsintervall_tage as i64;
            if tage > intervall * 2 {
                Auffaelligkeit::Ueberfaellig
            } else if tage > intervall {
                Auffaelligkeit::Auffaellig
            } else {
                Auffaelligkeit::Ruhig
            }
        }
        _ => Auffaelligkeit::Ruhig,
    }
}

/// Soll beim Öffnen der Brief gezeigt werden?
pub fn brief_faellig(projekt: &Projekt, letzte_notiz_ts: Option<i64>, jetzt_ms: i64) -> bool {
    tage_seit(projekt, letzte_notiz_ts, jetzt_ms) > projekt.erwartungsintervall_tage as i64
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Brief {
    pub tage_seit: i64,
    /// Wortlaut der letzten Übergabenotiz.
    pub letzte_uebergabe: Option<String>,
    /// Wortlaut der letzten Notiz überhaupt (falls keine Übergabe).
    pub letzte_notiz: Option<String>,
    pub offene_faeden: Vec<Notiz>,
    /// Aktivität seit dem letzten menschlichen Kontakt, gezählt je Quelle.
    pub aktivitaet_seit_letztem_besuch: BTreeMap<String, usize>,
    pub auffaelligkeit: Auffaelligkeit,
}

/// Baut den Brief aus den Notizen eines Projekts (beliebige Reihenfolge).
pub fn brief(projekt: &Projekt, notizen: &[Notiz], jetzt_ms: i64) -> Brief {
    let mut sortiert: Vec<&Notiz> = notizen.iter().collect();
    sortiert.sort_by_key(|n| std::cmp::Reverse(n.ts));

    let letzter_mensch_ts = sortiert
        .iter()
        .find(|n| matches!(n.quelle, Quelle::Mensch | Quelle::Cli))
        .map(|n| n.ts);
    let letzte_ts = sortiert.first().map(|n| n.ts);

    let letzte_uebergabe = sortiert
        .iter()
        .find(|n| n.art == Art::Uebergabe)
        .map(|n| n.text.clone());
    let letzte_notiz = sortiert.first().map(|n| n.text.clone());

    let offene_faeden: Vec<Notiz> = sortiert
        .iter()
        .filter(|n| n.ist_offen())
        .map(|n| (*n).clone())
        .collect();

    let mut aktivitaet = BTreeMap::new();
    if let Some(grenze) = letzter_mensch_ts {
        for n in &sortiert {
            if n.ts > grenze && !matches!(n.quelle, Quelle::Mensch | Quelle::Cli) {
                *aktivitaet.entry(n.quelle.as_str().to_string()).or_insert(0) += 1;
            }
        }
    }

    Brief {
        tage_seit: tage_seit(projekt, letzter_mensch_ts.or(letzte_ts), jetzt_ms),
        letzte_uebergabe,
        letzte_notiz,
        offene_faeden,
        aktivitaet_seit_letztem_besuch: aktivitaet,
        auffaelligkeit: auffaelligkeit(projekt, letzte_ts, jetzt_ms),
    }
}

/// Vorbefüllung für die Übergabenotiz beim Pausieren: offene Fäden als Liste plus ein
/// Hinweis auf Aktivität, damit man bestätigt statt tippt.
pub fn uebergabe_vorschlag(brief: &Brief) -> String {
    let mut s = String::new();
    if brief.offene_faeden.is_empty() {
        s.push_str("Nichts Neues, siehe letzte Notiz.");
    } else {
        s.push_str("Offen:\n");
        for f in &brief.offene_faeden {
            s.push_str("- ");
            s.push_str(f.text.lines().next().unwrap_or(""));
            s.push('\n');
        }
    }
    if !brief.aktivitaet_seit_letztem_besuch.is_empty() {
        s.push_str("\nSeit dem letzten Besuch: ");
        let teile: Vec<String> = brief
            .aktivitaet_seit_letztem_besuch
            .iter()
            .map(|(q, n)| format!("{n} × {q}"))
            .collect();
        s.push_str(&teile.join(", "));
    }
    s.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Projekt, Vorlage};
    use ulid::Ulid;

    fn projekt() -> Projekt {
        let mut p = Projekt::neu("Test", Vorlage::Software, 0);
        p.zuletzt_beruehrt = 0;
        p
    }

    #[test]
    fn auffaelligkeit_nach_intervall() {
        let p = projekt();
        assert_eq!(
            auffaelligkeit(&p, Some(0), 10 * MS_PRO_TAG),
            Auffaelligkeit::Ruhig
        );
        assert_eq!(
            auffaelligkeit(&p, Some(0), 20 * MS_PRO_TAG),
            Auffaelligkeit::Auffaellig
        );
        assert_eq!(
            auffaelligkeit(&p, Some(0), 40 * MS_PRO_TAG),
            Auffaelligkeit::Ueberfaellig
        );
    }

    #[test]
    fn pausiert_ist_ruhig_ausser_wiedervorlage() {
        let mut p = projekt();
        p.status = Status::Pausiert;
        assert_eq!(
            auffaelligkeit(&p, Some(0), 400 * MS_PRO_TAG),
            Auffaelligkeit::Ruhig
        );
        p.wiedervorlage = Some("1970-01-10".into());
        assert_eq!(
            auffaelligkeit(&p, Some(0), 5 * MS_PRO_TAG),
            Auffaelligkeit::Ruhig
        );
        assert_eq!(
            auffaelligkeit(&p, Some(0), 12 * MS_PRO_TAG),
            Auffaelligkeit::Ueberfaellig
        );
    }

    #[test]
    fn brief_zaehlt_aktivitaet_seit_mensch() {
        let p = projekt();
        let pid = Ulid::new();
        let notizen = vec![
            Notiz::neu(
                pid,
                Quelle::Mensch,
                Art::Uebergabe,
                "Beton bestellt. Nächster Schritt: Schalung.",
                10,
            ),
            Notiz::neu(pid, Quelle::Mensch, Art::Offen, "Bewehrung nötig?", 11),
            Notiz::neu(pid, Quelle::Git, Art::Log, "3 Commits", 20),
            Notiz::neu(pid, Quelle::Datei, Art::Log, "5 Dateien", 21),
            Notiz::neu(pid, Quelle::Datei, Art::Log, "2 Dateien", 22),
        ];
        let b = brief(&p, &notizen, 30 * MS_PRO_TAG);
        assert_eq!(
            b.letzte_uebergabe.as_deref(),
            Some("Beton bestellt. Nächster Schritt: Schalung.")
        );
        assert_eq!(b.offene_faeden.len(), 1);
        assert_eq!(b.aktivitaet_seit_letztem_besuch.get("git"), Some(&1));
        assert_eq!(b.aktivitaet_seit_letztem_besuch.get("datei"), Some(&2));
        let v = uebergabe_vorschlag(&b);
        assert!(v.contains("Bewehrung"));
        assert!(v.contains("2 × datei"));
    }

    #[test]
    fn datum() {
        assert_eq!(datum_zu_ms("1970-01-02"), Some(MS_PRO_TAG));
        assert_eq!(datum_zu_ms("kaputt"), None);
    }
}
