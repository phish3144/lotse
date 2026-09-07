import type { DataProvider } from './provider';
import type {
  Id,
  Kandidat,
  Notiz,
  Projekt,
  Pruefstatus,
  Referenz,
  TresorEintrag,
  VorlagenId,
} from './types';

// In-Memory-Mockprovider. Steht hinter der DataProvider-Schnittstelle und wird
// später 1:1 durch eine Implementierung auf Basis von lotse-core (WASM) ersetzt.
// Alle Zeitangaben sind relativ zum Ladezeitpunkt, damit "überfällig" &
// "vor N Tagen" beim Betrachten immer stimmig bleiben.

const NOW = Date.now();
const TAG_MS = 24 * 60 * 60 * 1000;

function vor(tagen: number): string {
  return new Date(NOW - tagen * TAG_MS).toISOString();
}

/** Datum (YYYY-MM-DD) relativ zu heute; negative Werte liegen in der Vergangenheit. */
function datumOffset(tagen: number): string {
  return new Date(NOW + tagen * TAG_MS).toISOString().slice(0, 10);
}

let seq = 0;
/**
 * Bereits belegte Kennungen. Wird unten mit den fest verdrahteten Beispiel-IDs gefüllt,
 * damit neu angelegte Datensätze nicht mit ihnen kollidieren – sonst trifft etwa
 * `completeThread` die falsche Notiz.
 */
const vergebeneIds = new Set<Id>();
function nextId(prefix: string): Id {
  let kandidat: Id;
  do {
    seq += 1;
    kandidat = `${prefix}_${seq.toString(36).padStart(4, '0')}`;
  } while (vergebeneIds.has(kandidat));
  vergebeneIds.add(kandidat);
  return kandidat;
}

/** Default-Erwartungsintervall je Vorlage, siehe CONCEPT.md Abschnitt 3. */
export const VORLAGEN_INTERVALL: Record<VorlagenId, number> = {
  software: 14,
  hardware_maker: 30,
  haus_garten: 60,
  kreativ: 30,
  finanzen_verwaltung: 90,
  lernen_forschung: 30,
  reise_veranstaltung: 120,
  generisch: 30,
};

// ---------------------------------------------------------------------------
// Projekte
// ---------------------------------------------------------------------------

