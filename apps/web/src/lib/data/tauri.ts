import { invoke } from '@tauri-apps/api/core';
import type { DataProvider } from './provider';
import type {
  Id,
  Kandidat,
  Notiz,
  NotizArt,
  Projekt,
  ProjektStatus,
  Pruefstatus,
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
const ms = (isoText: string): number => new Date(isoText).getTime();

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

export interface SyncStatus {
  eingerichtet: boolean;
  url?: string;
  email?: string;
  ausstehend: number;
  last_server_seq: number;
  last_sync_ms?: number;
}

export interface SyncErgebnis {
  gepusht: number;
  uebernommen: number;
  verworfen: number;
  konflikte: number;
}

/**
 * Abgleich zwischen Geräten. Belang der Hülle, nicht der Datenschicht – deshalb neben
 * `konto` und nicht im `DataProvider`.
 */
export const sync = {
  status: () => invoke<SyncStatus>('sync_status'),
  jetzt: () => invoke<SyncErgebnis>('sync_jetzt'),
  registrieren: (url: string, email: string, code: string) => invoke<void>('sync_register', { url, email, code }),
};

/** Ist auf diesem Gerät die Tresor-Stufe »nur Desktop« lesbar? */
export const kannNurDesktop = () => invoke<boolean>('kann_nur_desktop');

/**
 * Zugriffe auf das System, die es nur in der Hülle gibt. Im Browser sind sie nicht
 * verfügbar; die Oberfläche blendet die zugehörigen Knöpfe dort aus.
 */
export const system = {
  /** Systemdialog zur Ordnerwahl. `null`, wenn abgebrochen. */
  ordnerWaehlen: () => invoke<string | null>('ordner_waehlen'),
  /** Öffnet Ordner, Datei oder URL im System – nur auf ausdrücklichen Klick. */
  oeffnen: (ziel: string) => invoke<void>('oeffnen', { ziel }),
};

export function createTauriProvider(): DataProvider {
  return {
    async listProjects() {
      return (await invoke<RohProjekt[]>('projekte')).map(projekt);
    },
    async getProject(id) {
      const p = await invoke<RohProjekt | null>('projekt', { id });
      return p ? projekt(p) : undefined;
    },
    async createProject(titel, vorlage, kurs) {
      return projekt(await invoke<RohProjekt>('projekt_anlegen', { titel, vorlage, kurs: kurs ?? null }));
    },
    async saveProject(p) {
      const roh: RohProjekt = { ...p, angelegt: ms(p.angelegt), zuletzt_beruehrt: ms(p.zuletzt_beruehrt) };
      return projekt(await invoke<RohProjekt>('projekt_speichern', { projekt: roh }));
    },
    async postkorb() {
      return projekt(await invoke<RohProjekt>('postkorb'));
    },
    async deleteProject(id) {
      await invoke<void>('projekt_loeschen', { id });
    },
    async listNotes(projectId) {
      return (await invoke<RohNotiz[]>('notizen', { projektId: projectId })).map(notiz);
    },
    async addNote(projectId, note) {
      const art: NotizArt = note.art;
      return notiz(await invoke<RohNotiz>('notiz_anlegen', { projektId: projectId, text: note.text, art }));
    },
    async completeThread(noteId) {
      await invoke<void>('faden_erledigen', { id: noteId });
    },
    async moveNote(noteId, projectId) {
      return notiz(await invoke<RohNotiz>('notiz_verschieben', { id: noteId, projektId: projectId }));
    },
    async setStatus(projectId, status: ProjektStatus, uebergabeText, wiedervorlage) {
      return projekt(
        await invoke<RohProjekt>('status_setzen', {
          projektId: projectId,
          status,
          uebergabe: uebergabeText ?? null,
          wiedervorlage: wiedervorlage ?? null,
        }),
      );
    },
    async listOpenThreads() {
      return (await invoke<RohNotiz[]>('offene_faeden')).map(notiz);
    },
    async listReferences(projectId) {
      return (await invoke<RohReferenz[]>('referenzen', { projektId: projectId })).map(referenz);
    },
    async addReference(projectId, typ, ziel, rolle) {
      return referenz(await invoke<RohReferenz>('referenz_anlegen', { projektId: projectId, typ, ziel, rolle }));
    },
    async checkReference(id) {
      return invoke<Pruefstatus>('referenz_pruefen', { id });
    },
    async listVaultEntries(projectId) {
      return (await invoke<RohTresorEintrag[]>('tresor_liste', { projektId: projectId })).map(tresor);
    },
    async listAllVaultEntries() {
      return (await invoke<RohTresorEintrag[]>('tresor_liste', { projektId: null })).map(tresor);
    },
    async addVaultEntry(titel, projektIds, stufe, felder) {
      // Der Kern nimmt die Felder als Paare entgegen; die Klartextwerte gehen genau
      // einmal über diese Grenze und werden hier nicht behalten.
      const paare = felder.map((f) => [f.name, f.wert] as [string, string]);
      return tresor(await invoke<RohTresorEintrag>('tresor_anlegen', { titel, projektIds, stufe, felder: paare }));
    },
    async readVaultField(id, feld) {
      return tresorFeldLesen(id, feld);
    },
    async deleteVaultEntry(id) {
      await invoke<void>('tresor_loeschen', { id });
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
    async rejectCandidate(candidateId) {
      await invoke<void>('kandidat_verwerfen', { pfad: candidateId });
    },
    async scan(wurzeln) {
      return (await invoke<RohKandidat[]>('scan', { wurzeln })).map(kandidat);
    },
  };
}
