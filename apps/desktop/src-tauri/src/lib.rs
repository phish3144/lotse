//! Tauri-Hülle um `lotse-core`.
//!
//! Die Kommandos spiegeln `apps/web/src/lib/data/provider.ts`. Nach dem Entsperren hält der
//! App-State Speicher und Schlüssel; Schlüsselmaterial verlässt den Rust-Prozess nie, die
//! Oberfläche sieht nur entsperrte Einzelwerte auf Anfrage.

use std::path::PathBuf;
use std::sync::Mutex;

use lotse_core::brief::Brief;
use lotse_core::crypto::{KdfParams, Key32, RecoveryCode};
use lotse_core::konto;
use lotse_core::model::*;
use lotse_core::store::{HafenKarte, Store, Treffer};
use lotse_core::sync::client as sync_client;
use lotse_core::vault::{TresorEintrag, VaultKeys};
use lotse_core::{now_ms, Error};
use serde::Serialize;
use tauri::{Manager, State};
use ulid::Ulid;
use zeroize::Zeroizing;

struct Sitzung {
    store: Store,
    /// Für Passwortwechsel und Export; bis dahin nur gehalten.
    #[allow(dead_code)]
    account_key: Key32,
    auth_key: Key32,
    vault: VaultKeys,
    geraet_id: Ulid,
    geraet_name: String,
}

/// Laufender Ordner-Beobachter. Der Thread hält den `Beobachter` selbst; hier steht nur
/// der Schalter, mit dem er sich beenden lässt.
struct BeobachterHandle {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Default)]
pub struct AppState {
    home: Mutex<Option<PathBuf>>,
    sitzung: Mutex<Option<Sitzung>>,
    beobachter: Mutex<Option<BeobachterHandle>>,
    /// Zuletzt geholte Kalender: Adresse → (Zeitpunkt, Inhalt). Nur im Arbeitsspeicher.
    kalender: Mutex<std::collections::HashMap<String, (i64, String)>>,
}

type R<T> = Result<T, String>;

fn fehler(e: Error) -> String {
    e.to_string()
}

/// Nimmt eine Sperre, auch wenn sie vergiftet ist.
///
/// Vergiftet wird sie, wenn ein Thread beim Halten abstürzt. Rust hält die Sperre
/// danach dauerhaft geschlossen – das würde die App bis zum Neustart unbrauchbar
/// machen, und zwar mit der Meldung „Nicht entsperrt", die nach einer gesperrten
/// Sitzung aussieht statt nach einem Fehler. Dahinter steht eine SQLite-Verbindung;
/// halbfertige Schreibvorgänge macht die selbst rückgängig, der Schlüssel im Speicher
/// bleibt gültig. Weiterarbeiten ist hier die bessere Antwort als aussperren.
fn sperre<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|vergiftet| vergiftet.into_inner())
}

fn home(state: &State<AppState>) -> R<PathBuf> {
    sperre(&state.home)
        .clone()
        .ok_or_else(|| "Datenordner unbekannt".to_string())
}

fn mit<T>(state: &State<AppState>, f: impl FnOnce(&mut Sitzung) -> lotse_core::Result<T>) -> R<T> {
    let mut guard = sperre(&state.sitzung);
    let s = guard
        .as_mut()
        .ok_or_else(|| "Nicht entsperrt".to_string())?;
    f(s).map_err(fehler)
}

fn ulid(s: &str) -> R<Ulid> {
    Ulid::from_string(s).map_err(|_| format!("Keine gültige ID: {s}"))
}

// ------------------------------------------------------------------ Konto

#[derive(Serialize)]
struct KontoStatus {
    eingerichtet: bool,
    entsperrt: bool,
    geraet: Option<String>,
    home: String,
}

#[tauri::command]
fn konto_status(state: State<AppState>) -> R<KontoStatus> {
    let h = home(&state)?;
    let entsperrt = sperre(&state.sitzung).is_some();
    let geraet = konto::lesen(&h).ok().map(|k| k.geraet_name);
    Ok(KontoStatus {
        eingerichtet: konto::existiert(&h),
        entsperrt,
        geraet,
        home: h.to_string_lossy().to_string(),
    })
}

#[derive(Serialize)]
struct Geheimnisse {
    wiederherstellungscode: String,
    desktop_schluessel: String,
    schluesselbund: bool,
}

/// Richtet das Konto ein. Liefert Wiederherstellungscode und Desktop-Schlüssel zur
/// einmaligen Anzeige; der Desktop-Schlüssel wandert zusätzlich in den Schlüsselbund.
#[tauri::command]
fn einrichten(state: State<AppState>, passwort: String, geraet: String) -> R<Geheimnisse> {
    let h = home(&state)?;
    let passwort = Zeroizing::new(passwort);
    let e = konto::einrichten(&h, passwort.as_bytes(), &geraet, KdfParams::default())
        .map_err(fehler)?;
    let schluesselbund = konto::desktop_key_speichern(e.konto.geraet_id, &e.desktop_key).is_ok();
    let vault = VaultKeys::with_desktop_key(&e.account_key, &e.desktop_key);
    let geheim = Geheimnisse {
        wiederherstellungscode: e.recovery_code.display().to_string(),
        desktop_schluessel: e.desktop_key.display().to_string(),
        schluesselbund,
    };
    *sperre(&state.sitzung) = Some(Sitzung {
        store: e.store,
        account_key: e.account_key,
        auth_key: e.auth_key,
        vault,
        geraet_id: e.konto.geraet_id,
        geraet_name: e.konto.geraet_name,
    });
    Ok(geheim)
}

/// Bestätigt den Wiederherstellungscode nach der Einrichtung (kalte Wiedereingabe).
#[tauri::command]
fn wiederherstellungscode_pruefen(state: State<AppState>, code: String) -> R<bool> {
    let h = home(&state)?;
    let k = konto::lesen(&h).map_err(fehler)?;
    let parsed = match RecoveryCode::parse(&code) {
        Ok(c) => c,
        Err(_) => return Ok(false),
    };
    Ok(lotse_core::crypto::konto_wiederherstellen(&k.header, &parsed).is_ok())
}

#[tauri::command]
fn entsperren(
    state: State<AppState>,
    passwort: String,
    desktop_schluessel: Option<String>,
) -> R<()> {
    let h = home(&state)?;
    let passwort = Zeroizing::new(passwort);
    let u = konto::entsperren(&h, passwort.as_bytes())
        .map_err(|_| "Falsches Master-Passwort".to_string())?;
    let dk = match desktop_schluessel.filter(|s| !s.trim().is_empty()) {
        Some(s) => {
            let dk = lotse_core::vault::DesktopKey::parse(&s).map_err(fehler)?;
            konto::desktop_key_speichern(u.konto.geraet_id, &dk).ok();
            Some(dk)
        }
        None => konto::desktop_key_laden(u.konto.geraet_id)
            .map_err(fehler)?
            .or(konto::desktop_key_aus_umgebung().map_err(fehler)?),
    };
    let vault = match &dk {
        Some(dk) => VaultKeys::with_desktop_key(&u.account_key, dk),
        None => VaultKeys::from_account_key(&u.account_key),
    };
    *sperre(&state.sitzung) = Some(Sitzung {
        store: u.store,
        account_key: u.account_key,
        auth_key: u.auth_key,
        vault,
        geraet_id: u.konto.geraet_id,
        geraet_name: u.konto.geraet_name,
    });
    Ok(())
}