const projects: Projekt[] = [
  {
    id: 'p_getraenkekasse',
    titel: 'Getränkekasse-App',
    kurs: 'Kleine App für die Bürogetränkekasse: Bestand buchen, Kontostand je Person, monatlicher Abgleich mit dem Vereinskonto.',
    status: 'aktiv',
    erwartungsintervall_tage: VORLAGEN_INTERVALL.software,
    tags: ['nebenprojekt', 'rust', 'axum'],
    vorlage: 'software',
    angelegt: vor(260),
    zuletzt_beruehrt: vor(20),
  },
  {
    id: 'p_3d_drucker',
    titel: '3D-Drucker-Umbau (Klipper)',
    kurs: 'Ender 3 auf Klipper-Firmware umbauen: eigenes Mainboard, Input Shaping, ruhigerer Betrieb.',
    status: 'aktiv',
    erwartungsintervall_tage: VORLAGEN_INTERVALL.hardware_maker,
    tags: ['3d-druck', 'werkstatt'],
    vorlage: 'hardware_maker',
    angelegt: vor(140),
    zuletzt_beruehrt: vor(10),
  },
  {
    id: 'p_gartenteich',
    titel: 'Gartenteich anlegen',
    kurs: 'Naturteich im hinteren Gartenteil, ca. 6 m², für Frösche und Libellen, kein Fischbesatz.',
    status: 'pausiert',
    wiedervorlage: datumOffset(-10),
    erwartungsintervall_tage: VORLAGEN_INTERVALL.haus_garten,
    tags: ['garten', 'sommerprojekt'],
    vorlage: 'haus_garten',
    angelegt: vor(200),
    zuletzt_beruehrt: vor(55),
  },
  {
    id: 'p_album3',
    titel: 'Album Nr. 3',
    kurs: 'Viertes... nein, drittes Album. Acht Stücke, ruhiger als die letzten beiden, komplett in Ableton.',
    status: 'aktiv',
    erwartungsintervall_tage: VORLAGEN_INTERVALL.kreativ,
    tags: ['musik', 'ableton'],
    vorlage: 'kreativ',
    angelegt: vor(400),
    zuletzt_beruehrt: vor(45),
  },
  {
    id: 'p_steuer2025',
    titel: 'Steuererklärung 2025',
    kurs: 'Steuererklärung für 2025, dieses Mal mit Steuerberater wegen der Nebentätigkeit.',
    status: 'wartet',
    wiedervorlage: datumOffset(20),
    erwartungsintervall_tage: VORLAGEN_INTERVALL.finanzen_verwaltung,
    tags: ['steuer', 'jährlich'],
    vorlage: 'finanzen_verwaltung',
    angelegt: vor(70),
    zuletzt_beruehrt: vor(15),
  },
  {
    id: 'p_ml_kurs',
    titel: 'Machine-Learning-Grundlagen',
    kurs: 'Onlinekurs samt Übungsaufgaben durcharbeiten, Ziel: eigenes kleines Projekt am Ende jedes Kapitels.',
    status: 'aktiv',
    erwartungsintervall_tage: VORLAGEN_INTERVALL.lernen_forschung,
    tags: ['lernen', 'ml'],
    vorlage: 'lernen_forschung',
    angelegt: vor(60),
    zuletzt_beruehrt: vor(5),
  },
  {
    id: 'p_island',
    titel: 'Islandreise 2027 planen',
    kurs: 'Zwei Wochen Ringstraße im Juni 2027, eigenes Auto, Zelt und ein paar Hütten.',
    status: 'idee',
    erwartungsintervall_tage: VORLAGEN_INTERVALL.reise_veranstaltung,
    tags: ['reise', 'island'],
    vorlage: 'reise_veranstaltung',
    angelegt: vor(30),
    zuletzt_beruehrt: vor(2),
  },
  {
    id: 'p_verein',
    titel: 'Vereinsvorstand Kassenwart',
    kurs: 'Kassenführung für den Sportverein: laufende Buchungen, Kassenbericht zur Jahreshauptversammlung.',
    status: 'aktiv',
    erwartungsintervall_tage: VORLAGEN_INTERVALL.generisch,
    tags: ['verein', 'ehrenamt'],
    vorlage: 'generisch',
    angelegt: vor(500),
    zuletzt_beruehrt: vor(40),
  },
  {
    id: 'p_postkorb',
    titel: 'Postkorb',
    kurs: 'Sammelbecken für Gedanken aus der Schnellerfassung ohne erkanntes Projekt.',
    status: 'aktiv',
    // Der Postkorb ist ein Sammelbecken, kein echtes Projekt mit Erwartung – daher
    // ein sehr langes Intervall, damit er nie als auffällig markiert wird.
    erwartungsintervall_tage: 3650,
    tags: ['postkorb'],
    vorlage: 'generisch',
    angelegt: vor(500),
    zuletzt_beruehrt: vor(2),
  },
];

export const POSTKORB_PROJEKT_ID: Id = 'p_postkorb';

// ---------------------------------------------------------------------------
// Notizen (Logbuch)
// ---------------------------------------------------------------------------

