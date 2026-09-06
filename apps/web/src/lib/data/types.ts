// Datenmodell nach docs/CONCEPT.md Abschnitt 3. Feldnamen bewusst auf Deutsch,
// wie im Konzeptdokument festgelegt. Dies ist die Grenze zum späteren
// lotse-core (Rust/WASM) – die Typen hier müssen mit dessen Modell übereinstimmen.

/** ULID als String. Kein eigener Typ-Wrapper, um Reibung mit dem Rust-Kern zu vermeiden. */
export type Id = string;

/** ISO-8601-Zeitstempel (UTC). */
export type Zeitstempel = string;

/** Feste, kleine, nicht anklagende Statuswerte. `eingemottet` heißt bewusst beendet, nicht gescheitert. */
export type ProjektStatus =
  | 'idee'
  | 'aktiv'
  | 'pausiert'
  | 'wartet'
  | 'abgeschlossen'
  | 'eingemottet';

/** Kennung einer der acht Start-Vorlagen. Nur informativ, das Datenmodell bleibt für alle gleich. */
export type VorlagenId =
  | 'software'
  | 'hardware_maker'
  | 'haus_garten'
  | 'kreativ'
  | 'finanzen_verwaltung'
  | 'lernen_forschung'
  | 'reise_veranstaltung'
  | 'generisch';

export interface Projekt {
  id: Id;
  titel: string;
  /** 1–2 Sätze in eigenen Worten: worum geht es, was ist das Ziel. */
  kurs: string;
  status: ProjektStatus;
  /** Optionales Datum, für `pausiert` und `wartet`. ISO-Datum (YYYY-MM-DD). */
  wiedervorlage?: string;
  /** Ab wann Stille auffällig ist; Default aus der Vorlage. */
  erwartungsintervall_tage: number;
  tags: string[];
  vorlage: VorlagenId;
  /** Optionale Projekt-ID, falls "Neues Projekt aus bestehendem". */
  abgeleitet_von?: Id;
  angelegt: Zeitstempel;
  zuletzt_beruehrt: Zeitstempel;
}

/** Herkunft einer Notiz. */
export type NotizQuelle = 'mensch' | 'cli' | 'datei' | 'git' | 'import' | 'mcp' | 'ki';

/**
 * Faden, Logbuch-Eintrag, Entscheidung und Übergabenotiz sind bewusst eine Entität
 * mit einem Art-Feld – eine Eingabezeile, keine Typ-Frage.
 */
export type NotizArt = 'log' | 'offen' | 'entscheidung' | 'status' | 'uebergabe';

/** Notiz (Logbuch-Eintrag) – die wichtigste Entität. Notizen werden nie überschrieben. */
export interface Notiz {
  id: Id;
  projekt_id: Id;
  ts: Zeitstempel;
  quelle: NotizQuelle;
  art: NotizArt;
  /** Markdown im Kern, wird im UI vorerst als Klartext mit Absätzen dargestellt. */
  text: string;
  /** Nur für `art: 'offen'`: wann der Faden erledigt wurde. */
  erledigt_am?: Zeitstempel;
}

export type ReferenzTyp =
  | 'ordner'
  | 'git_repo'
  | 'url'
  | 'datei'
  | 'physisch'
  | 'geraet'
  | 'passwortmanager';

export type ReferenzRolle = 'material' | 'ergebnis' | 'doku';

/** `nicht_pruefbar` ist ein ehrlicher Zustand, kein Fehler. */
export type Pruefstatus = 'ok' | 'nicht_erreichbar' | 'nicht_pruefbar';

export interface Referenz {
  id: Id;
  projekt_id: Id;
  typ: ReferenzTyp;
  /** Pfad, URL oder Ortsbeschreibung ("Keller, Regal 3, blaue Kiste"). */
  ziel: string;
  rolle: ReferenzRolle;
  /** Für Pfade: auf welchem Gerät der Pfad gilt. */
  geraet_id?: Id;
  zuletzt_geprueft?: Zeitstempel;
  pruefstatus: Pruefstatus;
}

/** Zwei Tresor-Stufen, siehe THREAT_MODEL.md. */
export type TresorStufe = 'ueberall' | 'nur_desktop';

export interface TresorFeld {
  name: string;
  /** Verschlüsselter Wert. Im Mock nur als Platzhalter, nie im Klartext im UI-Layer. */
  wert_verschluesselt: string;
}

export interface TresorEintrag {
  id: Id;
  titel: string;
  /** Titel und Zuordnung im Klartext, damit Suche und Cockpit funktionieren. */
  projekt_ids: Id[];
  stufe: TresorStufe;
  felder: TresorFeld[];
}

export interface Geraet {
  id: Id;
  name: string;
  plattform: string;
  angelegt: Zeitstempel;
  zuletzt_sync?: Zeitstempel;
}

/** Von der Hafeneinfahrt erkannter, noch nicht bestätigter Projektkandidat. */
export interface Kandidat {
  id: Id;
  /** Vorgeschlagener Titel, meist der Ordnername. */
  titel_vorschlag: string;
  /** Pfad oder Ortsbeschreibung, an der die Erkennungsmarke gefunden wurde. */
  pfad: string;
  vorlage_vorschlag: VorlagenId;
  /** Erkennungsmarke, die zum Fund geführt hat, z. B. ".git", "Cargo.toml". */
  erkennungsmarke: string;
  erkannt_am: Zeitstempel;
}