/// Konto mit dem Wiederherstellungscode öffnen und dabei ein neues Passwort setzen.
/// Der Weg für ein vergessenes Master-Passwort; ohne ihn wäre die App an der Stelle
/// eine Sackgasse.
#[tauri::command]
fn konto_wiederherstellen(state: State<AppState>, code: String, neues_passwort: String) -> R<()> {
    let h = home(&state)?;
    let neues_passwort = Zeroizing::new(neues_passwort);
    if neues_passwort.len() < 12 {
        return Err("Das neue Master-Passwort braucht mindestens 12 Zeichen".into());
    }
    let k = konto::lesen(&h).map_err(fehler)?;
    let code = RecoveryCode::parse(&code)
        .map_err(|_| "Das ist kein gültiger Wiederherstellungscode".to_string())?;

    // Geräte, die per Anmeldung dazukamen, tragen das Recovery-Wrapping nicht lokal.
    // Dort scheitert jeder Code – das ist kein falscher Code, und die Meldung darf das
    // nicht behaupten.
    if k.header.wrapped_account_key_recovery.is_none() {
        return Err("Dieses Gerät kam per Anmeldung dazu und trägt den              Wiederherstellungscode nicht lokal. Stelle das Konto auf dem Gerät wieder              her, auf dem du Lotse eingerichtet hast; danach meldest du dieses hier mit              dem neuen Passwort neu an."
            .into());
    }

    let ak = lotse_core::crypto::konto_wiederherstellen(&k.header, &code)
        .map_err(|_| "Dieser Code passt nicht zu diesem Konto".to_string())?;

    // Der Code bleibt derselbe; er wird nur am neuen Salt neu verankert.
    let w = lotse_core::crypto::passwort_wechseln(
        &k.header,
        &ak,
        neues_passwort.as_bytes(),
        lotse_core::crypto::RecoveryWechsel::Behalten(&code),
    )
    .map_err(fehler)?;

    // Die lokale Datenbank hängt am Account-Schlüssel, nicht am Passwort – sie öffnet
    // also weiterhin. Von dort kommt die Adresse des Dienstes.
    let store = lotse_core::store::Store::open(&h.join(konto::DB_DATEI), &ak, k.geraet_id)
        .map_err(fehler)?;

    // Liegt das Konto beim Dienst, muss er den Wechsel erfahren. Sonst hätte dieses
    // Gerät ein neues Passwort und der Dienst weiter das alte; auffallen würde das erst
    // beim nächsten Login auf einem zweiten Gerät – also wieder im schlechtesten
    // Moment. Erst danach wird lokal geschrieben: lehnt der Dienst ab, bleibt hier alles,
    // wie es war.
    if let (Ok(client), Ok(Some(email))) = (
        sync_client::client_aus_store(&store),
        store.meta_get(sync_client::meta::EMAIL),
    ) {
        let salt: [u8; lotse_core::crypto::SALT_LEN] = k
            .header
            .salt
            .as_slice()
            .try_into()
            .map_err(|_| "Salt hat die falsche Länge".to_string())?;
        let alter_recovery_key = code.derive_key(&salt, &k.header.kdf).map_err(fehler)?;
        let alter_recovery_auth = lotse_core::crypto::recovery_auth_key(&alter_recovery_key);
        client
            .wiederherstellung_abschliessen(
                &email,
                &alter_recovery_auth,
                &w.auth_key,
                &w.recovery_auth_key,
                &w.header,
            )
            .map_err(fehler)?;
    }

    let neues_konto = konto::Konto {
        header: w.header,
        geraet_id: k.geraet_id,
        geraet_name: k.geraet_name.clone(),
    };
    konto::schreiben(&h, &neues_konto).map_err(fehler)?;
    let vault = match konto::desktop_key_laden(k.geraet_id).map_err(fehler)? {
        Some(dk) => VaultKeys::with_desktop_key(&ak, &dk),
        None => VaultKeys::from_account_key(&ak),
    };
    *sperre(&state.sitzung) = Some(Sitzung {
        store,
        account_key: ak,
        auth_key: w.auth_key,
        vault,
        geraet_id: k.geraet_id,
        geraet_name: k.geraet_name,
    });
    Ok(())
}

/// Master-Passwort wechseln. Der Wiederherstellungscode muss dabei neu verankert werden
/// (er hängt am Salt): entweder wird der bisherige angegeben, oder es entsteht ein neuer,
/// der einmal angezeigt wird.
#[tauri::command]
fn passwort_aendern(
    state: State<AppState>,
    altes_passwort: String,
    neues_passwort: String,
    code: Option<String>,
) -> R<Option<String>> {
    let h = home(&state)?;
    let altes_passwort = Zeroizing::new(altes_passwort);
    let neues_passwort = Zeroizing::new(neues_passwort);
    if neues_passwort.len() < 12 {
        return Err("Das neue Master-Passwort braucht mindestens 12 Zeichen".into());
    }
    let k = konto::lesen(&h).map_err(fehler)?;
    let (ak, alter_auth) =
        lotse_core::crypto::konto_entsperren(&k.header, altes_passwort.as_bytes())
            .map_err(|_| "Das bisherige Passwort stimmt nicht".to_string())?;

    let geparst = match code.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => {
            // Geräte, die per Anmeldung dazukamen, haben das Recovery-Wrapping nicht
            // lokal – der Dienst hält es. Ohne Wrapping lässt sich ein Code hier nicht
            // prüfen, und ungeprüft übernehmen wäre schlimmer als absagen.
            if k.header.wrapped_account_key_recovery.is_none() {
                return Err("Auf diesem Gerät liegt kein Wiederherstellungs-Wrapping; \
                     es kam per Anmeldung dazu. Lass das Code-Feld leer, dann entsteht \
                     ein neuer Wiederherstellungscode, der für alle Geräte gilt."
                    .into());
            }
            Some(
                RecoveryCode::parse(s)
                    .map_err(|_| "Das ist kein gültiger Wiederherstellungscode".to_string())?,
            )
        }
        None => None,
    };
    let wahl = match &geparst {
        Some(c) => lotse_core::crypto::RecoveryWechsel::Behalten(c),
        None => lotse_core::crypto::RecoveryWechsel::Neu,
    };
    let w = lotse_core::crypto::passwort_wechseln(&k.header, &ak, neues_passwort.as_bytes(), wahl)
        .map_err(|e| match e {
            Error::Decrypt => "Dieser Wiederherstellungscode passt nicht zu diesem Konto".into(),
            andere => fehler(andere),
        })?;

    // Reihenfolge mit Bedacht: erst lokal schreiben, dann den Dienst. Scheitert der
    // Dienst, wird die alte Datei zurückgeschrieben – am Ende gilt überall dasselbe
    // Passwort. Andersherum genügte ein fehlgeschlagener Dateizugriff, um Gerät und
    // Dienst dauerhaft zu trennen: ein zweiter Versuch bräuchte den alten
    // Auth-Schlüssel, den der Dienst dann schon nicht mehr kennt.
    let neues_konto = konto::Konto {
        header: w.header.clone(),
        geraet_id: k.geraet_id,
        geraet_name: k.geraet_name.clone(),
    };
    konto::schreiben(&h, &neues_konto).map_err(fehler)?;

    // Ohne eingerichteten Abgleich gibt es nichts zu melden; das ist kein Fehler.
    let dienst = {
        let mut guard = sperre(&state.sitzung);
        let s = guard
            .as_mut()
            .ok_or_else(|| "Nicht entsperrt".to_string())?;
        sync_client::client_aus_store(&s.store).ok()
    };
    if let Some(client) = dienst {
        if let Err(e) =
            client.passwort_wechseln(&alter_auth, &w.auth_key, &w.recovery_auth_key, &w.header)
        {
            return Err(match konto::schreiben(&h, &k) {
                Ok(()) => format!(
                    "Der Dienst hat den Wechsel abgelehnt ({e}). Es bleibt beim bisherigen Passwort."
                ),
                Err(_) => format!(
                    "Der Dienst hat den Wechsel abgelehnt ({e}), und die alte Kontodatei ließ \
                     sich nicht zurückschreiben. Dieses Gerät kennt jetzt das neue Passwort, \
                     der Dienst das alte. Mit dem Wiederherstellungscode lässt sich beides \
                     wieder zusammenbringen."
                ),
            });
        }
    }

    // Erst wenn beides steht, gilt der neue Schlüssel auch für die laufende Sitzung.
    {
        let mut guard = sperre(&state.sitzung);
        if let Some(s) = guard.as_mut() {
            s.auth_key = w.auth_key;
        }
    }

    Ok(w.neuer_code.map(|c| c.display().to_string()))
}

#[tauri::command]
fn sperren(state: State<AppState>) -> R<()> {
    // Erst den Beobachter anhalten: er greift sonst weiter auf die Sitzung zu.
    beobachter_stoppen(state.clone())?;
    *sperre(&state.sitzung) = None;
    Ok(())
}

#[tauri::command]
fn kann_nur_desktop(state: State<AppState>) -> R<bool> {
    mit(&state, |s| Ok(s.vault.kann_nur_desktop()))
}

// ------------------------------------------------------------------ Daten

#[tauri::command]
fn hafen(state: State<AppState>) -> R<Vec<HafenKarte>> {
    mit(&state, |s| s.store.hafen(now_ms()))
}

#[tauri::command]
fn projekte(state: State<AppState>) -> R<Vec<Projekt>> {
    mit(&state, |s| s.store.projekte())
}

#[tauri::command]
fn projekt(state: State<AppState>, id: String) -> R<Option<Projekt>> {
    let id = ulid(&id)?;
    mit(&state, |s| s.store.projekt(id))
}

#[tauri::command]
fn projekt_anlegen(
    state: State<AppState>,
    titel: String,
    vorlage: String,
    kurs: Option<String>,
) -> R<Projekt> {
    let v = Vorlage::parse(&vorlage).unwrap_or(Vorlage::Generisch);
    mit(&state, |s| {
        let mut p = Projekt::neu(titel, v, now_ms());
        p.kurs = kurs.unwrap_or_default();
        s.store.projekt_speichern(&p)?;
        Ok(p)
    })
}