const notes: Notiz[] = [
  // Getränkekasse-App
  { id: 'n_0001', projekt_id: 'p_getraenkekasse', ts: vor(20), quelle: 'git', art: 'log', text: '3 Commits, zuletzt: Zahlungsabgleich mit Bank-API repariert.' },
  { id: 'n_0002', projekt_id: 'p_getraenkekasse', ts: vor(22), quelle: 'mcp', art: 'log', text: 'Claude-Code-Sitzung: Abrechnungslogik refaktoriert, siehe PR #42.' },
  { id: 'n_0003', projekt_id: 'p_getraenkekasse', ts: vor(25), quelle: 'mensch', art: 'offen', text: 'Stripe-Webhook: Fehlerbehandlung bei doppelten Events fehlt noch.' },
  {
    id: 'n_0004',
    projekt_id: 'p_getraenkekasse',
    ts: vor(60),
    quelle: 'mensch',
    art: 'entscheidung',
    text: 'Kontext: Zahlungen bisher nur manuell abgeglichen, das dauert jeden Monat zu lange.\nEntschieden: Stripe für Kartenzahlungen anbinden, Barkasse bleibt manuell.\nVerworfen weil: eigene Zahlungsabwicklung wäre Overkill für 15 Personen.\nNeu bewerten wenn: die Vereinsgröße sich mehr als verdoppelt.',
  },

  // 3D-Drucker-Umbau
  { id: 'n_0010', projekt_id: 'p_3d_drucker', ts: vor(10), quelle: 'cli', art: 'log', text: 'Filament PETG bestellt, Firmware auf Klipper 0.13 aktualisiert.' },
  { id: 'n_0011', projekt_id: 'p_3d_drucker', ts: vor(12), quelle: 'datei', art: 'log', text: '6 Dateien geändert in cad/, zuletzt bracket_v3.stl.' },
  { id: 'n_0012', projekt_id: 'p_3d_drucker', ts: vor(18), quelle: 'mensch', art: 'offen', text: 'Vibrationskompensation (Input Shaping) noch kalibrieren.' },

  // Gartenteich
  { id: 'n_0020', projekt_id: 'p_gartenteich', ts: vor(55), quelle: 'mensch', art: 'uebergabe', text: 'Folie liegt im Schuppen, der Aushub ist fertig. Als Nächstes: Teichrand mit Steinen verlegen, sobald es wieder regnet und der Boden weicher ist.' },
  { id: 'n_0021', projekt_id: 'p_gartenteich', ts: vor(55), quelle: 'mensch', art: 'status', text: 'Status geändert: aktiv → pausiert.' },
  { id: 'n_0022', projekt_id: 'p_gartenteich', ts: vor(58), quelle: 'mensch', art: 'offen', text: 'Pumpe fürs Frühjahr bestellen.' },

  // Album Nr. 3
  { id: 'n_0030', projekt_id: 'p_album3', ts: vor(45), quelle: 'mensch', art: 'log', text: 'Gesangsspur für Track 4 aufgenommen, zwei Takes.' },
  { id: 'n_0031', projekt_id: 'p_album3', ts: vor(50), quelle: 'datei', art: 'log', text: '9 Dateien geändert in Ableton/, zuletzt take_master.als.' },
  { id: 'n_0032', projekt_id: 'p_album3', ts: vor(70), quelle: 'mensch', art: 'offen', text: 'Mastering-Angebot vom Studio einholen.' },

  // Steuererklärung 2025
  { id: 'n_0040', projekt_id: 'p_steuer2025', ts: vor(15), quelle: 'mensch', art: 'uebergabe', text: 'Unterlagen sind beim Steuerberater. Warte auf Rückmeldung zu den Werbungskosten für die Nebentätigkeit.' },
  { id: 'n_0041', projekt_id: 'p_steuer2025', ts: vor(16), quelle: 'mensch', art: 'status', text: 'Status geändert: aktiv → wartet.' },
  { id: 'n_0042', projekt_id: 'p_steuer2025', ts: vor(25), quelle: 'import', art: 'log', text: 'Kontoauszüge 2025 importiert (12 PDFs).' },
  { id: 'n_0043', projekt_id: 'p_steuer2025', ts: vor(30), quelle: 'mensch', art: 'offen', text: 'Spendenquittung vom Verein nachreichen.' },

  // Machine-Learning-Grundlagen
  { id: 'n_0050', projekt_id: 'p_ml_kurs', ts: vor(5), quelle: 'ki', art: 'log', text: 'Zusammenfassung: Kapitel 4 (Backpropagation) durchgearbeitet, Übungsaufgaben 1–3 gelöst.' },
  {
    id: 'n_0051',
    projekt_id: 'p_ml_kurs',
    ts: vor(20),
    quelle: 'mensch',
    art: 'entscheidung',
    text: 'Kontext: Wahl zwischen Kurs A und Kurs B.\nEntschieden: Kurs B, weil praxisnäher mit mehr Übungsaufgaben.\nVerworfen weil: Kurs A zu theorielastig für den Einstieg.\nNeu bewerten wenn: Kurs B nach Kapitel 6 nicht mehr zum eigenen Tempo passt.',
  },
  { id: 'n_0052', projekt_id: 'p_ml_kurs', ts: vor(5), quelle: 'mensch', art: 'offen', text: 'Übungsaufgabe 5 (Backprop von Hand) fertig rechnen.' },

  // Islandreise 2027
  { id: 'n_0060', projekt_id: 'p_island', ts: vor(2), quelle: 'mensch', art: 'log', text: 'Grobe Route für die Ringstraße skizziert, 14 Tage im Juni.' },
  { id: 'n_0061', projekt_id: 'p_island', ts: vor(10), quelle: 'mensch', art: 'offen', text: 'Mietwagen-Anbieter vergleichen (4x4 nötig für die Hochlandpisten?).' },

  // Vereinsvorstand Kassenwart
  { id: 'n_0070', projekt_id: 'p_verein', ts: vor(40), quelle: 'mensch', art: 'log', text: 'Kassenbericht für die Herbstversammlung vorbereitet.' },
  { id: 'n_0071', projekt_id: 'p_verein', ts: vor(42), quelle: 'mcp', art: 'log', text: 'Claude-Code-Sitzung: Vereinssatzung auf Aktualität der Kassenprüfer-Klausel geprüft.' },
  { id: 'n_0072', projekt_id: 'p_verein', ts: vor(45), quelle: 'mensch', art: 'offen', text: 'Kontoauszüge Q3 mit dem Kassenbuch abgleichen.' },

  // Postkorb
  { id: 'n_0080', projekt_id: 'p_postkorb', ts: vor(2), quelle: 'cli', art: 'log', text: 'Idee: Balkonkraftwerk – Angebote vergleichen, kein @Projekt erkannt.' },
];

