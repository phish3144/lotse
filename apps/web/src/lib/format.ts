import type { Auffaelligkeit } from './brief';
import type {
  NotizArt,
  NotizQuelle,
  Pruefstatus,
  ProjektStatus,
  ReferenzRolle,
  ReferenzTyp,
  TresorStufe,
  VorlagenId,
} from './data/types';

const TAG_MS = 24 * 60 * 60 * 1000;

/** "heute", "vor 1 Tag", "vor 12 Tagen" – für Karten und das Logbuch. */
export function alterInTagenText(iso: string, jetzt: Date = new Date()): string {
  const tage = Math.floor((jetzt.getTime() - new Date(iso).getTime()) / TAG_MS);
  if (tage <= 0) return 'heute';
  if (tage === 1) return 'vor 1 Tag';
  return `vor ${tage} Tagen`;
}

/** Wie `alterInTagenText`, aber ausgehend von einer bereits berechneten Tageszahl. */
export function tageText(tage: number): string {
  if (tage <= 0) return 'heute';
  if (tage === 1) return 'vor 1 Tag';
  return `vor ${tage} Tagen`;
}

export function datumText(iso: string): string {
  // Ein reines Datum (JJJJ-MM-TT) ist ein Kalendertag, kein Zeitpunkt. `new Date` läse
  // es als Mitternacht UTC; westlich davon stünde dann der Vortag da.
  const nurDatum = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  const d = nurDatum
    ? new Date(Number(nurDatum[1]), Number(nurDatum[2]) - 1, Number(nurDatum[3]))
    : new Date(iso);
  return d.toLocaleDateString('de-DE', { year: 'numeric', month: 'long', day: 'numeric' });
}

export const STATUS_LABEL: Record<ProjektStatus, string> = {
  idee: 'Idee',
  aktiv: 'Aktiv',
  pausiert: 'Pausiert',
  wartet: 'Wartet',
  abgeschlossen: 'Abgeschlossen',
  eingemottet: 'Eingemottet',
};

export const QUELLE_LABEL: Record<NotizQuelle, string> = {
  mensch: 'Mensch',
  cli: 'CLI',
  datei: 'Datei',
  git: 'Git',
  import: 'Import',
  mcp: 'MCP',
  ki: 'KI',
};

export const PRUEFSTATUS_LABEL: Record<Pruefstatus, string> = {
  ok: 'ok',
  nicht_erreichbar: 'nicht erreichbar',
  nicht_pruefbar: 'nicht prüfbar',
};

export const AUFFAELLIGKEIT_LABEL: Record<Auffaelligkeit, string> = {
  ruhig: 'ruhig',
  auffaellig: 'auffällig',
  ueberfaellig: 'überfällig',
};

export const ART_LABEL: Record<NotizArt, string> = {
  log: 'Notiz',
  offen: 'Offener Faden',
  entscheidung: 'Entscheidung',
  status: 'Statuswechsel',
  uebergabe: 'Übergabe',
};

export const VORLAGEN_LABEL: Record<VorlagenId, string> = {
  software: 'Software',
  hardware_maker: 'Hardware & Maker',
  haus_garten: 'Haus & Garten',
  kreativ: 'Kreativ',
  finanzen_verwaltung: 'Finanzen & Verwaltung',
  lernen_forschung: 'Lernen & Forschung',
  reise_veranstaltung: 'Reise & Veranstaltung',
  generisch: 'Generisch',
};

export const REFERENZ_TYP_LABEL: Record<ReferenzTyp, string> = {
  ordner: 'Ordner',
  git_repo: 'Git-Repo',
  url: 'URL',
  datei: 'Datei',
  physisch: 'Physisch',
  geraet: 'Gerät',
  passwortmanager: 'Passwortmanager',
  anhang: 'Anhang',
};

/**
 * Typen, die sich in der Oberfläche anlegen lassen. `anhang` fehlt bewusst: dafür
 * braucht es einen Datei-Upload, den es noch nicht gibt.
 */
export const ANLEGBARE_REFERENZ_TYPEN: ReferenzTyp[] = [
  'ordner',
  'git_repo',
  'url',
  'datei',
  'physisch',
  'geraet',
  'passwortmanager',
];

export const REFERENZ_ROLLE_LABEL: Record<ReferenzRolle, string> = {
  material: 'Material',
  ergebnis: 'Ergebnis',
  doku: 'Doku',
};

export const STUFE_LABEL: Record<TresorStufe, string> = {
  ueberall: 'überall',
  nur_desktop: 'nur Desktop',
};