#[tauri::command]
fn projekt_speichern(state: State<AppState>, projekt: Projekt) -> R<Projekt> {
    mit(&state, |s| {
        s.store.projekt_speichern(&projekt)?;
        Ok(projekt)
    })
}

/// Auffangprojekt für Gedanken ohne Zuordnung. Wird beim ersten Zugriff angelegt.
#[tauri::command]
fn postkorb(state: State<AppState>) -> R<Projekt> {
    mit(&state, |s| s.store.postkorb())
}

#[tauri::command]
fn projekt_loeschen(state: State<AppState>, id: String) -> R<()> {
    let id = ulid(&id)?;
    mit(&state, |s| s.store.projekt_loeschen(id))
}

#[tauri::command]
fn brief(state: State<AppState>, projekt_id: String) -> R<Brief> {
    let id = ulid(&projekt_id)?;
    mit(&state, |s| s.store.brief(id, now_ms()))
}

#[tauri::command]
fn notizen(state: State<AppState>, projekt_id: String) -> R<Vec<Notiz>> {
    let id = ulid(&projekt_id)?;
    mit(&state, |s| s.store.notizen(id))
}

#[tauri::command]
fn notiz(state: State<AppState>, id: String) -> R<Option<Notiz>> {
    let id = ulid(&id)?;
    mit(&state, |s| s.store.notiz(id))
}

#[tauri::command]
fn notiz_anlegen(
    state: State<AppState>,
    projekt_id: String,
    text: String,
    art: String,
) -> R<Notiz> {
    let id = ulid(&projekt_id)?;
    let art = Art::parse(&art).unwrap_or(Art::Log);
    mit(&state, |s| {
        let n = Notiz::neu(id, Quelle::Mensch, art, text, now_ms());
        s.store.notiz_speichern(&n)?;
        Ok(n)
    })
}

#[tauri::command]
fn faden_erledigen(state: State<AppState>, id: String) -> R<()> {
    let id = ulid(&id)?;
    mit(&state, |s| s.store.faden_erledigen(id))
}

/// Ordnet eine Notiz einem anderen Projekt zu – der Weg aus dem Postkorb heraus.
/// Der Text bleibt unangetastet; nur die Zuordnung ändert sich.
#[tauri::command]
fn notiz_verschieben(state: State<AppState>, id: String, projekt_id: String) -> R<Notiz> {
    let id = ulid(&id)?;
    let ziel = ulid(&projekt_id)?;
    mit(&state, |s| {
        let Some(mut n) = s.store.notiz(id)? else {
            return Err(Error::NotFound(format!("Notiz {id}")));
        };
        if s.store.projekt(ziel)?.is_none() {
            return Err(Error::NotFound(format!("Projekt {ziel}")));
        }
        n.projekt_id = ziel;
        s.store.notiz_speichern(&n)?;
        Ok(n)
    })
}

#[tauri::command]
fn status_setzen(
    state: State<AppState>,
    projekt_id: String,
    status: String,
    uebergabe: Option<String>,
    wiedervorlage: Option<String>,
) -> R<Projekt> {
    let id = ulid(&projekt_id)?;
    let st = Status::parse(&status).ok_or_else(|| format!("Unbekannter Status {status}"))?;
    mit(&state, |s| {
        s.store.status_setzen(
            id,
            st,
            uebergabe.as_deref(),
            wiedervorlage.as_deref(),
            Quelle::Mensch,
        )?;
        s.store
            .projekt(id)?
            .ok_or_else(|| Error::NotFound(format!("Projekt {id}")))
    })
}

#[tauri::command]
fn offene_faeden(state: State<AppState>) -> R<Vec<Notiz>> {
    mit(&state, |s| s.store.offene_faeden(None))
}

#[tauri::command]
fn suche(state: State<AppState>, anfrage: String) -> R<Vec<Treffer>> {
    mit(&state, |s| s.store.suche(&anfrage))
}

#[tauri::command]
fn referenzen(state: State<AppState>, projekt_id: String) -> R<Vec<Referenz>> {
    let id = ulid(&projekt_id)?;
    mit(&state, |s| s.store.referenzen(id))
}

#[tauri::command]
fn referenz_anlegen(
    state: State<AppState>,
    projekt_id: String,
    typ: String,
    ziel: String,
    rolle: String,
) -> R<Referenz> {
    let id = ulid(&projekt_id)?;
    let typ = ReferenzTyp::parse(&typ).ok_or_else(|| "Unbekannter Referenztyp".to_string())?;
    let rolle = Rolle::parse(&rolle).unwrap_or(Rolle::Material);
    mit(&state, |s| {
        let r = Referenz::neu(id, typ, ziel, rolle);
        s.store.referenz_speichern(&r)?;
        Ok(r)
    })
}

#[tauri::command]
fn referenz_pruefen(state: State<AppState>, id: String) -> R<Pruefstatus> {
    let id = ulid(&id)?;
    mit(&state, |s| s.store.referenz_pruefen(id))
}

#[tauri::command]
fn kandidaten(state: State<AppState>) -> R<Vec<Kandidat>> {
    mit(&state, |s| s.store.kandidaten())
}

#[tauri::command]
fn kandidat_uebernehmen(state: State<AppState>, pfad: String, titel: Option<String>) -> R<Projekt> {
    mit(&state, |s| {
        let k = lotse_core::detect::erkenne(std::path::Path::new(&pfad))
            .ok_or_else(|| Error::NotFound(format!("Kein Projekt erkannt in {pfad}")))?;
        let mut p = lotse_core::detect::uebernehmen(&mut s.store, &k)?;
        if let Some(t) = titel.filter(|t| !t.trim().is_empty()) {
            p.titel = t;
            s.store.projekt_speichern(&p)?;
        }
        Ok(p)
    })
}

#[tauri::command]
fn kandidat_verwerfen(state: State<AppState>, pfad: String) -> R<()> {
    mit(&state, |s| s.store.kandidat_verwerfen(&pfad))
}

#[tauri::command]
fn scan(state: State<AppState>, wurzeln: Vec<String>) -> R<Vec<Kandidat>> {
    let roots: Vec<PathBuf> = wurzeln.into_iter().map(PathBuf::from).collect();
    mit(&state, |s| {
        let alle = lotse_core::detect::scan(&roots, &lotse_core::detect::ScanOptionen::default())?;
        let neu: Vec<Kandidat> = alle
            .into_iter()
            .filter(|k| {
                k.bekannte_id
                    .and_then(|id| s.store.projekt(id).ok().flatten())
                    .is_none()
            })
            .collect();
        s.store.kandidaten_merken(&neu)?;
        Ok(neu)
    })
}

/// Systemdialog zur Ordnerwahl. Spart das Abtippen von Pfaden beim ersten Scan.
/// `Ok(None)`, wenn der Nutzer abbricht.
#[tauri::command]
async fn ordner_waehlen(app: tauri::AppHandle) -> R<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog().file().pick_folder(move |pfad| {
        let _ = tx.send(pfad);
    });
    let gewaehlt = rx.recv().map_err(|_| "Auswahl abgebrochen".to_string())?;
    Ok(gewaehlt
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().to_string()))
}

/// Öffnet eine Referenz im System: Ordner im Dateimanager, URL im Browser.
/// Nur auf ausdrücklichen Klick, nie automatisch.
#[tauri::command]
fn oeffnen(app: tauri::AppHandle, ziel: String) -> R<()> {
    use tauri_plugin_opener::OpenerExt;
    let ziel = ziel.trim();
    if ziel.is_empty() {
        return Err("Kein Ziel".into());
    }
    if ziel.starts_with("http://") || ziel.starts_with("https://") {
        app.opener()
            .open_url(ziel, None::<&str>)
            .map_err(|e| e.to_string())
    } else {
        app.opener()
            .open_path(ziel, None::<&str>)
            .map_err(|e| e.to_string())
    }
}

// ------------------------------------------------------------------ Export

/// Was im Bundle gelandet ist. `tresor_nicht_lesbar` zählt Einträge der Stufe
/// »nur Desktop«, für die auf diesem Gerät der Schlüssel fehlt – ehrlicher als
/// sie stillschweigend wegzulassen.
#[derive(Serialize)]
struct BundleBilanz {
    projekte: usize,
    notizen: usize,
    tresor: usize,
    tresor_nicht_lesbar: usize,
}

/// Klartext-Spiegel: ein Ordner mit Markdown, den grep und Obsidian lesen.
/// Ohne Tresor-Werte – der Spiegel ist unverschlüsselt.
#[tauri::command]
fn export_spiegel(state: State<AppState>, ziel: String) -> R<usize> {
    let pfad = PathBuf::from(ziel);
    mit(&state, |s| {
        lotse_core::export::spiegel::schreiben(&s.store, &pfad)
    })
}