// ---------------------------------------------------------------------------
// Referenzen
// ---------------------------------------------------------------------------

const references: Referenz[] = [
  { id: 'r_001', projekt_id: 'p_getraenkekasse', typ: 'git_repo', ziel: 'git@github.com:beispiel/getraenkekasse.git', rolle: 'ergebnis', pruefstatus: 'ok', zuletzt_geprueft: vor(20) },
  { id: 'r_002', projekt_id: 'p_getraenkekasse', typ: 'url', ziel: 'https://getraenkekasse.internal.example', rolle: 'ergebnis', pruefstatus: 'nicht_pruefbar' },
  { id: 'r_003', projekt_id: 'p_getraenkekasse', typ: 'passwortmanager', ziel: 'Proton Pass: „Getränkekasse Admin“', rolle: 'doku', pruefstatus: 'nicht_pruefbar' },

  { id: 'r_010', projekt_id: 'p_3d_drucker', typ: 'ordner', ziel: 'D:\\Projekte\\3d-drucker-umbau', rolle: 'material', geraet_id: 'g_werkstatt_pc', pruefstatus: 'nicht_erreichbar' },
  { id: 'r_011', projekt_id: 'p_3d_drucker', typ: 'physisch', ziel: 'Werkstatt, Regal 2, Kiste „Ersatzteile Drucker“', rolle: 'material', pruefstatus: 'nicht_pruefbar' },

  { id: 'r_020', projekt_id: 'p_gartenteich', typ: 'physisch', ziel: 'Schuppen, Teichfolie und Pumpe', rolle: 'material', pruefstatus: 'nicht_pruefbar' },
  { id: 'r_021', projekt_id: 'p_gartenteich', typ: 'ordner', ziel: '~/Fotos/Garten/Teich', rolle: 'doku', pruefstatus: 'ok', zuletzt_geprueft: vor(55) },

  { id: 'r_030', projekt_id: 'p_album3', typ: 'ordner', ziel: '~/Musik/Album3/Ableton', rolle: 'material', pruefstatus: 'ok', zuletzt_geprueft: vor(45) },
  { id: 'r_031', projekt_id: 'p_album3', typ: 'url', ziel: 'https://soundcloud.example/demo-v3-privat', rolle: 'ergebnis', pruefstatus: 'nicht_erreichbar' },

  { id: 'r_040', projekt_id: 'p_steuer2025', typ: 'ordner', ziel: '~/Dokumente/Steuer/2025', rolle: 'material', pruefstatus: 'ok', zuletzt_geprueft: vor(15) },
  { id: 'r_041', projekt_id: 'p_steuer2025', typ: 'physisch', ziel: 'Aktenordner „Steuer 2025“, Büro-Regal', rolle: 'material', pruefstatus: 'nicht_pruefbar' },

  { id: 'r_050', projekt_id: 'p_ml_kurs', typ: 'url', ziel: 'https://kurs-anbieter.example/course/ml-grundlagen', rolle: 'material', pruefstatus: 'ok', zuletzt_geprueft: vor(5) },
  { id: 'r_051', projekt_id: 'p_ml_kurs', typ: 'datei', ziel: '~/Uni/ml-notizen.tex', rolle: 'doku', pruefstatus: 'ok', zuletzt_geprueft: vor(5) },

  { id: 'r_060', projekt_id: 'p_island', typ: 'url', ziel: 'https://karten.example/ringstrasse-route-entwurf', rolle: 'doku', pruefstatus: 'nicht_pruefbar' },

  { id: 'r_070', projekt_id: 'p_verein', typ: 'ordner', ziel: '~/Verein/Kassenbuch', rolle: 'material', pruefstatus: 'ok', zuletzt_geprueft: vor(40) },
  { id: 'r_071', projekt_id: 'p_verein', typ: 'passwortmanager', ziel: 'Proton Pass: „Vereinskonto Online-Banking“', rolle: 'doku', pruefstatus: 'nicht_pruefbar' },
];

