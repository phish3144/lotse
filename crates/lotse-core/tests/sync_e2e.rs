//! End-to-End-Test gegen einen laufenden Sync-Dienst (`wrangler dev`).
//!
//! Läuft nur, wenn `LOTSE_SYNC_URL` gesetzt ist, z. B. `http://127.0.0.1:8787`.
//! Start: `scripts/sync-e2e.sh`.

use lotse_core::crypto::{self, KdfParams, Key32};
use lotse_core::model::*;
use lotse_core::now_ms;
use lotse_core::store::Store;
use lotse_core::sync::client::{self, Client, Geraet};
use lotse_core::vault::{TresorEintrag, VaultKeys};
use ulid::Ulid;

fn url() -> Option<String> {
    std::env::var("LOTSE_SYNC_URL")
        .ok()
        .filter(|s| !s.is_empty())
}

fn geraet(name: &str) -> Geraet {
    Geraet {
        id: Ulid::new(),
        name: name.into(),
        platform: "cli".into(),
    }
}

#[test]
fn zwei_geraete_gleichen_ab() {
    let Some(url) = url() else {
        eprintln!("LOTSE_SYNC_URL nicht gesetzt, Test übersprungen");
        return;
    };
    let email = format!(
        "test-{}@example.invalid",
        Ulid::new().to_string().to_lowercase()
    );
    let passwort = b"korrekt batterie pferd";

    // Gerät A richtet das Konto ein und registriert es.
    let konto = crypto::konto_einrichten(passwort, KdfParams::schnell_fuer_tests()).unwrap();
    let ga = geraet("A");
    let mut a = Store::open_in_memory(&konto.account_key, ga.id).unwrap();
    let anon = Client::new(&url);
    let (account_id, token_a) = anon
        .register(
            &email,
            &konto.auth_key,
            &konto.recovery_auth_key,
            &konto.header,
            &ga,
        )
        .unwrap();
    client::verbindung_merken(&a, &url, &email, &account_id, &token_a).unwrap();
    let ca = client::client_aus_store(&a).unwrap();

    // Doppelte Registrierung wird abgelehnt.
    let err = anon
        .register(
            &email,
            &konto.auth_key,
            &konto.recovery_auth_key,
            &konto.header,
            &ga,
        )
        .unwrap_err();
    assert!(
        matches!(err, lotse_core::Error::Sync { status: 409, .. }),
        "{err}"
    );

    // Daten auf A.
    let p = Projekt::neu("Gartenhaus", Vorlage::HausGarten, now_ms());
    a.projekt_speichern(&p).unwrap();
    a.notiz_speichern(&Notiz::neu(
        p.id,
        Quelle::Cli,
        Art::Offen,
        "Bewehrung?",
        now_ms(),
    ))
    .unwrap();
    let vk = VaultKeys::from_account_key(&konto.account_key);
    let e = TresorEintrag::neu(
        &vk,
        "Router",
        vec![p.id],
        Stufe::Ueberall,
        &[("pw", "geheim")],
        now_ms(),
    )
    .unwrap();
    a.tresor_speichern(&e).unwrap();

    let r = client::abgleichen(&mut a, &ca).unwrap();
    assert_eq!(r.gepusht, 3);
    assert_eq!(r.abgelehnt, 0);
    assert_eq!(a.ausstehend().unwrap(), 0);

    // Gerät B meldet sich mit Passwort an, ohne lokales Konto.
    let gb = geraet("B");
    let pre = anon.prelogin(&email).unwrap();
    let stretched = crypto::derive_stretched(passwort, &pre.salt, &pre.kdf).unwrap();
    let pk = crypto::split_password_keys(&stretched);
    let login = anon.login(&email, &pk.auth, &gb).unwrap();
    let (ak_b, _) = crypto::konto_entsperren(&login.header, passwort).unwrap();
    assert_eq!(ak_b.as_bytes(), konto.account_key.as_bytes());

    let mut b = Store::open_in_memory(&ak_b, gb.id).unwrap();
    client::verbindung_merken(&b, &url, &email, &login.account_id, &login.session_token).unwrap();
    let cb = client::client_aus_store(&b).unwrap();
    let r = client::abgleichen(&mut b, &cb).unwrap();
    assert_eq!(r.uebernommen, 3);
    assert_eq!(b.projekte().unwrap()[0].titel, "Gartenhaus");
    assert_eq!(b.offene_faeden(None).unwrap().len(), 1);
    let eb = b.tresor_eintrag(e.id).unwrap().unwrap();
    assert_eq!(
        eb.feld_lesen(&VaultKeys::from_account_key(&ak_b), "pw")
            .unwrap()
            .as_str(),
        "geheim"
    );

    // B schreibt, A holt. Eigene Umschläge kommen zurück und werden still verworfen.
    b.notiz_speichern(&Notiz::neu(p.id, Quelle::Cli, Art::Log, "von B", now_ms()))
        .unwrap();
    client::abgleichen(&mut b, &cb).unwrap();
    let r = client::abgleichen(&mut a, &ca).unwrap();
    assert_eq!(r.uebernommen, 1);
    assert!(a.notizen(p.id).unwrap().iter().any(|n| n.text == "von B"));

    // Konflikt am Projektkopf: beide ändern, B zuletzt. A bekommt B's Fassung plus Konfliktnotiz.
    let mut pa = a.projekt(p.id).unwrap().unwrap();
    pa.titel = "Gartenhaus A".into();
    a.projekt_speichern(&pa).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let mut pb = b.projekt(p.id).unwrap().unwrap();
    pb.titel = "Gartenhaus B".into();
    b.projekt_speichern(&pb).unwrap();
    client::abgleichen(&mut a, &ca).unwrap();
    client::abgleichen(&mut b, &cb).unwrap(); // B pusht seine neuere Fassung, holt A's ältere: verworfen
    assert_eq!(b.projekt(p.id).unwrap().unwrap().titel, "Gartenhaus B");
    let r = client::abgleichen(&mut a, &ca).unwrap();
    assert_eq!(r.konflikte, 1);
    assert_eq!(a.projekt(p.id).unwrap().unwrap().titel, "Gartenhaus B");
    assert!(a
        .notizen(p.id)
        .unwrap()
        .iter()
        .any(|n| n.quelle == Quelle::Sync));

    // Löschen wandert als Tombstone.
    a.tresor_loeschen(e.id).unwrap();
    client::abgleichen(&mut a, &ca).unwrap();
    client::abgleichen(&mut b, &cb).unwrap();
    assert!(b.tresor_eintrag(e.id).unwrap().is_none());

    // Geräte sichtbar, Widerruf wirkt.
    let geraete = ca.geraete().unwrap();
    assert_eq!(geraete.len(), 2);
    ca.geraet_widerrufen(&gb.id.to_string()).unwrap();
    let err = cb.status().unwrap_err();
    assert!(
        matches!(err, lotse_core::Error::Sync { status: 401, .. }),
        "{err}"
    );

    // Der Dienst hat nie Klartext gesehen: Status zählt nur Datensätze.
    let st = ca.status().unwrap();
    assert!(st.record_count >= 5);
    let _ = Key32::random();
}