/// Der Fluchtweg: alles als JSON, mit `age` und Passphrase verschlüsselt. Lässt sich
/// ohne Lotse öffnen (`age -d -o bundle.json backup.json.age`). Enthält den Tresor,
/// soweit er auf diesem Gerät lesbar ist.
#[tauri::command]
fn export_bundle(state: State<AppState>, ziel: String, passphrase: String) -> R<BundleBilanz> {
    let pfad = PathBuf::from(ziel);
    let passphrase = Zeroizing::new(passphrase);
    if passphrase.len() < 8 {
        return Err("Die Passphrase braucht mindestens 8 Zeichen".into());
    }
    // Nur das Zusammentragen braucht Speicher und Schlüssel. Verschlüsseln und Schreiben
    // laufen ohne Sperre – sonst hinge die ganze Oberfläche am Export, „Sperren“
    // eingeschlossen.
    let b = {
        let mut guard = sperre(&state.sitzung);
        let s = guard
            .as_mut()
            .ok_or_else(|| "Nicht entsperrt".to_string())?;
        lotse_core::export::bundle::bauen(&s.store, Some(&s.vault)).map_err(fehler)?
    };
    let json = serde_json::to_vec_pretty(&b).map_err(|e| e.to_string())?;
    let verschluesselt =
        lotse_core::export::bundle::age_verschluesseln(&json, &passphrase).map_err(fehler)?;
    lotse_core::export::atomar_schreiben(&pfad, &verschluesselt).map_err(fehler)?;

    Ok(BundleBilanz {
        projekte: b.projekte.len(),
        notizen: b.notizen.len(),
        tresor: b.tresor.len(),
        tresor_nicht_lesbar: b.tresor_nicht_lesbar.len(),
    })
}

/// Systemdialog für einen Speicherort. `None`, wenn abgebrochen.
#[tauri::command]
async fn datei_waehlen(app: tauri::AppHandle, name: String) -> R<Option<String>> {
    use tauri_plugin_dialog::DialogExt;
    let (tx, rx) = std::sync::mpsc::channel();
    app.dialog()
        .file()
        .set_file_name(&name)
        .save_file(move |pfad| {
            let _ = tx.send(pfad);
        });
    let gewaehlt = rx.recv().map_err(|_| "Auswahl abgebrochen".to_string())?;
    Ok(gewaehlt
        .and_then(|p| p.into_path().ok())
        .map(|p| p.to_string_lossy().to_string()))
}

// ------------------------------------------------------- Ordner-Beobachter

#[derive(Serialize, Clone)]
struct BeobachterStatus {
    laeuft: bool,
    wurzeln: Vec<String>,
}

#[derive(Serialize, Clone)]
struct BeobachterBilanz {
    datei_notizen: usize,
    git_notizen: usize,
    kandidaten: usize,
}

#[tauri::command]
fn beobachter_status(state: State<AppState>) -> R<BeobachterStatus> {
    let laeuft = sperre(&state.beobachter).is_some();
    let wurzeln = mit(&state, |s| lotse_core::watcher::wurzeln_laden(&s.store))?;
    Ok(BeobachterStatus {
        laeuft,
        wurzeln: wurzeln
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    })
}

/// Startet die Beobachtung im Hintergrund. Der Thread verarbeitet Ereignisse ohne die
/// Sitzung zu sperren und greift nur zum Schreiben kurz zu. Wird das Konto gesperrt,
/// beendet er sich von selbst.
#[tauri::command]
fn beobachter_starten(
    app: tauri::AppHandle,
    state: State<AppState>,
    wurzeln: Vec<String>,
) -> R<BeobachterStatus> {
    use std::sync::atomic::Ordering;
    use tauri::Emitter;

    let pfade: Vec<PathBuf> = wurzeln
        .iter()
        .map(|w| PathBuf::from(w.trim()))
        .filter(|p| !p.as_os_str().is_empty())
        .collect();
    if pfade.is_empty() {
        return Err("Kein Wurzelordner angegeben".into());
    }
    for p in &pfade {
        if !p.is_dir() {
            return Err(format!("Kein Ordner: {}", p.display()));
        }
    }

    mit(&state, |s| {
        lotse_core::watcher::wurzeln_speichern(&s.store, &pfade)
    })?;

    // Anhalten und Ablegen unter einer einzigen Sperre. Sonst könnten zwei fast
    // gleichzeitige Starts – ein Doppelklick genügt – einen Beobachter zurücklassen,
    // dessen Stop-Schalter niemand mehr hält.
    let mut handle = sperre(&state.beobachter);
    if let Some(vorheriger) = handle.take() {
        vorheriger.stop.store(true, Ordering::SeqCst);
    }

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_thread = stop.clone();
    let pfade_thread = pfade.clone();

    std::thread::spawn(move || {
        let zustand = app.state::<AppState>();

        // Start braucht den Speicher nur lesend und nur kurz.
        let mut b = {
            let mut guard = sperre(&zustand.sitzung);
            let Some(s) = guard.as_mut() else { return };
            match lotse_core::watcher::Beobachter::starten(&s.store, pfade_thread) {
                Ok(b) => b,
                Err(e) => {
                    let _ = app.emit("beobachter-fehler", e.to_string());
                    return;
                }
            }
        };

        // Erster Durchlauf setzt den Git-Stand und findet Kandidaten.
        let schreiben = |b: &mut lotse_core::watcher::Beobachter| -> bool {
            let mut guard = sperre(&zustand.sitzung);
            // Gesperrtes Konto beendet die Beobachtung.
            let Some(s) = guard.as_mut() else {
                return false;
            };
            if b.zuordnung_laden(&s.store).is_err() {
                return false;
            }
            match b.schreiben(&mut s.store) {
                Ok(bilanz) => {
                    if bilanz.datei_notizen > 0 || bilanz.git_notizen > 0 || bilanz.kandidaten > 0 {
                        let _ = app.emit(
                            "beobachter-bilanz",
                            BeobachterBilanz {
                                datei_notizen: bilanz.datei_notizen,
                                git_notizen: bilanz.git_notizen,
                                kandidaten: bilanz.kandidaten,
                            },
                        );
                    }
                    true
                }
                Err(_) => false,
            }
        };

        // Die Gegenseite (GitHub) in großem Abstand mitnehmen, falls eingeschaltet.
        // Sie hängt am selben Thread, weil sie dieselbe Frage beantwortet: was hat sich
        // getan, seit ich weg war.
        let ferne = |app: &tauri::AppHandle, stop: &std::sync::atomic::AtomicBool| {
            let zustand = app.state::<AppState>();
            if !forge_faellig(&zustand) {
                return;
            }
            match forge_lauf(&zustand, None, Some(stop)) {
                Ok(e) if e.abgefragt > 0 => {
                    let _ = app.emit("forge-bilanz", e);
                }
                Ok(_) => {}
                Err(e) => {
                    let _ = app.emit("forge-fehler", e);
                }
            }
        };

        if !schreiben(&mut b) {
            return;
        }
        ferne(&app, &stop_thread);
        while !stop_thread.load(Ordering::SeqCst) {
            // Ohne Sperre warten, sonst stünde die Oberfläche die ganze Zeit an.
            b.verarbeiten(std::time::Duration::from_secs(20));
            if stop_thread.load(Ordering::SeqCst) || !schreiben(&mut b) {
                break;
            }
            if stop_thread.load(Ordering::SeqCst) {
                break;
            }
            ferne(&app, &stop_thread);
        }
        // Eigenen Eintrag räumen, damit der Status nicht „läuft“ behauptet. Nur den
        // eigenen: nach Anhalten und sofortigem Neustart läuft schon ein anderer
        // Thread, und dessen Eintrag darf dieser hier nicht wegräumen.
        {
            let mut g = sperre(&zustand.beobachter);
            let ist_meiner = g
                .as_ref()
                .is_some_and(|h| std::sync::Arc::ptr_eq(&h.stop, &stop_thread));
            if ist_meiner {
                *g = None;
            }
        }
    });

    *handle = Some(BeobachterHandle { stop });
    drop(handle);

    Ok(BeobachterStatus {
        laeuft: true,
        wurzeln: pfade
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect(),
    })
}

#[tauri::command]
fn beobachter_stoppen(state: State<AppState>) -> R<()> {
    use std::sync::atomic::Ordering;
    if let Some(h) = sperre(&state.beobachter).take() {
        h.stop.store(true, Ordering::SeqCst);
    }
    Ok(())
}

// ---------------------------------------------------------------------- KI

