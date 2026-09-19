import type { Notiz, NotizQuelle, Projekt } from './data/types';

// Reine, seiteneffektfreie Funktionen für den "Wo war ich"-Brief und die
// Auffälligkeit eines Projekts. Siehe docs/CONCEPT.md Abschnitt 4 und 5.
// Bewusst getrennt vom Provider, damit sich das Verhalten ohne Mock- oder
// WASM-Anbindung testen lässt.
//
// **Dies ist eine Übersetzung von `crates/lotse-core/src/brief.rs`, keine zweite
// Meinung.** Bis 0.11 war es eine: hier war rot ab dem einfachen Intervall und die
// Vorwarnung ab 70 %, im Kern (und in der Wiki-Beschreibung) rot erst ab dem doppelten;
// hier zählte »seit deinem letzten Besuch« ab der letzten Übergabe und über alle
// Quellen, im Kern ab dem letzten eigenen Eintrag und ohne die eigenen. Die Farbe im
// Hafen widersprach also dem, was `lotse hafen` ausgab. Wer hier etwas ändert, ändert es
// dort mit.

export type Auffaelligkeit = 'ruhig' | 'auffaellig' | 'ueberfaellig';

export interface WoWarIchBrief {
  /** Tage seit dem letzten *eigenen* Kontakt (`mensch` oder `cli`). */
  tageSeitLetztemKontakt: number;
  /** Die zuletzt geschriebene Übergabenotiz, im Wortlaut, falls vorhanden. */
  letzteUebergabe?: Notiz;
  /** Die letzte Notiz überhaupt – schlechter als eine Übergabe, besser als nichts. */
  letzteNotiz?: Notiz;
  /** Offene Fäden (Notizen `art: 'offen'` ohne `erledigt_am`), neueste zuerst. */
  offeneFaeden: Notiz[];
  /**
   * Aktivität seit dem letzten Besuch, nach Quelle gruppiert. Referenzpunkt ist die
   * letzte eigene Notiz (`mensch` oder `cli`), und gezählt wird nur, was *nicht* von
   * dir kam: eigene Einträge sind kein »das ist passiert, während ich weg war«.
   */
  aktivitaetSeitLetztemBesuch: Partial<Record<NotizQuelle, Notiz[]>>;
}

const TAG_MS = 24 * 60 * 60 * 1000;

/** Quellen, die für „das habe ich selbst geschrieben“ stehen. */
function vonDir(n: Notiz): boolean {
  return n.quelle === 'mensch' || n.quelle === 'cli';
}

function nachTsSortiert(notes: Notiz[]): Notiz[] {
  return [...notes].sort((a, b) => b.ts.localeCompare(a.ts));
}

/**
 * Tage seit einem Zeitpunkt – `brief::tage_seit`. Ohne Notiz gilt `zuletzt_beruehrt`,
 * in keinem Fall aber etwas vor dem Anlegen: ein Vorhaben kann nicht länger still sein
 * als es existiert.
 */
function tageSeit(project: Projekt, notizTs: string | undefined, jetzt: Date): number {
  const letzter = Math.max(
    new Date(notizTs ?? project.zuletzt_beruehrt).getTime(),
    new Date(project.angelegt).getTime(),
  );
  return Math.floor(Math.max(jetzt.getTime() - letzter, 0) / TAG_MS);
}

/**
 * Auffälligkeit eines Projekts für den Hafen – `brief::auffaelligkeit`. Nur `aktiv`
 * kann auffallen; sonst wäre jedes abgeschlossene Vorhaben nach einem Jahr rot und die
 * Farbe bedeutete nichts mehr. `auffaellig` ab dem einfachen, `ueberfaellig` ab dem
 * doppelten Erwartungsintervall. Eine verstrichene Wiedervorlage setzt beides außer
 * Kraft, in jedem Status.
 */
