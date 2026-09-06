import { describe, expect, it } from 'vitest';
import { auffaelligkeit, computeBrief, sollBriefZeigen } from './brief';
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

  it('ist auffaellig zwischen 70% und 100% des Intervalls', () => {
    const p = projekt({ erwartungsintervall_tage: 10 });
    const notes = [notiz({ ts: tageVorJetzt(8) })];
    expect(auffaelligkeit(p, notes, JETZT)).toBe('auffaellig');
  });

  it('ist ueberfaellig, wenn das eigene Erwartungsintervall gerissen wird', () => {
    const p = projekt({ status: 'aktiv', erwartungsintervall_tage: 14 });
    const notes = [notiz({ ts: tageVorJetzt(20) })];
    expect(auffaelligkeit(p, notes, JETZT)).toBe('ueberfaellig');
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
    const p = projekt({ erwartungsintervall_tage: 14, zuletzt_beruehrt: tageVorJetzt(30) });
    expect(auffaelligkeit(p, [], JETZT)).toBe('ueberfaellig');
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
});

describe('computeBrief', () => {
  it('liefert Tage seit letztem Kontakt, letzte Übergabe, offene Fäden und Aktivität nach Quelle', () => {
    const p = projekt({ erwartungsintervall_tage: 14, angelegt: tageVorJetzt(365) });
    const notes: Notiz[] = [
      notiz({ id: 'a', ts: tageVorJetzt(40), art: 'uebergabe', quelle: 'mensch', text: 'Übergabe: Stand X.' }),
      notiz({ id: 'b', ts: tageVorJetzt(35), art: 'log', quelle: 'git', text: '2 Commits.' }),
      notiz({ id: 'c', ts: tageVorJetzt(30), art: 'log', quelle: 'datei', text: '5 Dateien geändert.' }),
      notiz({ id: 'd', ts: tageVorJetzt(20), art: 'offen', quelle: 'mensch', text: 'Noch zu tun.' }),
      notiz({ id: 'e', ts: tageVorJetzt(10), art: 'offen', quelle: 'mensch', text: 'Erledigter Faden.', erledigt_am: tageVorJetzt(5) }),
    ];

    const brief = computeBrief(p, notes, JETZT);

    expect(brief.tageSeitLetztemKontakt).toBe(10);
    expect(brief.letzteUebergabe?.id).toBe('a');
    expect(brief.offeneFaeden.map((n) => n.id)).toEqual(['d']);
    expect(brief.aktivitaetSeitLetztemBesuch.git?.map((n) => n.id)).toEqual(['b']);
    expect(brief.aktivitaetSeitLetztemBesuch.datei?.map((n) => n.id)).toEqual(['c']);
    // Die Übergabenotiz selbst zählt nicht als "Aktivität seit dem letzten Besuch".
    expect(brief.aktivitaetSeitLetztemBesuch.mensch?.some((n) => n.id === 'a')).toBeFalsy();
  });

  it('zählt Aktivität ab Anlegen, wenn es keine Übergabenotiz gibt', () => {
    const p = projekt({ angelegt: tageVorJetzt(50) });
    const notes: Notiz[] = [notiz({ id: 'x', ts: tageVorJetzt(10), quelle: 'cli', art: 'log' })];

    const brief = computeBrief(p, notes, JETZT);

    expect(brief.letzteUebergabe).toBeUndefined();
    expect(brief.aktivitaetSeitLetztemBesuch.cli?.map((n) => n.id)).toEqual(['x']);
  });
});