const META_KI_URL: &str = "ki_basis_url";
const META_KI_MODELL: &str = "ki_modell";
const META_KI_EINTRAG: &str = "ki_schluessel_eintrag";
const META_KI_FELD: &str = "ki_schluessel_feld";

#[derive(Serialize)]
struct KiStatus {
    basis_url: String,
    modell: String,
    schluessel_eintrag: Option<String>,
    schluessel_feld: Option<String>,
    /// Antwortet unter der üblichen Adresse ein Ollama? Dann geht es ohne Schlüssel.
    ollama_da: bool,
}

/// Liest den Schlüssel aus dem Tresor. Der Tresor-Zugriff gehört in die Hülle;
/// das Modul `ai` bekommt den Schlüssel hereingereicht (siehe `CLAUDE.md`).
fn ki_schluessel(s: &mut Sitzung) -> lotse_core::Result<Option<String>> {
    let Some(id) = s.store.meta_get(META_KI_EINTRAG)?.filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    let feld = s
        .store
        .meta_get(META_KI_FELD)?
        .unwrap_or_else(|| "schluessel".to_string());
    let Ok(uid) = Ulid::from_string(&id) else {
        return Ok(None);
    };
    let Some(e) = s.store.tresor_eintrag(uid)? else {
        return Ok(None);
    };
    Ok(Some(e.feld_lesen(&s.vault, &feld)?.to_string()))
}

fn ki_ziel(s: &mut Sitzung) -> lotse_core::Result<lotse_core::ai::Ziel> {
    let basis_url = s
        .store
        .meta_get(META_KI_URL)?
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| lotse_core::ai::OLLAMA_URL.to_string());
    let modell = s.store.meta_get(META_KI_MODELL)?.unwrap_or_default();
    let schluessel = ki_schluessel(s)?;
    Ok(lotse_core::ai::Ziel {
        basis_url,
        modell,
        schluessel,
    })
}

#[tauri::command]
fn ki_status(state: State<AppState>) -> R<KiStatus> {
    let ziel = mit(&state, ki_ziel)?;
    // Ein kurzer Blick, ob lokal etwas antwortet – damit die Oberfläche sagen kann
    // „Ollama läuft" statt den Nutzer raten zu lassen.
    let ollama_da = lotse_core::ai::erreichbar(lotse_core::ai::OLLAMA_URL);
    let (eintrag, feld) = mit(&state, |s| {
        Ok((
            s.store.meta_get(META_KI_EINTRAG)?,
            s.store.meta_get(META_KI_FELD)?,
        ))
    })?;
    Ok(KiStatus {
        basis_url: ziel.basis_url,
        modell: ziel.modell,
        schluessel_eintrag: eintrag,
        schluessel_feld: feld,
        ollama_da,
    })
}

#[tauri::command]
fn ki_ziel_setzen(
    state: State<AppState>,
    basis_url: String,
    modell: String,
    schluessel_eintrag: String,
    schluessel_feld: String,
) -> R<()> {
    mit(&state, |s| {
        s.store.meta_set(META_KI_URL, basis_url.trim())?;
        s.store.meta_set(META_KI_MODELL, modell.trim())?;
        s.store
            .meta_set(META_KI_EINTRAG, schluessel_eintrag.trim())?;
        s.store.meta_set(META_KI_FELD, schluessel_feld.trim())?;
        Ok(())
    })
}

/// Modelle eines Ziels, damit niemand einen Modellnamen abtippen muss.
/// `basis_url` kommt aus dem Formular, der Schlüssel aus dem gemerkten Tresor-Eintrag.
#[tauri::command]
fn ki_modelle(state: State<AppState>, basis_url: String) -> R<Vec<String>> {
    let schluessel = mit(&state, ki_schluessel)?;
    let ziel = lotse_core::ai::Ziel {
        basis_url: if basis_url.trim().is_empty() {
            lotse_core::ai::OLLAMA_URL.to_string()
        } else {
            basis_url.trim().to_string()
        },
        modell: "egal".into(),
        schluessel,
    };
    lotse_core::ai::modelle(&ziel).map_err(fehler)
}

/// Genau der Text, der gesendet würde. Die Oberfläche zeigt ihn, bevor etwas das
/// Gerät verlässt – so verlangt es das Konzept.
#[tauri::command]
fn ki_anfrage_text(state: State<AppState>, projekt_id: String) -> R<String> {
    let id = ulid(&projekt_id)?;
    mit(&state, |s| {
        let p = s
            .store
            .projekt(id)?
            .ok_or_else(|| Error::NotFound(format!("Projekt {id}")))?;
        let b = s.store.brief(id, now_ms())?;
        let mut lage = String::new();
        if let Some(u) = &b.letzte_uebergabe {
            lage.push_str("Letzte Übergabe: ");
            lage.push_str(u);
            lage.push('\n');
        } else if let Some(n) = &b.letzte_notiz {
            lage.push_str("Letzte Notiz: ");
            lage.push_str(n);
            lage.push('\n');
        }
        lage.push_str(&format!("Letzter Kontakt vor {} Tagen.\n", b.tage_seit));
        for (quelle, anzahl) in &b.aktivitaet_seit_letztem_besuch {
            lage.push_str(&format!("Seitdem {anzahl} Einträge aus Quelle {quelle}.\n"));
        }
        let faeden: Vec<String> = b.offene_faeden.iter().map(|n| n.text.clone()).collect();
        Ok(lotse_core::ai::anfrage_text(&lage, &faeden, &p.titel))
    })
}

/// Schickt genau den gezeigten Text und liefert die Zusammenfassung zurück. Sie wird
/// nicht gespeichert – der Mensch entscheidet, ob sie ins Logbuch soll.
#[tauri::command]
fn ki_verdichten(state: State<AppState>, eingabe: String) -> R<String> {
    let ziel = mit(&state, ki_ziel)?;
    lotse_core::ai::verdichten(&ziel, &eingabe).map_err(fehler)
}

// --------------------------------------------------------------- Remote-Git

/// Wo der Token liegt: Kennung eines Tresor-Eintrags und Feldname, je Hoster getrennt.
/// Der Token selbst steht nie hier, nur der Zeiger darauf – und ein Token für GitHub
/// wird nie an GitLab geschickt, auch nicht versehentlich.
const META_FORGE_EINTRAG: &str = "forge_token_eintrag";
const META_FORGE_FELD: &str = "forge_token_feld";
const META_FORGE_EINTRAG_GITLAB: &str = "forge_token_gitlab_eintrag";
const META_FORGE_FELD_GITLAB: &str = "forge_token_gitlab_feld";

/// Die beiden Schlüssel zum Hoster. Ein unbekannter Name ist ein Fehler und wird
/// nicht stillschweigend zu GitHub – sonst landete ein GitLab-Token unter dem
/// GitHub-Zeiger und würde später an GitHub geschickt.
fn forge_meta_schluessel(anbieter: &str) -> R<(&'static str, &'static str)> {
    if anbieter.eq_ignore_ascii_case("gitlab") {
        Ok((META_FORGE_EINTRAG_GITLAB, META_FORGE_FELD_GITLAB))
    } else if anbieter.eq_ignore_ascii_case("github") {
        Ok((META_FORGE_EINTRAG, META_FORGE_FELD))
    } else {
        Err(format!("Unbekannter Hoster: {anbieter}"))
    }
}

/// „Von allein mitlaufen“: der Beobachter fragt die Gegenseite in großem Abstand mit ab.
const META_FORGE_AUTO: &str = "forge_auto";
/// Zeitpunkt der letzten Abfrage, damit ein Neustart nicht sofort wieder anfragt.
const META_FORGE_ZULETZT: &str = "forge_zuletzt";

/// Abstand zwischen zwei selbsttätigen Abfragen. Der Stand auf der Gegenseite ändert
/// sich in Minuten, nicht in Sekunden; das Kontingent von GitHub ist begrenzt.
const FORGE_ABSTAND_MS: i64 = 30 * 60 * 1000;

#[derive(Serialize)]
struct ForgeProjekt {
    projekt_id: String,
    titel: String,
    repo: String,
    /// „GitHub“ oder „GitLab“.
    anbieter: &'static str,
    /// Adresse zum Öffnen im Browser.
    url: String,
    /// Wie das Repo gefunden wurde: „adresse“ oder „ordner“.
    herkunft: &'static str,
}

#[derive(Serialize)]
struct ForgeStatus {
    /// Projekte mit einem erkannten Repo auf der Gegenseite.
    projekte: Vec<ForgeProjekt>,
    token_eintrag: Option<String>,
    token_feld: Option<String>,
    token_eintrag_gitlab: Option<String>,
    token_feld_gitlab: Option<String>,
    auto: bool,
    zuletzt: Option<i64>,
}

