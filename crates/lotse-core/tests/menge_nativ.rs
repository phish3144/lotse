//! Wie viel kostet der Brief bei einem großen Logbuch – nativ?
//!
//! Das Gegenstück zu `apps/web/scripts/wasm-mengentest.mjs`. Beide Messungen tragen
//! `docs/WEB_CLIENT.md` Abschnitt 4a: dort steht, dass `store::brief` alle Notizen eines
//! Vorhabens lädt und dass das bei einem großen Logbuch auch nativ der falsche Weg ist.
//! Eine Behauptung über Kosten gehört gemessen, nicht geschätzt.
//!
//! Läuft nicht in CI (Wanduhr-Zahlen gehören nicht in eine rot/grün-Prüfung):
//!
//!   cargo test --release -p lotse-core --test menge_nativ -- --ignored --nocapture
//!
//! `--release` ist nicht Bequemlichkeit: im Debug-Bau messen sich SQLCipher und die
//! Serialisierung um ein Vielfaches langsamer, und die Zahl wäre unbrauchbar.

use lotse_core::crypto::KdfParams;
use lotse_core::model::{Art, Notiz, Projekt, Quelle, Vorlage};
use lotse_core::store::Store;
use std::time::Instant;
use ulid::Ulid;

fn store_mit(n: usize) -> (tempfile::TempDir, Store, Ulid) {
    let dir = tempfile::tempdir().unwrap();
    let pfad = dir.path().join("lotse.db");
    let konto = lotse_core::crypto::konto_einrichten(b"menge", KdfParams::schnell_fuer_tests())
        .expect("Konto");
    let mut store = Store::open(&pfad, &konto.account_key, Ulid::new()).expect("Store");

    let p = Projekt::neu("Großes Logbuch", Vorlage::Software, 1_700_000_000_000);
    let projekt_id = p.id;
    store.projekt_speichern(&p).expect("Projekt");

    // In Blöcken, weil nur die *Entwicklung* der Kosten etwas verrät. Ein Mittelwert
    // verdeckt genau den Fall, der hier schon einmal gelauert hat: bis 0.10 lief bei jeder
    // Notiz ein voller Durchlauf des Suchindex, und das wächst mit dem Bestand.
    let block = (n / 5).max(1);
    let t0 = Instant::now();
    for i in 0..n {
        let art = if i % 50 == 0 { Art::Offen } else { Art::Log };
        let quelle = if i % 200 == 0 {
            Quelle::Mensch
        } else {
            Quelle::Git
        };
        let notiz = Notiz::neu(
            projekt_id,
            quelle,
            art,
            format!("Commit {i}: Datei geändert, Zeitstempel notiert"),
            1_700_000_000_000 + (i as i64) * 60_000,
        );
        let t = Instant::now();
        store.notiz_speichern(&notiz).expect("Notiz");
        let dauer = t.elapsed();
        if (i + 1) % block == 0 {
            println!(
                "    bis {:>7}: {:>6} µs für die letzte Notiz",
                i + 1,
                dauer.as_micros()
            );
        }
    }
    println!(
        "  {n} Notizen schreiben: {:?} ({} µs je Satz im Mittel)",
        t0.elapsed(),
        t0.elapsed().as_micros() / n.max(1) as u128
    );
    (dir, store, projekt_id)
}

#[test]
#[ignore = "Messung, keine Prüfung: von Hand mit --ignored --nocapture aufrufen"]
fn brief_bei_grossem_logbuch() {
    let groessen: Vec<usize> = match std::env::var("LOTSE_MENGE") {
        Ok(v) => v
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect::<Vec<usize>>(),
        Err(_) => vec![1_000, 20_000],
    };
    for n in groessen {
        println!("Bestand {n}:");
        let (_dir, store, projekt_id) = store_mit(n);

        let t = Instant::now();
        let notizen = store.notizen(projekt_id).expect("notizen");
        let laden = t.elapsed();

        let t = Instant::now();
        let brief = store.brief(projekt_id, 1_800_000_000_000).expect("brief");
        let ganz = t.elapsed();

        println!("  alle Notizen laden:  {laden:?} ({} Sätze)", notizen.len());
        println!("  store::brief:        {ganz:?}");
        println!(
            "  offene Fäden:        {} (der Brief braucht nur diese plus die letzten)",
            brief.offene_faeden.len()
        );
    }
}
