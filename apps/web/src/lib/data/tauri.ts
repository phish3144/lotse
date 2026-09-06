import { invoke } from '@tauri-apps/api/core';
import type { DataProvider } from './provider';
import type {
  Id,
  Kandidat,
  Notiz,
  NotizArt,
  Projekt,
  ProjektStatus,
  Referenz,
  TresorEintrag,
  VorlagenId,
} from './types';

/**
 * Datenprovider gegen den Rust-Kern in der Tauri-Hülle (`apps/desktop/src-tauri/src/lib.rs`).
 * Der Kern liefert Zeitstempel als Unix-Millisekunden; die Oberfläche arbeitet mit
 * ISO-Strings. Die Umrechnung passiert ausschließlich hier.
 */

// Rohformen, wie der Kern sie serialisiert.
interface RohProjekt extends Omit<Projekt, 'angelegt' | 'zuletzt_beruehrt'> {
  angelegt: number;
  zuletzt_beruehrt: number;
}
interface RohNotiz extends Omit<Notiz, 'ts' | 'erledigt_am'> {
  ts: number;
  erledigt_am?: number;
}
interface RohReferenz extends Omit<Referenz, 'zuletzt_geprueft'> {
  zuletzt_geprueft?: number;
}
interface RohTresorEintrag {
  id: Id;
  titel: string;
  projekt_ids: Id[];
  stufe: 'ueberall' | 'nur_desktop';
  felder: { name: string }[];
}
interface RohKandidat {
  pfad: string;
  name: string;
  vorlage: VorlagenId;
  marken: string[];
  hat_git: boolean;
}
interface RohTreffer {
  kind: 'projekt' | 'notiz' | 'referenz' | 'tresor';
  ref_id: Id;
  projekt_id?: Id;
  ausschnitt: string;
}

const iso = (ms: number): string => new Date(ms).toISOString();
const isoOpt = (ms?: number): string | undefined => (ms === undefined || ms === null ? undefined : iso(ms));

const projekt = (p: RohProjekt): Projekt => ({ ...p, angelegt: iso(p.angelegt), zuletzt_beruehrt: iso(p.zuletzt_beruehrt) });
const notiz = (n: RohNotiz): Notiz => ({ ...n, ts: iso(n.ts), erledigt_am: isoOpt(n.erledigt_am) });
const referenz = (r: RohReferenz): Referenz => ({ ...r, zuletzt_geprueft: isoOpt(r.zuletzt_geprueft) });
const tresor = (e: RohTresorEintrag): TresorEintrag => ({
  id: e.id,
  titel: e.titel,
  projekt_ids: e.projekt_ids,
  stufe: e.stufe,
  felder: e.felder.map((f) => ({ name: f.name, wert_verschluesselt: '••••••' })),
});
const kandidat = (k: RohKandidat): Kandidat => ({
  id: k.pfad,
  titel_vorschlag: k.name,
  pfad: k.pfad,
  vorlage_vorschlag: k.vorlage,
  erkennungsmarke: k.marken[0] ?? (k.hat_git ? '.git' : ''),
  erkannt_am: new Date().toISOString(),
});

/** Läuft die Oberfläche in der Tauri-Hülle? */
export function inTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export interface KontoStatus {
  eingerichtet: boolean;
  entsperrt: boolean;
  geraet?: string;
  home: string;
}

export interface Geheimnisse {
  wiederherstellungscode: string;
  desktop_schluessel: string;
  schluesselbund: boolean;
}

/** Konto-Befehle, die vor dem Datenprovider liegen. */
export const konto = {
  status: () => invoke<KontoStatus>('konto_status'),
  einrichten: (passwort: string, geraet: string) => invoke<Geheimnisse>('einrichten', { passwort, geraet }),
  codePruefen: (code: string) => invoke<boolean>('wiederherstellungscode_pruefen', { code }),
  entsperren: (passwort: string, desktopSchluessel?: string) =>
    invoke<void>('entsperren', { passwort, desktopSchluessel: desktopSchluessel ?? null }),
  sperren: () => invoke<void>('sperren'),
  syncLogin: (url: string, email: string, passwort: string, geraet: string) =>
    invoke<void>('sync_login', { url, email, passwort, geraet }),
};

/** Tresor-Werte werden nie im Provider gehalten; genau ein Feld auf Klick. */
export const tresorFeldLesen = (id: Id, feld: string) => invoke<string>('tresor_feld_lesen', { id, feld });

export function createTauriProvider(): DataProvider {
  return {
    async listProjects() {
      return (await invoke<RohProjekt[]>('projekte')).map(projekt);
    },
    async getProject(id) {
      const p = await invoke<RohProjekt | null>('projekt', { id });
      return p ? projekt(p) : undefined;
    },
    async listNotes(projectId) {
      return (await invoke<RohNotiz[]>('notizen', { projektId: projectId })).map(notiz);
    },
    async addNote(projectId, note) {
      const art: NotizArt = note.art;
      return notiz(await invoke<RohNotiz>('notiz_anlegen', { projektId: projectId, text: note.text, art }));
    },
    async setStatus(projectId, status: ProjektStatus, uebergabeText) {
      return projekt(
        await invoke<RohProjekt>('status_setzen', {
          projektId: projectId,
          status,
          uebergabe: uebergabeText ?? null,
          wiedervorlage: null,
        }),
      );
    },
    async listOpenThreads() {
      return (await invoke<RohNotiz[]>('offene_faeden')).map(notiz);
    },
    async listReferences(projectId) {
      return (await invoke<RohReferenz[]>('referenzen', { projektId: projectId })).map(referenz);
    },
    async listVaultEntries(projectId) {
      return (await invoke<RohTresorEintrag[]>('tresor_liste', { projektId: projectId })).map(tresor);
    },
    async search(query) {
      const treffer = await invoke<RohTreffer[]>('suche', { anfrage: query });
      const projects: Projekt[] = [];
      const notes: Notiz[] = [];
      const references: Referenz[] = [];
      const vaultEntries: TresorEintrag[] = [];
      const refCache = new Map<Id, Referenz[]>();
      for (const t of treffer) {
        if (t.kind === 'projekt') {
          const p = await invoke<RohProjekt | null>('projekt', { id: t.ref_id });
          if (p) projects.push(projekt(p));
        } else if (t.kind === 'notiz') {
          const n = await invoke<RohNotiz | null>('notiz', { id: t.ref_id });
          if (n) notes.push(notiz(n));
        } else if (t.kind === 'referenz' && t.projekt_id) {
          let liste = refCache.get(t.projekt_id);
          if (!liste) {
            liste = (await invoke<RohReferenz[]>('referenzen', { projektId: t.projekt_id })).map(referenz);
            refCache.set(t.projekt_id, liste);
          }
          const r = liste.find((x) => x.id === t.ref_id);
          if (r) references.push(r);
        } else if (t.kind === 'tresor') {
          const alle = await invoke<RohTresorEintrag[]>('tresor_liste', { projektId: null });
          const e = alle.find((x) => x.id === t.ref_id);
          if (e) vaultEntries.push(tresor(e));
        }
      }
      return { projects, notes, references, vaultEntries };
    },
    async listCandidates() {
      return (await invoke<RohKandidat[]>('kandidaten')).map(kandidat);
    },
    async confirmCandidate(candidateId, titel) {
      return projekt(await invoke<RohProjekt>('kandidat_uebernehmen', { pfad: candidateId, titel: titel ?? null }));
    },
  };
}