/// Sucht zu jedem Projekt das Repo auf der Gegenseite.
///
/// Zweistufig, weil das Erkennen `git` aufruft: gesammelt wird unter der Sperre,
/// nachgesehen wird ohne sie. Sonst stünde die Oberfläche bei vielen Projekten an.
fn forge_ziele(
    state: &State<AppState>,
    nur: Option<Ulid>,
) -> R<Vec<(Projekt, lotse_core::forge::RepoZeiger, lotse_core::forge::Herkunft)>> {
    let gesammelt: Vec<(Projekt, Vec<Referenz>, Ulid)> = mit(state, |s| {
        let geraet = s.store.device_id();
        let mut out = Vec::new();
        for p in s.store.projekte()? {
            if nur.is_some_and(|id| p.id != id) {
                continue;
            }
            let refs = s.store.referenzen(p.id)?;
            if !refs.is_empty() {
                out.push((p, refs, geraet));
            }
        }
        Ok(out)
    })?;

    Ok(gesammelt
        .into_iter()
        .filter_map(|(p, refs, geraet)| {
            lotse_core::forge::zeiger_aus_referenzen(&refs, geraet).map(|(z, h)| (p, z, h))
        })
        .collect())
}

#[tauri::command]
fn forge_status(state: State<AppState>) -> R<ForgeStatus> {
    let projekte = forge_ziele(&state, None)?
        .into_iter()
        .map(|(p, z, herkunft)| ForgeProjekt {
            projekt_id: p.id.to_string(),
            titel: p.titel,
            repo: z.anzeige(),
            anbieter: z.anbieter.as_str(),
            url: z.web_url(),
            herkunft: herkunft.as_str(),
        })
        .collect();
    mit(&state, |s| {
        Ok(ForgeStatus {
            projekte,
            token_eintrag: s.store.meta_get(META_FORGE_EINTRAG)?,
            token_feld: s.store.meta_get(META_FORGE_FELD)?,
            token_eintrag_gitlab: s.store.meta_get(META_FORGE_EINTRAG_GITLAB)?,
            token_feld_gitlab: s.store.meta_get(META_FORGE_FELD_GITLAB)?,
            auto: s.store.meta_get(META_FORGE_AUTO)?.as_deref() == Some("1"),
            zuletzt: s
                .store
                .meta_get(META_FORGE_ZULETZT)?
                .and_then(|v| v.parse().ok()),
        })
    })
}

/// Merkt sich, welcher Tresor-Eintrag den Token für einen Hoster hält. Leere Kennung
/// löst die Bindung.
#[tauri::command]
fn forge_token_setzen(
    state: State<AppState>,
    anbieter: String,
    eintrag_id: String,
    feld: String,
) -> R<()> {
    let (k_eintrag, k_feld) = forge_meta_schluessel(&anbieter)?;
    mit(&state, |s| {
        s.store.meta_set(k_eintrag, eintrag_id.trim())?;
        s.store.meta_set(k_feld, feld.trim())?;
        Ok(())
    })
}

/// Schaltet die selbsttätige Abfrage im Beobachter ein oder aus.
#[tauri::command]
fn forge_auto_setzen(state: State<AppState>, an: bool) -> R<()> {
    mit(&state, |s| {
        s.store.meta_set(META_FORGE_AUTO, if an { "1" } else { "0" })?;
        Ok(())
    })
}

#[derive(Serialize, Clone)]
struct ForgeErgebnis {
    abgefragt: usize,
    notizen: usize,
    fehler: Vec<String>,
}

/// Holt den Token eines Hosters aus dem Tresor. Der Tresor-Zugriff passiert hier in der
/// Hülle; das Modul `forge` bekommt den Token hereingereicht (siehe `CLAUDE.md`).
fn forge_token(state: &State<AppState>, anbieter: lotse_core::forge::Anbieter) -> R<Option<String>> {
    let (k_eintrag, k_feld) = forge_meta_schluessel(anbieter.as_str())?;
    mit(state, |s| {
        let Some(id) = s.store.meta_get(k_eintrag)?.filter(|v| !v.is_empty()) else {
            return Ok(None);
        };
        let feld = s
            .store
            .meta_get(k_feld)?
            .unwrap_or_else(|| "token".to_string());
        let Ok(uid) = Ulid::from_string(&id) else {
            return Ok(None);
        };
        let Some(e) = s.store.tresor_eintrag(uid)? else {
            return Ok(None);
        };
        Ok(Some(e.feld_lesen(&s.vault, &feld)?.to_string()))
    })
}

/// Fragt den Stand bei GitHub ab und schreibt je Projekt höchstens eine Notiz.
///
/// `abbruch` ist der Stop-Schalter des Beobachters: nach „Anhalten“ oder „Sperren“
/// wird nichts mehr geschrieben, auch wenn eine Anfrage noch unterwegs war.
fn forge_lauf(
    state: &State<AppState>,
    nur: Option<Ulid>,
    abbruch: Option<&std::sync::atomic::AtomicBool>,
) -> R<ForgeErgebnis> {
    use std::sync::atomic::Ordering;
    let angehalten = || abbruch.is_some_and(|a| a.load(Ordering::SeqCst));

    let ziele = forge_ziele(state, nur)?;
    let versuche = ziele.len();

    // Je Hoster ein Token, einmal geholt. Ein Token für GitHub geht nie an GitLab.
    let mut token_github = None;
    let mut token_gitlab = None;
    for anbieter in ziele.iter().map(|(_, z, _)| z.anbieter) {
        match anbieter {
            lotse_core::forge::Anbieter::GitHub if token_github.is_none() => {
                token_github = Some(forge_token(state, anbieter)?)
            }
            lotse_core::forge::Anbieter::GitLab if token_gitlab.is_none() => {
                token_gitlab = Some(forge_token(state, anbieter)?)
            }
            _ => {}
        }
    }

    let mut notizen = 0usize;
    let mut fehler = Vec::new();
    let mut abgefragt = 0usize;

    for (p, z, _) in ziele {
        if angehalten() {
            break;
        }
        let token = match z.anbieter {
            lotse_core::forge::Anbieter::GitHub => &token_github,
            lotse_core::forge::Anbieter::GitLab => &token_gitlab,
        };
        let token = token.as_ref().and_then(|t| t.as_deref());
        // Ohne gehaltene Sperre abfragen: das geht übers Netz und dauert.
        match lotse_core::forge::abfragen(&z, token) {
            Ok(stand) => {
                abgefragt += 1;
                if angehalten() {
                    break;
                }
                if let Some(n) = lotse_core::forge::notiz(p.id, &z, &stand, now_ms()) {
                    mit(state, |s| s.store.notiz_speichern(&n))?;
                    notizen += 1;
                }
            }
            Err(e) => fehler.push(format!("{}: {e}", z.anzeige())),
        }
    }

    // Der Zeitpunkt zählt den Versuch, nicht den Erfolg – und nur bei einem vollen
    // Durchlauf. Sonst liefe der Beobachter ohne Netz alle zwanzig Sekunden wieder los,
    // und eine Abfrage für ein einzelnes Projekt würde den nächsten vollen Durchlauf
    // verschieben.
    if nur.is_none() && versuche > 0 && !angehalten() {
        mit(state, |s| {
            s.store.meta_set(META_FORGE_ZULETZT, &now_ms().to_string())
        })?;
    }

    Ok(ForgeErgebnis {
        abgefragt,
        notizen,
        fehler,
    })
}

/// Das Repo eines einzelnen Projekts, für die Projektseite. `None`, wenn es keins gibt.
#[tauri::command]
fn forge_projekt(state: State<AppState>, projekt_id: String) -> R<Option<ForgeProjekt>> {
    let id = ulid(&projekt_id)?;
    Ok(forge_ziele(&state, Some(id))?
        .into_iter()
        .next()
        .map(|(p, z, herkunft)| ForgeProjekt {
            projekt_id: p.id.to_string(),
            titel: p.titel,
            repo: z.anzeige(),
            anbieter: z.anbieter.as_str(),
            url: z.web_url(),
            herkunft: herkunft.as_str(),
        }))
}

/// Von Hand angestoßen. Ohne `projekt_id` alle Projekte mit erkanntem Repo.
#[tauri::command]
fn forge_abfragen(state: State<AppState>, projekt_id: Option<String>) -> R<ForgeErgebnis> {
    let nur = match projekt_id.as_deref().filter(|s| !s.is_empty()) {
        Some(p) => Some(ulid(p)?),
        None => None,
    };
    forge_lauf(&state, nur, None)
}

