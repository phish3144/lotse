import type { Auffaelligkeit } from './brief';
import type { NotizQuelle, Pruefstatus, ProjektStatus } from './data/types';

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
  return new Date(iso).toLocaleDateString('de-DE', { year: 'numeric', month: 'long', day: 'numeric' });
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