export function auffaelligkeit(project: Projekt, notes: Notiz[], jetzt: Date = new Date()): Auffaelligkeit {
  if (project.wiedervorlage) {
    const heuteIso = jetzt.toISOString().slice(0, 10);
    if (project.wiedervorlage < heuteIso) {
      return 'ueberfaellig';
    }
  }

  if (project.status !== 'aktiv') {
    return 'ruhig';
  }

  const tage = tageSeit(project, nachTsSortiert(notes)[0]?.ts, jetzt);
  const intervall = project.erwartungsintervall_tage;
  if (tage > intervall * 2) {
    return 'ueberfaellig';
  }
  if (tage > intervall) {
    return 'auffaellig';
  }
  return 'ruhig';
}

/**
 * Bezugspunkt für »seit meinem letzten Besuch« – `brief::brief`: der letzte *eigene*
 * Eintrag (`mensch` oder `cli`), sonst das Anlegen des Vorhabens. Nicht die letzte Notiz
 * überhaupt: in einem übernommenen Vorhaben schreibt nur der Beobachter, und der Punkt
 * rückte dann täglich nach – der Brief wurde nie fällig und blieb immer leer.
 */
function eigenerBezug(project: Projekt, notes: Notiz[]): string {
  return nachTsSortiert(notes).find(vonDir)?.ts ?? project.angelegt;
}

/**
 * Ob die Brief-Karte beim Öffnen der Projektseite gezeigt werden soll – `brief::faellig`:
 * das Vorhaben hat länger geruht als sein Erwartungsintervall (CONCEPT.md Abschnitt 4).
 * Gezählt ab dem letzten *eigenen* Eintrag, nicht ab dem letzten überhaupt.
 */
export function sollBriefZeigen(project: Projekt, notes: Notiz[], jetzt: Date = new Date()): boolean {
  const eigene = notes.filter((n) => n.projekt_id === project.id);
  return tageSeit(project, eigenerBezug(project, eigene), jetzt) > project.erwartungsintervall_tage;
}

/**
 * Ob im Brief etwas steht, das aus deiner Abwesenheit stammt: eine Übergabe, offene
 * Fäden, oder Aktivität, seit du zuletzt selbst geschrieben hast. `sollBriefZeigen`
 * allein reichte nicht – an einem Vorhaben, in dem der Beobachter gerade mitschreibt,
 * war die Karte unsichtbar, und mit ihr der einzige Knopf, der die KI von Hand auslöst.
 *
 * Bewusst **nicht** »irgendeine Notiz«: ein Brief, der jedes Mal kommt, wird weggeklickt
 * und ist dann nichts wert (docs/wiki/Wo-war-ich-Brief.md). Die Zeile »Letzter Kontakt«
 * allein ist kein Inhalt.
 */
export function briefHatInhalt(brief: WoWarIchBrief): boolean {
  return (
    brief.letzteUebergabe !== undefined ||
    brief.offeneFaeden.length > 0 ||
    Object.keys(brief.aktivitaetSeitLetztemBesuch).length > 0
  );
}

/** Berechnet den "Wo war ich"-Brief für ein Projekt aus dessen Notizen. */
export function computeBrief(project: Projekt, notes: Notiz[], jetzt: Date = new Date()): WoWarIchBrief {
  const eigene = notes.filter((n) => n.projekt_id === project.id);
  const sorted = nachTsSortiert(eigene);

  const letzteUebergabe = sorted.find((n) => n.art === 'uebergabe');
  const letzteNotiz = sorted[0];
  const offeneFaeden = sorted.filter((n) => n.art === 'offen' && !n.erledigt_am);
  const bezug = eigenerBezug(project, eigene);
  const tageSeitLetztemKontakt = tageSeit(project, bezug, jetzt);

  const aktivitaetSeitLetztemBesuch: Partial<Record<NotizQuelle, Notiz[]>> = {};
  for (const n of sorted) {
    if (n.ts > bezug && !vonDir(n)) {
      (aktivitaetSeitLetztemBesuch[n.quelle] ??= []).push(n);
    }
  }

  return { tageSeitLetztemKontakt, letzteUebergabe, letzteNotiz, offeneFaeden, aktivitaetSeitLetztemBesuch };
}