/// Ein Durchlauf aus dem Beobachter heraus, wenn er eingeschaltet und der letzte lange
/// genug her ist. Antwortet `true`, wenn abgefragt wurde.
fn forge_faellig(state: &State<AppState>) -> bool {
    let entscheidung = mit(state, |s| {
        if s.store.meta_get(META_FORGE_AUTO)?.as_deref() != Some("1") {
            return Ok(false);
        }
        let zuletzt: i64 = s
            .store
            .meta_get(META_FORGE_ZULETZT)?
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        Ok(now_ms() - zuletzt >= FORGE_ABSTAND_MS)
    });
    entscheidung.unwrap_or(false)
}

// ------------------------------------------------------------------ Update

/// Ob beim Start nachgesehen wird. Gesetzt heißt „nein"; ohne Eintrag wird nachgesehen.
const META_UPDATE_AUS: &str = "update_aus";
/// Zeitpunkt der letzten Prüfung, damit nicht jeder Start anfragt.
const META_UPDATE_ZULETZT: &str = "update_zuletzt";
const UPDATE_ABSTAND_MS: i64 = 24 * 60 * 60 * 1000;

#[derive(Serialize, Clone)]
struct UpdateStand {
    /// Laufende Version dieser App.
    laufend: String,
    /// Neuere Version, falls es eine gibt.
    neu: Option<String>,
    /// Seite mit allen Dateien und dem Änderungstext.
    seite: Option<String>,
    /// Datei für dieses System, falls es eine gibt.
    datei: Option<String>,
    datei_url: Option<String>,
    datei_bytes: Option<u64>,
    vorab: bool,
    /// Wird beim Start nachgesehen?
    automatisch: bool,
    zuletzt: Option<i64>,
}

fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Sieht nach, ob es eine neuere Version gibt. Lädt nichts und führt nichts aus:
/// die Installer sind unsigniert, deshalb bleibt das Herunterladen Sache des Menschen
/// (siehe `docs/THREAT_MODEL.md`).
#[tauri::command]
fn update_pruefen(state: State<AppState>, erzwingen: Option<bool>) -> R<UpdateStand> {
    let (automatisch, zuletzt) = mit(&state, |s| {
        Ok((
            s.store.meta_get(META_UPDATE_AUS)?.as_deref() != Some("1"),
            s.store
                .meta_get(META_UPDATE_ZULETZT)?
                .and_then(|v| v.parse::<i64>().ok()),
        ))
    })?;
    let laufend = app_version();
    let leer = UpdateStand {
        laufend: laufend.clone(),
        neu: None,
        seite: None,
        datei: None,
        datei_url: None,
        datei_bytes: None,
        vorab: false,
        automatisch,
        zuletzt,
    };

    let erzwingen = erzwingen.unwrap_or(false);
    if !erzwingen {
        // Ohne ausdrücklichen Wunsch nur, wenn es eingeschaltet und lange genug her ist.
        if !automatisch || zuletzt.is_some_and(|z| now_ms() - z < UPDATE_ABSTAND_MS) {
            return Ok(leer);
        }
    }

    // Solange es nur Vorabversionen gibt, wären fertige Versionen eine leere Liste.
    let gefunden = lotse_core::update::neueste(&lotse_core::update::eigenes_repo(), true)
        .map_err(fehler)?;
    let jetzt = now_ms();
    mit(&state, |s| {
        s.store.meta_set(META_UPDATE_ZULETZT, &jetzt.to_string())
    })?;

    let Some(v) = gefunden.filter(|v| lotse_core::update::neuer_als(&v.version, &laufend)) else {
        return Ok(UpdateStand {
            zuletzt: Some(jetzt),
            ..leer
        });
    };
    let datei = lotse_core::update::passende_datei(&v, std::env::consts::OS, std::env::consts::ARCH);
    Ok(UpdateStand {
        laufend,
        neu: Some(v.version.clone()),
        seite: Some(v.seite.clone()),
        datei: datei.map(|d| d.name.clone()),
        datei_url: datei.map(|d| d.url.clone()),
        datei_bytes: datei.map(|d| d.bytes),
        vorab: v.vorab,
        automatisch,
        zuletzt: Some(jetzt),
    })
}

/// Schaltet das Nachsehen beim Start ein oder aus.
#[tauri::command]
fn update_automatisch_setzen(state: State<AppState>, an: bool) -> R<()> {
    mit(&state, |s| {
        s.store.meta_set(META_UPDATE_AUS, if an { "0" } else { "1" })?;
        Ok(())
    })
}

// ------------------------------------------------------------------ Kalender

/// Wie lange ein geholter Kalender wiederverwendet wird, bevor er neu geladen wird.
/// Termine ändern sich nicht im Minutentakt, und jeder Aufruf geht übers Netz.
const KALENDER_FRISCHE_MS: i64 = 15 * 60 * 1000;

#[derive(Serialize, Clone)]
struct TerminAnzeige {
    titel: String,
    datum: String,
    uhrzeit: Option<String>,
    utc: bool,
    ort: Option<String>,
    /// Der Termin wiederholt sich; `false` heißt nicht, dass er einmalig ist, sondern
    /// nur, dass keine Regel dranhängt.
    wiederholt: bool,
    /// Die Regel enthält Teile, die Lotse nicht ausrechnet – dann steht hier `true` und
    /// die Oberfläche sagt es dazu, statt ein Datum zu behaupten.
    ungenau: bool,
}

#[derive(Serialize, Clone)]
struct KalenderErgebnis {
    quellen: Vec<String>,
    termine: Vec<TerminAnzeige>,
    fehler: Vec<String>,
}

/// Heutiges Datum als `JJJJ-MM-TT`, aus derselben Uhr wie alle Zeitstempel.
fn heute() -> String {
    lotse_core::export::iso(now_ms())
        .chars()
        .take(10)
        .collect()
}

/// Holt einen Kalender, aber höchstens alle 15 Minuten neu. Der Zwischenspeicher liegt
/// im App-Zustand und ist nach dem Beenden weg – Termine gehören dem Kalender, nicht
/// Lotse (siehe `kalender.rs`).
fn kalender_holen(state: &State<AppState>, ziel: &str) -> lotse_core::Result<String> {
    if let Some((geholt, ics)) = sperre(&state.kalender).get(ziel) {
        if now_ms() - geholt < KALENDER_FRISCHE_MS {
            return Ok(ics.clone());
        }
    }
    let ics = lotse_core::kalender::holen(ziel)?;
    sperre(&state.kalender).insert(ziel.to_string(), (now_ms(), ics.clone()));
    Ok(ics)
}

/// Anstehende Termine eines Projekts aus seinen Kalender-Referenzen.
#[tauri::command]
fn kalender_termine(
    state: State<AppState>,
    projekt_id: String,
    tage: Option<u32>,
) -> R<KalenderErgebnis> {
    let id = ulid(&projekt_id)?;
    // Erst sammeln, dann holen: das Netz gehört nicht unter die Sperre.
    let ziele: Vec<String> = mit(&state, |s| {
        Ok(s.store
            .referenzen(id)?
            .into_iter()
            .filter(|r| {
                matches!(r.typ, ReferenzTyp::Url | ReferenzTyp::Datei)
                    && lotse_core::kalender::ist_kalender(&r.ziel)
            })
            .map(|r| r.ziel)
            .collect())
    })?;

    let von = heute();
    let bis = lotse_core::kalender::tage_spaeter(&von, i64::from(tage.unwrap_or(90)));
    let mut alle = Vec::new();
    let mut fehler = Vec::new();
    for ziel in &ziele {
        match kalender_holen(&state, ziel).map(|ics| lotse_core::kalender::lesen(&ics)) {
            Ok(t) => alle.extend(t),
            Err(e) => fehler.push(format!("{ziel}: {e}")),
        }
    }

    let termine = lotse_core::kalender::kommende(&alle, &von, &bis, 20)
        .into_iter()
        .map(|t| TerminAnzeige {
            titel: t.titel,
            datum: t.datum,
            uhrzeit: t.uhrzeit,
            utc: t.utc,
            ort: t.ort,
            wiederholt: t.wiederholung.is_some(),
            ungenau: t.wiederholung.is_some_and(|r| !r.genau),
        })
        .collect();

    Ok(KalenderErgebnis {
        quellen: ziele,
        termine,
        fehler,
    })
}

// ------------------------------------------------------------------ Geräte

#[tauri::command]
fn sync_geraete(state: State<AppState>) -> R<Vec<sync_client::GeraetInfo>> {
    mit(&state, |s| {
        match sync_client::client_aus_store(&s.store) {
            Ok(client) => client.geraete(),
            // Ohne Abgleich gibt es keine weiteren Geräte, nur dieses hier.
            Err(_) => Ok(Vec::new()),
        }
    })
}

