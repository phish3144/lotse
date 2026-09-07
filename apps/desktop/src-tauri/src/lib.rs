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
}

type R<T> = Result<T, String>;

fn fehler(e: Error) -> String {
    e.to_string()
}

fn home(state: &State<AppState>) -> R<PathBuf> {
    state
        .home
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())?
        .clone()
        .ok_or_else(|| "Datenordner unbekannt".to_string())
}

fn mit<T>(state: &State<AppState>, f: impl FnOnce(&mut Sitzung) -> lotse_core::Result<T>) -> R<T> {
    let mut guard = state
        .sitzung
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())?;
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
    let entsperrt = state.sitzung.lock().map(|g| g.is_some()).unwrap_or(false);
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
    *state
        .sitzung
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())? = Some(Sitzung {
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
    *state
        .sitzung
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())? = Some(Sitzung {
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

    let neues_konto = konto::Konto {
        header: w.header,
        geraet_id: k.geraet_id,
        geraet_name: k.geraet_name.clone(),
    };
    konto::schreiben(&h, &neues_konto).map_err(fehler)?;
    let store = lotse_core::store::Store::open(&h.join(konto::DB_DATEI), &ak, k.geraet_id)
        .map_err(fehler)?;
    let vault = match konto::desktop_key_laden(k.geraet_id).map_err(fehler)? {
        Some(dk) => VaultKeys::with_desktop_key(&ak, &dk),
        None => VaultKeys::from_account_key(&ak),
    };
    *state
        .sitzung
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())? = Some(Sitzung {
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

    // Wenn das Konto beim Dienst liegt, muss er den Wechsel mitbekommen – sonst passt
    // der dort gespeicherte Wiederherstellungscode nicht mehr zum Konto.
    {
        let mut guard = state
            .sitzung
            .lock()
            .map_err(|_| "Zustand gesperrt".to_string())?;
        let s = guard
            .as_mut()
            .ok_or_else(|| "Nicht entsperrt".to_string())?;
        // Ohne eingerichteten Abgleich gibt es nichts zu melden; das ist kein Fehler.
        if let Ok(client) = sync_client::client_aus_store(&s.store) {
            client
                .passwort_wechseln(&alter_auth, &w.auth_key, &w.recovery_auth_key, &w.header)
                .map_err(fehler)?;
        }
        s.auth_key = w.auth_key;
    }

    konto::schreiben(
        &h,
        &konto::Konto {
            header: w.header,
            geraet_id: k.geraet_id,
            geraet_name: k.geraet_name,
        },
    )
    .map_err(fehler)?;

    Ok(w.neuer_code.map(|c| c.display().to_string()))
}

#[tauri::command]
fn sperren(state: State<AppState>) -> R<()> {
    // Erst den Beobachter anhalten: er greift sonst weiter auf die Sitzung zu.
    beobachter_stoppen(state.clone())?;
    *state
        .sitzung
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())? = None;
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
    let mut guard = state
        .sitzung
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())?;
    let s = guard
        .as_mut()
        .ok_or_else(|| "Nicht entsperrt".to_string())?;
    let b = lotse_core::export::bundle::schreiben(&s.store, Some(&s.vault), &passphrase, &pfad)
        .map_err(fehler)?;
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
    let laeuft = state
        .beobachter
        .lock()
        .map(|g| g.is_some())
        .unwrap_or(false);
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

    beobachter_stoppen(state.clone())?;
    mit(&state, |s| {
        lotse_core::watcher::wurzeln_speichern(&s.store, &pfade)
    })?;

    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_thread = stop.clone();
    let pfade_thread = pfade.clone();

    std::thread::spawn(move || {
        let zustand = app.state::<AppState>();

        // Start braucht den Speicher nur lesend und nur kurz.
        let mut b = {
            let Ok(mut guard) = zustand.sitzung.lock() else {
                return;
            };
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
            let Ok(mut guard) = zustand.sitzung.lock() else {
                return false;
            };
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

        if !schreiben(&mut b) {
            return;
        }
        while !stop_thread.load(Ordering::SeqCst) {
            // Ohne Sperre warten, sonst stünde die Oberfläche die ganze Zeit an.
            b.verarbeiten(std::time::Duration::from_secs(20));
            if stop_thread.load(Ordering::SeqCst) || !schreiben(&mut b) {
                break;
            }
        }
        // Eigenen Eintrag räumen, damit der Status nicht „läuft“ behauptet. Nur den
        // eigenen: nach Anhalten und sofortigem Neustart läuft schon ein anderer
        // Thread, und dessen Eintrag darf dieser hier nicht wegräumen.
        {
            let ergebnis = zustand.beobachter.lock();
            if let Ok(mut g) = ergebnis {
                let ist_meiner = g
                    .as_ref()
                    .is_some_and(|h| std::sync::Arc::ptr_eq(&h.stop, &stop_thread));
                if ist_meiner {
                    *g = None;
                }
            }
        }
    });

    *state
        .beobachter
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())? = Some(BeobachterHandle { stop });

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
    if let Some(h) = state
        .beobachter
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())?
        .take()
    {
        h.stop.store(true, Ordering::SeqCst);
    }
    Ok(())
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
    *state
        .sitzung
        .lock()
        .map_err(|_| "Zustand gesperrt".to_string())? = Some(Sitzung {
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
            if let Ok(mut h) = app.state::<AppState>().home.lock() {
                *h = Some(home);
            }
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
