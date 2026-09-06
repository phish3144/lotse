//! Tauri-Hülle um `lotse-core`.
//!
//! Gerüst. Die Kommandos spiegeln die `DataProvider`-Schnittstelle der Oberfläche
//! (`apps/web/src/lib/data/provider.ts`). Der Speicher wird nach dem Entsperren im
//! App-State gehalten; Schlüsselmaterial verlässt den Rust-Prozess nie.
//!
//! Noch offen (Phase 1): Entsperren über OS-Schlüsselbund, Ordner-Beobachter, Sync-Client,
//! Tray, Auto-Lock.

use std::sync::Mutex;

use lotse_core::store::{HafenKarte, Store};
use lotse_core::{model::*, now_ms};
use tauri::State;
use ulid::Ulid;

#[derive(Default)]
pub struct AppState {
    store: Mutex<Option<Store>>,
}

fn mit_store<T>(state: &State<AppState>, f: impl FnOnce(&mut Store) -> lotse_core::Result<T>) -> Result<T, String> {
    let mut guard = state.store.lock().map_err(|_| "Speicher gesperrt".to_string())?;
    let store = guard.as_mut().ok_or_else(|| "Nicht entsperrt".to_string())?;
    f(store).map_err(|e| e.to_string())
}

#[tauri::command]
fn hafen(state: State<AppState>) -> Result<Vec<HafenKarte>, String> {
    mit_store(&state, |s| s.hafen(now_ms()))
}

#[tauri::command]
fn projekt(state: State<AppState>, id: String) -> Result<Option<Projekt>, String> {
    let id = Ulid::from_string(&id).map_err(|e| e.to_string())?;
    mit_store(&state, |s| s.projekt(id))
}

#[tauri::command]
fn notizen(state: State<AppState>, projekt_id: String) -> Result<Vec<Notiz>, String> {
    let id = Ulid::from_string(&projekt_id).map_err(|e| e.to_string())?;
    mit_store(&state, |s| s.notizen(id))
}

#[tauri::command]
fn notiz_anlegen(state: State<AppState>, projekt_id: String, text: String, offen: bool) -> Result<Notiz, String> {
    let id = Ulid::from_string(&projekt_id).map_err(|e| e.to_string())?;
    let art = if offen { Art::Offen } else { Art::Log };
    let n = Notiz::neu(id, Quelle::Mensch, art, text, now_ms());
    mit_store(&state, |s| s.notiz_speichern(&n))?;
    Ok(n)
}

#[tauri::command]
fn status_setzen(
    state: State<AppState>,
    projekt_id: String,
    status: String,
    uebergabe: Option<String>,
    wiedervorlage: Option<String>,
) -> Result<(), String> {
    let id = Ulid::from_string(&projekt_id).map_err(|e| e.to_string())?;
    let st = Status::parse(&status).ok_or_else(|| format!("Unbekannter Status {status}"))?;
    mit_store(&state, |s| {
        s.status_setzen(id, st, uebergabe.as_deref(), wiedervorlage.as_deref(), Quelle::Mensch)
    })
}

#[tauri::command]
fn offene_faeden(state: State<AppState>) -> Result<Vec<Notiz>, String> {
    mit_store(&state, |s| s.offene_faeden(None))
}

#[tauri::command]
fn suche(state: State<AppState>, anfrage: String) -> Result<Vec<lotse_core::store::Treffer>, String> {
    mit_store(&state, |s| s.suche(&anfrage))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            hafen,
            projekt,
            notizen,
            notiz_anlegen,
            status_setzen,
            offene_faeden,
            suche
        ])
        .run(tauri::generate_context!())
        .expect("Lotse konnte nicht gestartet werden");
}
