import type { Notiz, NotizQuelle, Projekt } from './data/types';

// Reine, seiteneffektfreie Funktionen für den "Wo war ich"-Brief und die
// Auffälligkeit eines Projekts. Siehe docs/CONCEPT.md Abschnitt 4 und 5.
// Bewusst getrennt vom Provider, damit sich das Verhalten ohne Mock- oder
// WASM-Anbindung testen lässt.

export type Auffaelligkeit = 'ruhig' | 'auffaellig' | 'ueberfaellig';

export interface WoWarIchBrief {
  /** Tage seit dem letzten Logbuch-Eintrag (oder `zuletzt_beruehrt`, falls es keine Notizen gibt). */
  tageSeitLetztemKontakt: number;
  /** Die zuletzt geschriebene Übergabenotiz, im Wortlaut, falls vorhanden. */
  letzteUebergabe?: Notiz;
  /** Offene Fäden (Notizen `art: 'offen'` ohne `erledigt_am`), neueste zuerst. */
  offeneFaeden: Notiz[];
  /**
   * Aktivität seit dem letzten Besuch, nach Quelle gruppiert. Referenzpunkt für
   * "letzter Besuch" ist die letzte Übergabenotiz (der einzige erzwungene,
   * verlässliche Zeitpunkt, siehe CONCEPT.md Abschnitt 5); ohne Übergabenotiz
   * wird ab dem Anlegen des Projekts gezählt.
   */
  aktivitaetSeitLetztemBesuch: Partial<Record<NotizQuelle, Notiz[]>>;
}

const TAG_MS = 24 * 60 * 60 * 1000;

function tageZwischen(spaeter: Date, frueher: Date): number {
  return Math.floor((spaeter.getTime() - frueher.getTime()) / TAG_MS);
}

function nachTsSortiert(notes: Notiz[]): Notiz[] {
  return [...notes].sort((a, b) => b.ts.localeCompare(a.ts));
}

function letzterKontaktTs(project: Projekt, notes: Notiz[]): string {
  const sorted = nachTsSortiert(notes);
  return sorted[0]?.ts ?? project.zuletzt_beruehrt;
}

/**
 * Auffälligkeit eines Projekts für den Hafen. Rot (`ueberfaellig`) wird nur,
 * wer sein eigenes Erwartungsintervall reißt oder dessen Wiedervorlage
 * verstrichen ist – `pausiert` und `wartet` werden nie rot außer über eine
 * verstrichene Wiedervorlage. `auffaellig` ist eine frühere Vorwarnstufe
 * (70–100 % des Intervalls), damit ein Projekt nicht unvermittelt "rot" wird.
 */
export function auffaelligkeit(project: Projekt, notes: Notiz[], jetzt: Date = new Date()): Auffaelligkeit {
  if (project.wiedervorlage) {
    const heuteIso = jetzt.toISOString().slice(0, 10);
    if (project.wiedervorlage < heuteIso) {
      return 'ueberfaellig';
    }
  }

  // abgeschlossen/eingemottet sind bewusst beendet, pausiert/wartet ruhen
  // bewusst: das Erwartungsintervall greift für sie nicht, nur die Wiedervorlage oben.
  if (
    project.status === 'abgeschlossen' ||
    project.status === 'eingemottet' ||
    project.status === 'pausiert' ||
    project.status === 'wartet'
  ) {
    return 'ruhig';
  }

  const tage = tageZwischen(jetzt, new Date(letzterKontaktTs(project, notes)));
  if (tage > project.erwartungsintervall_tage) {
    return 'ueberfaellig';
  }
  if (tage > project.erwartungsintervall_tage * 0.7) {
    return 'auffaellig';
  }
  return 'ruhig';
}

/**
 * Ob die Brief-Karte beim Öffnen der Projektseite gezeigt werden soll: das
 * Projekt hat länger als sein Erwartungsintervall geruht (CONCEPT.md Abschnitt 4).
 */
export function sollBriefZeigen(project: Projekt, notes: Notiz[], jetzt: Date = new Date()): boolean {
  const tage = tageZwischen(jetzt, new Date(letzterKontaktTs(project, notes)));
  return tage > project.erwartungsintervall_tage;
}

/** Berechnet den "Wo war ich"-Brief für ein Projekt aus dessen Notizen. */
export function computeBrief(project: Projekt, notes: Notiz[], jetzt: Date = new Date()): WoWarIchBrief {
  const eigene = notes.filter((n) => n.projekt_id === project.id);
  const sorted = nachTsSortiert(eigene);

  const letzteUebergabe = sorted.find((n) => n.art === 'uebergabe');
  const offeneFaeden = sorted.filter((n) => n.art === 'offen' && !n.erledigt_am);
  const letzterKontakt = sorted[0]?.ts ?? project.zuletzt_beruehrt;
  const tageSeitLetztemKontakt = tageZwischen(jetzt, new Date(letzterKontakt));

  const seitReferenz = letzteUebergabe?.ts ?? project.angelegt;
  const aktivitaetSeitLetztemBesuch: Partial<Record<NotizQuelle, Notiz[]>> = {};
  for (const n of sorted) {
    if (n.id === letzteUebergabe?.id) {
      continue;
    }
    if (n.ts > seitReferenz) {
      (aktivitaetSeitLetztemBesuch[n.quelle] ??= []).push(n);
    }
  }

  return { tageSeitLetztemKontakt, letzteUebergabe, offeneFaeden, aktivitaetSeitLetztemBesuch };
}
