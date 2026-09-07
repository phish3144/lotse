import type {
  Id,
  Kandidat,
  Notiz,
  NotizArt,
  Projekt,
  ProjektStatus,
  Pruefstatus,
  Referenz,
  ReferenzRolle,
  ReferenzTyp,
  TresorEintrag,
  TresorStufe,
  VorlagenId,
} from './types';

/**
 * Grenze zur Datenschicht. Heute von `mock.ts` implementiert (In-Memory, deterministisch,
 * für Entwicklung und Tests) und von `tauri.ts` gegen den Rust-Kern. Später ersetzt eine
 * Implementierung auf Basis von `lotse-core` als WebAssembly diese Schnittstelle 1:1 –
 * die UI-Schicht ändert sich nicht.
 *
 * Sync und Konto liegen bewusst nicht hier: das sind Belange der Hülle, nicht der Daten
 * (siehe `tauri.ts`).
 */
export interface DataProvider {
  listProjects(): Promise<Projekt[]>;
  getProject(id: Id): Promise<Projekt | undefined>;
  /** Legt ein Projekt an. Das Erwartungsintervall kommt aus der Vorlage. */
  createProject(titel: string, vorlage: VorlagenId, kurs?: string): Promise<Projekt>;
  /** Schreibt einen geänderten Projektkopf zurück (Kurs, Titel, Tags, Intervall, Wiedervorlage). */
  saveProject(projekt: Projekt): Promise<Projekt>;
  /** Auffangprojekt für Gedanken ohne Zuordnung. Wird beim ersten Zugriff angelegt. */
  postkorb(): Promise<Projekt>;

  listNotes(projectId: Id): Promise<Notiz[]>;
  /** Legt eine neue Notiz an. Notizen werden nie überschrieben. */
  addNote(projectId: Id, note: Pick<Notiz, 'quelle' | 'art' | 'text'> & Partial<Pick<Notiz, 'erledigt_am'>>): Promise<Notiz>;
  /** Hakt einen offenen Faden ab. Der Eintrag bleibt im Logbuch stehen. */
  completeThread(noteId: Id): Promise<void>;

  /**
   * Setzt den Projektstatus. Beim Wechsel auf `pausiert` oder `wartet` ist
   * `uebergabeText` verpflichtend (siehe CONCEPT.md Abschnitt 4); der Aufrufer
   * (UI) erzwingt das, der Provider nimmt den Text entgegen und legt eine
   * Notiz vom Typ `status` (und ggf. `uebergabe`) an.
   */
  setStatus(projectId: Id, status: ProjektStatus, uebergabeText?: string, wiedervorlage?: string): Promise<Projekt>;

  /** Alle offenen Fäden (Notizen mit `art: 'offen'` und ohne `erledigt_am`) projektübergreifend. */
  listOpenThreads(): Promise<Notiz[]>;

  listReferences(projectId: Id): Promise<Referenz[]>;
  addReference(projectId: Id, typ: ReferenzTyp, ziel: string, rolle: ReferenzRolle): Promise<Referenz>;
  /** Prüft, ob eine Referenz noch erreichbar ist. `nicht_pruefbar` ist ein ehrlicher Zustand. */
  checkReference(id: Id): Promise<Pruefstatus>;

  /** Tresor-Einträge eines Projekts. Werte sind hier immer verdeckt. */
  listVaultEntries(projectId: Id): Promise<TresorEintrag[]>;
  /** Alle Tresor-Einträge, projektübergreifend. */
  listAllVaultEntries(): Promise<TresorEintrag[]>;
  addVaultEntry(
    titel: string,
    projektIds: Id[],
    stufe: TresorStufe,
    felder: { name: string; wert: string }[],
  ): Promise<TresorEintrag>;
  /**
   * Entschlüsselt genau ein Feld für genau einen Blick. Werte werden nie auf Vorrat
   * geladen und nie im Provider gehalten.
   */
  readVaultField(id: Id, feld: string): Promise<string>;
  /** Löschen vernichtet den Schlüssel des Eintrags (Crypto-Shredding). */
  deleteVaultEntry(id: Id): Promise<void>;

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
  /** Verwirft einen Kandidaten dauerhaft; er taucht bei künftigen Scans nicht wieder auf. */
  rejectCandidate(candidateId: Id): Promise<void>;
  /** Durchsucht Wurzelordner nach Erkennungsmarken und merkt die Funde als Kandidaten. */
  scan(wurzeln: string[]): Promise<Kandidat[]>;
}

/** Notiz-Arten, die in der Oberfläche von Hand erfasst werden können. */
export const ERFASSBARE_ARTEN: NotizArt[] = ['log', 'offen', 'entscheidung'];