// ---------------------------------------------------------------------------
// Tresor (Zugänge). `wert_verschluesselt` ist im Mock nur ein Platzhalter für
// maskierte Darstellung – es gibt hier keine echte Kryptografie, siehe README.
// ---------------------------------------------------------------------------

const vaultEntries: TresorEintrag[] = [
  {
    id: 'v_001',
    titel: 'Getränkekasse Admin-Login',
    projekt_ids: ['p_getraenkekasse'],
    stufe: 'ueberall',
    felder: [
      { name: 'Benutzername', wert_verschluesselt: '••••••••' },
      { name: 'Passwort', wert_verschluesselt: '••••••••••••' },
    ],
  },
  {
    id: 'v_002',
    titel: 'Klipper Web-Interface API-Key',
    projekt_ids: ['p_3d_drucker'],
    stufe: 'nur_desktop',
    felder: [{ name: 'API-Key', wert_verschluesselt: '••••••••••••••••' }],
  },
  {
    id: 'v_003',
    titel: 'Elster-Zugang',
    projekt_ids: ['p_steuer2025'],
    stufe: 'nur_desktop',
    felder: [{ name: 'PIN', wert_verschluesselt: '••••••' }],
  },
  {
    id: 'v_004',
    titel: 'Vereinskonto Online-Banking',
    projekt_ids: ['p_verein'],
    stufe: 'ueberall',
    felder: [
      { name: 'IBAN', wert_verschluesselt: '••••••••••••••••••••' },
      { name: 'PIN', wert_verschluesselt: '••••••' },
    ],
  },
];

