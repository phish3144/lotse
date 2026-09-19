import { describe, expect, it } from 'vitest';
import { auffaelligkeit, briefHatInhalt, computeBrief, sollBriefZeigen } from './brief';
import type { Notiz, Projekt } from './data/types';

const JETZT = new Date('2026-09-06T12:00:00.000Z');

function tageVorJetzt(tage: number): string {
  return new Date(JETZT.getTime() - tage * 24 * 60 * 60 * 1000).toISOString();
}

function projekt(overrides: Partial<Projekt> = {}): Projekt {
  return {
    id: 'p1',
    titel: 'Testprojekt',
    kurs: 'Kurs',
    status: 'aktiv',
    erwartungsintervall_tage: 14,
    tags: [],
    vorlage: 'software',
    angelegt: tageVorJetzt(365),
    zuletzt_beruehrt: tageVorJetzt(1),
    ...overrides,
  };
}

function notiz(overrides: Partial<Notiz> = {}): Notiz {
  return {
    id: 'n1',
    projekt_id: 'p1',
    ts: tageVorJetzt(1),
    quelle: 'mensch',
    art: 'log',
    text: 'Text',
    ...overrides,
  };
}

describe('auffaelligkeit', () => {
  it('ist ruhig innerhalb des Erwartungsintervalls', () => {
    const p = projekt({ erwartungsintervall_tage: 14 });
    const notes = [notiz({ ts: tageVorJetzt(3) })];
    expect(auffaelligkeit(p, notes, JETZT)).toBe('ruhig');
  });

  it('ist auffaellig, sobald das Erwartungsintervall gerissen ist', () => {
    const p = projekt({ erwartungsintervall_tage: 10 });
    const notes = [notiz({ ts: tageVorJetzt(15) })];
    expect(auffaelligkeit(p, notes, JETZT)).toBe('auffaellig');
  });

  it('ist ueberfaellig erst ab dem doppelten Intervall – wie der Kern und die Doku', () => {
    // Dieselbe Schwelle wie `brief::auffaelligkeit`. Vorher war die Oberfläche hier schon
    // ab dem einfachen Intervall rot; der Hafen widersprach damit `lotse hafen`.
    const p = projekt({ status: 'aktiv', erwartungsintervall_tage: 14 });
    expect(auffaelligkeit(p, [notiz({ ts: tageVorJetzt(20) })], JETZT)).toBe('auffaellig');
    expect(auffaelligkeit(p, [notiz({ ts: tageVorJetzt(30) })], JETZT)).toBe('ueberfaellig');
  });

  it('zählt die Farbe ab der letzten Notiz, welcher Quelle auch immer', () => {
    // Die Farbe fragt »ist hier etwas passiert?«, nicht »warst du selbst da?«.
    const p = projekt({ erwartungsintervall_tage: 14 });
    const notes = [notiz({ id: 'g', ts: tageVorJetzt(2), quelle: 'git', text: 'Commit.' })];
    expect(auffaelligkeit(p, notes, JETZT)).toBe('ruhig');
  });

  it('wird nie älter als das Anlegedatum', () => {
    const p = projekt({ erwartungsintervall_tage: 14, angelegt: tageVorJetzt(3), zuletzt_beruehrt: tageVorJetzt(900) });
    expect(auffaelligkeit(p, [], JETZT)).toBe('ruhig');
  });

  it('pausiert wird nicht rot, nur weil das Erwartungsintervall verstrichen ist', () => {
    const p = projekt({ status: 'pausiert', erwartungsintervall_tage: 14 });
    const notes = [notiz({ ts: tageVorJetzt(200) })];
    expect(auffaelligkeit(p, notes, JETZT)).toBe('ruhig');
  });

  it('wartet wird nicht rot ohne verstrichene Wiedervorlage', () => {
    const p = projekt({ status: 'wartet', erwartungsintervall_tage: 14, wiedervorlage: '2026-12-01' });
    const notes = [notiz({ ts: tageVorJetzt(200) })];
    expect(auffaelligkeit(p, notes, JETZT)).toBe('ruhig');
  });

  it('pausiert wird rot, sobald die Wiedervorlage verstrichen ist', () => {
    const p = projekt({ status: 'pausiert', wiedervorlage: '2026-08-01' });
    const notes = [notiz({ ts: tageVorJetzt(5) })];
    expect(auffaelligkeit(p, notes, JETZT)).toBe('ueberfaellig');
  });

  it('abgeschlossen bleibt immer ruhig', () => {
    const p = projekt({ status: 'abgeschlossen', erwartungsintervall_tage: 1 });
    const notes = [notiz({ ts: tageVorJetzt(900) })];
    expect(auffaelligkeit(p, notes, JETZT)).toBe('ruhig');
  });

  it('fällt ohne Notizen auf zuletzt_beruehrt zurück', () => {
    const p = projekt({ erwartungsintervall_tage: 14, zuletzt_beruehrt: tageVorJetzt(20) });
    expect(auffaelligkeit(p, [], JETZT)).toBe('auffaellig');
    const laenger = projekt({ erwartungsintervall_tage: 14, zuletzt_beruehrt: tageVorJetzt(60) });
    expect(auffaelligkeit(laenger, [], JETZT)).toBe('ueberfaellig');
  });
});

