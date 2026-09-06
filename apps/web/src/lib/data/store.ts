import { createMockProvider, POSTKORB_PROJEKT_ID } from './mock';
import type { DataProvider } from './provider';
import { createTauriProvider, inTauri } from './tauri';

// Einzige Provider-Instanz für die App. In der Tauri-Hülle spricht sie den Rust-Kern an,
// im Browser (bis der WebAssembly-Kern da ist) die Beispieldaten.
export const provider: DataProvider = inTauri() ? createTauriProvider() : createMockProvider();
export const echteDaten = inTauri();

export { POSTKORB_PROJEKT_ID };