// ---------------------------------------------------------------------------
// Hafeneinfahrt-Kandidaten
// ---------------------------------------------------------------------------

let candidates: Kandidat[] = [
  {
    id: 'k_001',
    titel_vorschlag: 'heizungssteuerung-esp32',
    pfad: '~/Projekte/heizungssteuerung-esp32',
    vorlage_vorschlag: 'hardware_maker',
    erkennungsmarke: 'platformio.ini',
    erkannt_am: vor(1),
  },
  {
    id: 'k_002',
    titel_vorschlag: 'rezepte-sammlung',
    pfad: '~/Dokumente/rezepte-sammlung',
    vorlage_vorschlag: 'generisch',
    erkennungsmarke: 'README.md',
    erkannt_am: vor(3),
  },
  {
    id: 'k_003',
    titel_vorschlag: '2026-urlaub-kroatien',
    pfad: '~/Fotos/2026-urlaub-kroatien',
    vorlage_vorschlag: 'kreativ',
    erkennungsmarke: 'RAW-Sammlung (312 Dateien)',
    erkannt_am: vor(6),
  },
];

// ---------------------------------------------------------------------------
// Provider-Implementierung
// ---------------------------------------------------------------------------

for (const eintrag of [...projects, ...notes, ...references, ...vaultEntries, ...candidates]) {
  vergebeneIds.add(eintrag.id);
}

function clone<T>(value: T): T {
  return structuredClone(value);
}

/**
 * Beispielwerte für die Tresor-Vorschau im Browser. In der Tauri-Hülle kommen die Werte
 * entschlüsselt aus dem Kern; hier gibt es nur Attrappen, damit die Bedienung zu sehen ist.
 */
const mockTresorWerte = new Map<string, string>([
  ['v_001|Benutzername', 'admin@getraenkekasse.example'],
  ['v_001|Passwort', 'beispiel-passwort-nicht-echt'],
  ['v_002|API-Key', 'kl_beispiel_9f2a41c7e8'],
  ['v_003|PIN', '123456'],
]);