describe('sollBriefZeigen', () => {
  it('ist false innerhalb des Intervalls', () => {
    const p = projekt({ erwartungsintervall_tage: 14 });
    const notes = [notiz({ ts: tageVorJetzt(3) })];
    expect(sollBriefZeigen(p, notes, JETZT)).toBe(false);
  });

  it('ist true, wenn länger als das Intervall geruht wurde', () => {
    const p = projekt({ erwartungsintervall_tage: 14 });
    const notes = [notiz({ ts: tageVorJetzt(30) })];
    expect(sollBriefZeigen(p, notes, JETZT)).toBe(true);
  });

  it('lässt sich nicht von Beobachter-Notizen zurücksetzen', () => {
    // Der Fall, an dem der Brief hing: in einem übernommenen Vorhaben schreibt nur der
    // Beobachter. Zählte man ab der letzten Notiz überhaupt, wurde der Brief nie fällig.
    const p = projekt({ erwartungsintervall_tage: 14, angelegt: tageVorJetzt(90) });
    const notes = [
      notiz({ id: 'i', ts: tageVorJetzt(90), quelle: 'import', art: 'offen', text: 'Kurs festlegen.' }),
      notiz({ id: 'g', ts: tageVorJetzt(1), quelle: 'git', text: 'Zwei Commits.' }),
    ];
    expect(sollBriefZeigen(p, notes, JETZT)).toBe(true);
  });

  it('zählt ab dem letzten eigenen Eintrag', () => {
    const p = projekt({ erwartungsintervall_tage: 14, angelegt: tageVorJetzt(90) });
    const notes = [
      notiz({ id: 'm', ts: tageVorJetzt(2), quelle: 'mensch', text: 'Heute dran gewesen.' }),
      notiz({ id: 'g', ts: tageVorJetzt(1), quelle: 'git', text: 'Zwei Commits.' }),
    ];
    expect(sollBriefZeigen(p, notes, JETZT)).toBe(false);
  });
});

describe('briefHatInhalt', () => {
  it('ist false, wenn nur die Zeile mit dem letzten Kontakt übrig bliebe', () => {
    const p = projekt();
    expect(briefHatInhalt(computeBrief(p, [], JETZT))).toBe(false);
  });

  it('ist true bei einem offenen Faden, auch innerhalb des Intervalls', () => {
    const p = projekt({ erwartungsintervall_tage: 14 });
    const notes = [notiz({ id: 'o', ts: tageVorJetzt(1), art: 'offen', quelle: 'mensch', text: 'Noch zu tun.' })];
    // Der Fall, an dem es hing: die Karte wäre unsichtbar gewesen, obwohl etwas drinsteht.
    expect(sollBriefZeigen(p, notes, JETZT)).toBe(false);
    expect(briefHatInhalt(computeBrief(p, notes, JETZT))).toBe(true);
  });

  it('ist true bei Aktivität seit dem letzten Besuch, auch ohne eigene Notiz', () => {
    const p = projekt({ erwartungsintervall_tage: 14, angelegt: tageVorJetzt(30) });
    const notes = [notiz({ id: 'g', ts: tageVorJetzt(2), art: 'log', quelle: 'git', text: 'Zwei Commits.' })];
    expect(briefHatInhalt(computeBrief(p, notes, JETZT))).toBe(true);
  });

  it('ist false, wenn seit dem eigenen Eintrag nichts geschah', () => {
    const p = projekt({ erwartungsintervall_tage: 14, angelegt: tageVorJetzt(30) });
    const notes = [notiz({ id: 'm', ts: tageVorJetzt(1), quelle: 'mensch', text: 'Nur ich.' })];
    expect(briefHatInhalt(computeBrief(p, notes, JETZT))).toBe(false);
  });

  it('ist false, wenn ein erledigter Faden alles ist, was es gab', () => {
    const p = projekt({ erwartungsintervall_tage: 14, angelegt: tageVorJetzt(30) });
    const notes = [
      notiz({
        id: 'e',
        ts: tageVorJetzt(40),
        art: 'offen',
        quelle: 'mensch',
        text: 'Erledigt.',
        erledigt_am: tageVorJetzt(35),
      }),
    ];
    expect(briefHatInhalt(computeBrief(p, notes, JETZT))).toBe(false);
  });
});

