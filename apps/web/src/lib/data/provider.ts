import type { Id, Kandidat, Notiz, Projekt, ProjektStatus, Referenz, TresorEintrag } from './types';

/**
 * Grenze zur Datenschicht. Heute von `mock.ts` implementiert (In-Memory, deterministisch,
 * für Entwicklung und Tests). Später ersetzt eine Implementierung auf Basis von
 * `lotse-core` als WebAssembly diese Schnittstelle 1:1 – die UI-Schicht ändert sich nicht.
 */
export interface DataProvider {
  listProjects(): Promise<Projekt[]>;
  getProject(id: Id): Promise<Projekt | undefined>;

  listNotes(projectId: Id): Promise<Notiz[]>;
  /** Legt eine neue Notiz an. Notizen werden nie überschrieben. */
  addNote(projectId: Id, note: Pick<Notiz, 'quelle' | 'art' | 'text'> & Partial<Pick<Notiz, 'erledigt_am'>>): Promise<Notiz>;

  /**
   * Setzt den Projektstatus. Beim Wechsel auf `pausiert` oder `wartet` ist
   * `uebergabeText` verpflichtend (siehe CONCEPT.md Abschnitt 4); der Aufrufer
   * (UI) erzwingt das, der Provider nimmt den Text entgegen und legt eine
   * Notiz vom Typ `status` (und ggf. `uebergabe`) an.
   */
  setStatus(projectId: Id, status: ProjektStatus, uebergabeText?: string): Promise<Projekt>;

  /** Alle offenen Fäden (Notizen mit `art: 'offen'` und ohne `erledigt_am`) projektübergreifend. */
  listOpenThreads(): Promise<Notiz[]>;

  listReferences(projectId: Id): Promise<Referenz[]>;
  listVaultEntries(projectId: Id): Promise<TresorEintrag[]>;

  /** Volltextsuche über Projekte, Logbücher, Referenz- und Tresor-Titel. */
  search(query: string): Promise<{
    projects: Projekt[];
    notes: Notiz[];
    references: Referenz[];
    vaultEntries: TresorEintrag[];
  }>;

  /** Hafeneinfahrt: vom Beobachter erkannte, noch nicht bestätigte Projektkandidaten. */
  listCandidates(): Promise<Kandidat[]>;
  /** Bestätigt einen Kandidaten und legt daraus ein Projekt an. */
  confirmCandidate(candidateId: Id, titel?: string): Promise<Projekt>;
}