export function createMockProvider(): DataProvider {
  return {
    async listProjects() {
      return clone(projects);
    },

    async getProject(id) {
      return clone(projects.find((p) => p.id === id));
    },

    async createProject(titel, vorlage, kurs) {
      const jetzt = new Date().toISOString();
      const project: Projekt = {
        id: nextId('p'),
        titel: titel.trim(),
        kurs: kurs?.trim() ?? '',
        status: 'idee',
        erwartungsintervall_tage: VORLAGEN_INTERVALL[vorlage],
        tags: [],
        vorlage,
        angelegt: jetzt,
        zuletzt_beruehrt: jetzt,
      };
      projects.push(project);
      return clone(project);
    },

    async saveProject(projekt) {
      const index = projects.findIndex((p) => p.id === projekt.id);
      if (index < 0) {
        throw new Error(`Unbekanntes Projekt: ${projekt.id}`);
      }
      projects[index] = clone(projekt);
      return clone(projects[index]);
    },

    async postkorb() {
      const vorhanden = projects.find((p) => p.id === POSTKORB_PROJEKT_ID);
      if (vorhanden) return clone(vorhanden);
      const jetzt = new Date().toISOString();
      const project: Projekt = {
        id: POSTKORB_PROJEKT_ID,
        titel: 'Postkorb',
        kurs: 'Gedanken ohne Zuordnung. Von hier aus einsortieren.',
        status: 'aktiv',
        erwartungsintervall_tage: 3650,
        tags: ['postkorb'],
        vorlage: 'generisch',
        angelegt: jetzt,
        zuletzt_beruehrt: jetzt,
      };
      projects.push(project);
      return clone(project);
    },

    async deleteProject(id) {
      const index = projects.findIndex((p) => p.id === id);
      if (index < 0) {
        throw new Error(`Unbekanntes Projekt: ${id}`);
      }
      projects.splice(index, 1);
      for (let i = notes.length - 1; i >= 0; i--) {
        if (notes[i].projekt_id === id) notes.splice(i, 1);
      }
      for (let i = references.length - 1; i >= 0; i--) {
        if (references[i].projekt_id === id) references.splice(i, 1);
      }
    },

    async listNotes(projectId) {
      return clone(notes.filter((n) => n.projekt_id === projectId)).sort((a, b) => b.ts.localeCompare(a.ts));
    },

    async addNote(projectId, note) {
      const project = projects.find((p) => p.id === projectId);
      if (!project) {
        throw new Error(`Unbekanntes Projekt: ${projectId}`);
      }
      const created: Notiz = {
        id: nextId('n'),
        projekt_id: projectId,
        ts: new Date().toISOString(),
        quelle: note.quelle,
        art: note.art,
        text: note.text,
        erledigt_am: note.erledigt_am,
      };
      notes.push(created);
      project.zuletzt_beruehrt = created.ts;
      return clone(created);
    },

    async completeThread(noteId) {
      const note = notes.find((n) => n.id === noteId);
      if (!note) {
        throw new Error(`Unbekannte Notiz: ${noteId}`);
      }
      note.erledigt_am = new Date().toISOString();
    },

    async moveNote(noteId, projectId) {
      const note = notes.find((n) => n.id === noteId);
      if (!note) {
        throw new Error(`Unbekannte Notiz: ${noteId}`);
      }
      if (!projects.some((p) => p.id === projectId)) {
        throw new Error(`Unbekanntes Projekt: ${projectId}`);
      }
      note.projekt_id = projectId;
      return clone(note);
    },

    async setStatus(projectId, status, uebergabeText) {
      const project = projects.find((p) => p.id === projectId);
      if (!project) {
        throw new Error(`Unbekanntes Projekt: ${projectId}`);
      }
      const brauchtUebergabe = status === 'pausiert' || status === 'wartet';
      if (brauchtUebergabe && !uebergabeText?.trim()) {
        throw new Error('Übergabenotiz ist beim Wechsel zu pausiert/wartet verpflichtend.');
      }
      const jetzt = new Date().toISOString();
      const alterStatus = project.status;
      project.status = status;
      project.zuletzt_beruehrt = jetzt;

      notes.push({
        id: nextId('n'),
        projekt_id: projectId,
        ts: jetzt,
        quelle: 'mensch',
        art: 'status',
        text: `Status geändert: ${alterStatus} → ${status}.`,
      });
      if (brauchtUebergabe && uebergabeText?.trim()) {
        notes.push({
          id: nextId('n'),
          projekt_id: projectId,
          ts: jetzt,
          quelle: 'mensch',
          art: 'uebergabe',
          text: uebergabeText.trim(),
        });
      }
      return clone(project);
    },

    async listOpenThreads() {
      return clone(notes.filter((n) => n.art === 'offen' && !n.erledigt_am)).sort((a, b) => b.ts.localeCompare(a.ts));
    },

    async listReferences(projectId) {
      return clone(references.filter((r) => r.projekt_id === projectId));
    },

    async addReference(projectId, typ, ziel, rolle) {
      const referenz: Referenz = {
        id: nextId('r'),
        projekt_id: projectId,
        typ,
        ziel: ziel.trim(),
        rolle,
        pruefstatus: typ === 'physisch' || typ === 'passwortmanager' ? 'nicht_pruefbar' : 'ok',
      };
      references.push(referenz);
      return clone(referenz);
    },

    async checkReference(id) {
      const referenz = references.find((r) => r.id === id);
      if (!referenz) {
        throw new Error(`Unbekannte Referenz: ${id}`);
      }
      // Im Browser lässt sich nichts wirklich prüfen – das ist ein ehrlicher Zustand.
      const status: Pruefstatus = referenz.typ === 'url' ? 'ok' : 'nicht_pruefbar';
      referenz.pruefstatus = status;
      referenz.zuletzt_geprueft = new Date().toISOString();
      return status;
    },

    async listVaultEntries(projectId) {
      return clone(vaultEntries.filter((v) => v.projekt_ids.includes(projectId)));
    },

    async listAllVaultEntries() {
      return clone(vaultEntries);
    },

    async addVaultEntry(titel, projektIds, stufe, felder) {
      const eintrag: TresorEintrag = {
        id: nextId('v'),
        titel: titel.trim(),
        projekt_ids: projektIds,
        stufe,
        felder: felder.map((f) => ({ name: f.name, wert_verschluesselt: '••••••••' })),
      };
      vaultEntries.push(eintrag);
      for (const f of felder) {
        mockTresorWerte.set(`${eintrag.id}|${f.name}`, f.wert);
      }
      return clone(eintrag);
    },

    async readVaultField(id, feld) {
      const wert = mockTresorWerte.get(`${id}|${feld}`);
      if (wert === undefined) {
        throw new Error('Für diesen Beispieleintrag ist kein Wert hinterlegt.');
      }
      return wert;
    },

    async deleteVaultEntry(id) {
      const index = vaultEntries.findIndex((v) => v.id === id);
      if (index < 0) {
        throw new Error(`Unbekannter Tresor-Eintrag: ${id}`);
      }
      for (const feld of vaultEntries[index].felder) {
        mockTresorWerte.delete(`${id}|${feld.name}`);
      }
      vaultEntries.splice(index, 1);
    },

    async search(query) {
      const q = query.trim().toLowerCase();
      if (!q) {
        return { projects: [], notes: [], references: [], vaultEntries: [] };
      }
      return {
        projects: clone(projects.filter((p) => p.titel.toLowerCase().includes(q) || p.kurs.toLowerCase().includes(q) || p.tags.some((t) => t.toLowerCase().includes(q)))),
        notes: clone(notes.filter((n) => n.text.toLowerCase().includes(q))),
        references: clone(references.filter((r) => r.ziel.toLowerCase().includes(q))),
        vaultEntries: clone(vaultEntries.filter((v) => v.titel.toLowerCase().includes(q))),
      };
    },

    // Im Browser gibt es kein Ziel, an das gesendet werden könnte. Ein erfundener
    // Vorschlag wäre schlimmer als eine klare Absage.
    async kiAnfrageText() {
      throw new Error('Die KI-Verdichtung gibt es nur in der Desktop-App.');
    },
    async kiVerdichten() {
      throw new Error('Die KI-Verdichtung gibt es nur in der Desktop-App.');
    },
    async listCandidates() {
      return clone(candidates);
    },

    async confirmCandidate(candidateId, titel) {
      const candidate = candidates.find((c) => c.id === candidateId);
      if (!candidate) {
        throw new Error(`Unbekannter Kandidat: ${candidateId}`);
      }
      candidates = candidates.filter((c) => c.id !== candidateId);
      const jetzt = new Date().toISOString();
      const project: Projekt = {
        id: nextId('p'),
        titel: titel?.trim() || candidate.titel_vorschlag,
        kurs: '',
        status: 'idee',
        erwartungsintervall_tage: VORLAGEN_INTERVALL[candidate.vorlage_vorschlag],
        tags: [],
        vorlage: candidate.vorlage_vorschlag,
        angelegt: jetzt,
        zuletzt_beruehrt: jetzt,
      };
      projects.push(project);
      notes.push({
        id: nextId('n'),
        projekt_id: project.id,
        ts: jetzt,
        quelle: 'import',
        art: 'log',
        text: `Aus Hafeneinfahrt bestätigt (Erkennungsmarke: ${candidate.erkennungsmarke}, Pfad: ${candidate.pfad}).`,
      });
      return clone(project);
    },

    async rejectCandidate(candidateId) {
      candidates = candidates.filter((c) => c.id !== candidateId);
    },

    async scan() {
      // Im Browser gibt es kein Dateisystem; die Beispielkandidaten bleiben, wie sie sind.
      return clone(candidates);
    },
  };
}
