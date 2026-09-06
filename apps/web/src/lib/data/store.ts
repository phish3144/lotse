import { createMockProvider, POSTKORB_PROJEKT_ID } from './mock';
import type { DataProvider } from './provider';

// Einzige Provider-Instanz für die App. Später wird `createMockProvider()`
// durch eine Implementierung ersetzt, die `lotse-core` als WebAssembly anspricht –
// die Schnittstelle (DataProvider) bleibt gleich, dieser Import ist die einzige
// Stelle, die sich dafür ändern muss.
export const provider: DataProvider = createMockProvider();

export { POSTKORB_PROJEKT_ID };