/// Entzieht einem Gerät den Zugang. Das eigene lässt sich nicht widerrufen –
/// dafür gibt es Sperren.
#[tauri::command]
fn sync_geraet_widerrufen(state: State<AppState>, id: String) -> R<()> {
    mit(&state, |s| {
        if id == s.geraet_id.to_string() {
            return Err(Error::Invalid(
                "Das eigene Gerät lässt sich nicht widerrufen".into(),
            ));
        }
        sync_client::client_aus_store(&s.store)?.geraet_widerrufen(&id)
    })
}

// ----------------------------------------------------------------- Tresor

#[tauri::command]
fn tresor_liste(state: State<AppState>, projekt_id: Option<String>) -> R<Vec<TresorEintrag>> {
    let pid = projekt_id.map(|p| ulid(&p)).transpose()?;
    mit(&state, |s| s.store.tresor_eintraege(pid))
}

#[tauri::command]
fn tresor_anlegen(
    state: State<AppState>,
    titel: String,
    projekt_ids: Vec<String>,
    stufe: String,
    felder: Vec<(String, String)>,
) -> R<TresorEintrag> {
    let pids: Vec<Ulid> = projekt_ids.iter().map(|p| ulid(p)).collect::<R<_>>()?;
    let stufe = Stufe::parse(&stufe).unwrap_or(Stufe::Ueberall);
    mit(&state, |s| {
        let refs: Vec<(&str, &str)> = felder
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let e = TresorEintrag::neu(&s.vault, titel, pids, stufe, &refs, now_ms())?;
        s.store.tresor_speichern(&e)?;
        Ok(e)
    })
}

/// Entschlüsselt genau ein Feld. Die Oberfläche zeigt es an und verwirft es wieder.
#[tauri::command]
fn tresor_feld_lesen(state: State<AppState>, id: String, feld: String) -> R<String> {
    let id = ulid(&id)?;
    mit(&state, |s| {
        let e = s
            .store
            .tresor_eintrag(id)?
            .ok_or_else(|| Error::NotFound(format!("Tresor-Eintrag {id}")))?;
        Ok(e.feld_lesen(&s.vault, &feld)?.to_string())
    })
}

#[tauri::command]
fn tresor_loeschen(state: State<AppState>, id: String) -> R<()> {
    let id = ulid(&id)?;
    mit(&state, |s| s.store.tresor_loeschen(id))
}

// ------------------------------------------------------------------- Sync

#[derive(Serialize)]
struct SyncStatus {
    eingerichtet: bool,
    url: Option<String>,
    email: Option<String>,
    ausstehend: u64,
    last_server_seq: u64,
    last_sync_ms: Option<i64>,
}

#[tauri::command]
fn sync_status(state: State<AppState>) -> R<SyncStatus> {
    mit(&state, |s| {
        let st = s.store.sync_state()?;
        Ok(SyncStatus {
            eingerichtet: s.store.meta_get(sync_client::meta::TOKEN)?.is_some(),
            url: s.store.meta_get(sync_client::meta::URL)?,
            email: s.store.meta_get(sync_client::meta::EMAIL)?,
            ausstehend: s.store.ausstehend()?,
            last_server_seq: st.last_server_seq,
            last_sync_ms: st.last_sync_ms,
        })
    })
}

#[derive(Serialize)]
struct SyncErgebnis {
    gepusht: usize,
    uebernommen: usize,
    verworfen: usize,
    konflikte: usize,
}

#[tauri::command]
async fn sync_jetzt(state: State<'_, AppState>) -> R<SyncErgebnis> {
    mit(&state, |s| {
        let client = sync_client::client_aus_store(&s.store)?;
        let r = sync_client::abgleichen(&mut s.store, &client)?;
        Ok(SyncErgebnis {
            gepusht: r.gepusht,
            uebernommen: r.uebernommen,
            verworfen: r.verworfen,
            konflikte: r.konflikte,
        })
    })
}

/// Bestehendes Konto beim Dienst registrieren. Braucht den Wiederherstellungscode.
#[tauri::command]
async fn sync_register(
    state: State<'_, AppState>,
    url: String,
    email: String,
    code: String,
) -> R<()> {
    let h = home(&state)?;
    let konto = konto::lesen(&h).map_err(fehler)?;
    let code = RecoveryCode::parse(&code).map_err(fehler)?;
    let salt: [u8; lotse_core::crypto::SALT_LEN] = konto
        .header
        .salt
        .as_slice()
        .try_into()
        .map_err(|_| "Salt".to_string())?;
    let rk = code.derive_key(&salt, &konto.header.kdf).map_err(fehler)?;
    let recovery_auth = lotse_core::crypto::recovery_auth_key(&rk);
    mit(&state, |s| {
        let client = sync_client::Client::new(&url);
        let g = sync_client::Geraet {
            id: s.geraet_id,
            name: s.geraet_name.clone(),
            platform: "desktop".into(),
        };
        let (account_id, token) =
            client.register(&email, &s.auth_key, &recovery_auth, &konto.header, &g)?;
        sync_client::verbindung_merken(&s.store, &url, &email, &account_id, &token)?;
        sync_client::abgleichen(&mut s.store, &client.with_token(&token))?;
        Ok(())
    })
}

/// Neues Gerät an bestehendem Konto anmelden (ersetzt die Einrichtung).
#[tauri::command]
async fn sync_login(
    state: State<'_, AppState>,
    url: String,
    email: String,
    passwort: String,
    geraet: String,
) -> R<()> {
    let h = home(&state)?;
    let passwort = Zeroizing::new(passwort);
    let client = sync_client::Client::new(&url);
    let pre = client.prelogin(&email).map_err(fehler)?;
    let stretched = lotse_core::crypto::derive_stretched(passwort.as_bytes(), &pre.salt, &pre.kdf)
        .map_err(fehler)?;
    let pk = lotse_core::crypto::split_password_keys(&stretched);
    let geraet_id = Ulid::new();
    let g = sync_client::Geraet {
        id: geraet_id,
        name: geraet.clone(),
        platform: "desktop".into(),
    };
    let login = client.login(&email, &pk.auth, &g).map_err(fehler)?;
    let (ak, auth) = lotse_core::crypto::konto_entsperren(&login.header, passwort.as_bytes())
        .map_err(|_| "Konto-Header lässt sich mit diesem Passwort nicht öffnen".to_string())?;
    let (_, mut store) =
        konto::aus_login(&h, login.header, geraet_id, &geraet, &ak).map_err(fehler)?;
    sync_client::verbindung_merken(
        &store,
        &url,
        &email,
        &login.account_id,
        &login.session_token,
    )
    .map_err(fehler)?;
    sync_client::abgleichen(
        &mut store,
        &sync_client::Client::new(&url).with_token(&login.session_token),
    )
    .map_err(fehler)?;
    let vault = VaultKeys::from_account_key(&ak);
    *sperre(&state.sitzung) = Some(Sitzung {
        store,
        account_key: ak,
        auth_key: auth,
        vault,
        geraet_id,
        geraet_name: geraet,
    });
    Ok(())
}

// --------------------------------------------------------------------- App

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| {
            let fallback = app.path().app_data_dir().ok();
            let home = konto::standard_home(fallback).unwrap_or_else(|| PathBuf::from("."));
            let zustand = app.state::<AppState>();
            *sperre(&zustand.home) = Some(home);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            konto_status,
            einrichten,
            wiederherstellungscode_pruefen,
            entsperren,
            sperren,
            konto_wiederherstellen,
            passwort_aendern,
            kann_nur_desktop,
            hafen,
            projekte,
            projekt,
            projekt_anlegen,
            projekt_speichern,
            postkorb,
            projekt_loeschen,
            brief,
            notizen,
            notiz,
            notiz_anlegen,
            faden_erledigen,
            notiz_verschieben,
            status_setzen,
            offene_faeden,
            suche,
            referenzen,
            referenz_anlegen,
            referenz_pruefen,
            kandidaten,
            kandidat_uebernehmen,
            kandidat_verwerfen,
            scan,
            ordner_waehlen,
            datei_waehlen,
            export_spiegel,
            export_bundle,
            beobachter_status,
            beobachter_starten,
            beobachter_stoppen,
            ki_status,
            ki_ziel_setzen,
            ki_modelle,
            ki_anfrage_text,
            ki_verdichten,
            forge_status,
            forge_token_setzen,
            forge_auto_setzen,
            forge_projekt,
            forge_abfragen,
            kalender_termine,
            update_pruefen,
            update_automatisch_setzen,
            oeffnen,
            tresor_liste,
            tresor_anlegen,
            tresor_feld_lesen,
            tresor_loeschen,
            sync_status,
            sync_geraete,
            sync_geraet_widerrufen,
            sync_jetzt,
            sync_register,
            sync_login
        ])
        .run(tauri::generate_context!())
        .expect("Lotse konnte nicht gestartet werden");
}