describe('computeBrief', () => {
  it('liefert Tage seit letztem Kontakt, letzte Übergabe, offene Fäden und Aktivität nach Quelle', () => {
    const p = projekt({ erwartungsintervall_tage: 14, angelegt: tageVorJetzt(365) });
    const notes: Notiz[] = [
      notiz({ id: 'd', ts: tageVorJetzt(45), art: 'offen', quelle: 'mensch', text: 'Noch zu tun.' }),
      notiz({ id: 'a', ts: tageVorJetzt(40), art: 'uebergabe', quelle: 'mensch', text: 'Übergabe: Stand X.' }),
      notiz({ id: 'b', ts: tageVorJetzt(35), art: 'log', quelle: 'git', text: '2 Commits.' }),
      notiz({ id: 'c', ts: tageVorJetzt(30), art: 'log', quelle: 'datei', text: '5 Dateien geändert.' }),
    ];

    const brief = computeBrief(p, notes, JETZT);

    // Gezählt ab der letzten eigenen Notiz – der Übergabe vor 40 Tagen. Die Dateiänderung
    // vor 30 Tagen setzt das nicht zurück: dabei warst du nicht.
    expect(brief.tageSeitLetztemKontakt).toBe(40);
    expect(brief.letzteUebergabe?.id).toBe('a');
    expect(brief.letzteNotiz?.id).toBe('c');
    expect(brief.offeneFaeden.map((n) => n.id)).toEqual(['d']);
    expect(brief.aktivitaetSeitLetztemBesuch.git?.map((n) => n.id)).toEqual(['b']);
    expect(brief.aktivitaetSeitLetztemBesuch.datei?.map((n) => n.id)).toEqual(['c']);
    // Eigene Einträge sind kein »das ist passiert, während ich weg war«.
    expect(brief.aktivitaetSeitLetztemBesuch.mensch).toBeUndefined();
  });

  it('nennt die letzte Notiz auch dann, wenn es keine Übergabe gibt', () => {
    const p = projekt({ angelegt: tageVorJetzt(50) });
    const notes: Notiz[] = [notiz({ id: 'z', ts: tageVorJetzt(4), quelle: 'git', text: 'Ein Commit.' })];

    const brief = computeBrief(p, notes, JETZT);

    expect(brief.letzteUebergabe).toBeUndefined();
    expect(brief.letzteNotiz?.id).toBe('z');
  });

  it('zählt Aktivität ab dem Anlegen, wenn es keinen eigenen Eintrag gibt', () => {
    // Genau der Zustand eines übernommenen Vorhabens: `import` beim Anlegen, danach nur
    // der Beobachter. Vorher blieb die Liste hier leer – und mit ihr der halbe Brief.
    const p = projekt({ angelegt: tageVorJetzt(50) });
    const notes: Notiz[] = [
      notiz({ id: 'i', ts: tageVorJetzt(50), quelle: 'import', art: 'offen', text: 'Kurs festlegen.' }),
      notiz({ id: 'x', ts: tageVorJetzt(10), quelle: 'git', art: 'log', text: 'Ein Commit.' }),
    ];

    const brief = computeBrief(p, notes, JETZT);

    expect(brief.letzteUebergabe).toBeUndefined();
    expect(brief.tageSeitLetztemKontakt).toBe(50);
    expect(brief.aktivitaetSeitLetztemBesuch.git?.map((n) => n.id)).toEqual(['x']);
  });

  it('nimmt eine eigene Notiz als Bezugspunkt, sobald es eine gibt', () => {
    const p = projekt({ angelegt: tageVorJetzt(50) });
    const notes: Notiz[] = [
      notiz({ id: 'alt', ts: tageVorJetzt(40), quelle: 'git', art: 'log', text: 'Alt.' }),
      notiz({ id: 'ich', ts: tageVorJetzt(9), quelle: 'cli', art: 'log', text: 'Von Hand notiert.' }),
      notiz({ id: 'neu', ts: tageVorJetzt(2), quelle: 'git', art: 'log', text: 'Neu.' }),
    ];

    const brief = computeBrief(p, notes, JETZT);

    expect(brief.tageSeitLetztemKontakt).toBe(9);
    expect(brief.aktivitaetSeitLetztemBesuch.git?.map((n) => n.id)).toEqual(['neu']);
  });
});
