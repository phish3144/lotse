import { describe, expect, it } from "vitest";
import {
  datumText,
  ersteZeile,
  hervorheben,
  nachTagen,
  tagesMarke,
  uhrzeitText,
  zeileMitTreffer,
} from './format';

describe("datumText", () => {
  it("zeigt ein reines Datum als denselben Kalendertag", () => {
    // `new Date('2026-09-10')` wäre Mitternacht UTC; westlich davon stünde der 9. da.
    expect(datumText("2026-09-10")).toBe("10. September 2026");
  });

  it("versteht weiterhin vollständige Zeitstempel", () => {
    const iso = new Date(2026, 8, 10, 12, 0).toISOString();
    expect(datumText(iso)).toBe("10. September 2026");
  });
});

describe("tagesMarke", () => {
  // Ortszeit, nicht UTC: ein Eintrag um 23:30 gehört zu dem Tag, an dem er geschrieben
  // wurde. Die Tests bauen die Zeitpunkte deshalb als Ortszeit, nicht als ISO-Z-String.
  const ortszeit = (j: number, m: number, t: number, h = 12, min = 0) =>
    new Date(j, m - 1, t, h, min).toISOString();
  const jetzt = new Date(2026, 8, 11, 10, 0); // 11. September 2026, Ortszeit

  it("nennt heute und gestern beim Namen", () => {
    expect(tagesMarke(ortszeit(2026, 9, 11, 8), jetzt)).toBe("Heute");
    expect(tagesMarke(ortszeit(2026, 9, 10, 23, 30), jetzt)).toBe("Gestern");
  });

  it("zählt den späten Abend zum richtigen Tag", () => {
    // 23:30 Ortszeit am 10. ist in UTC+2 schon der 11. UTC. Trotzdem: gestern.
    expect(tagesMarke(ortszeit(2026, 9, 10, 23, 59), jetzt)).toBe("Gestern");
    expect(tagesMarke(ortszeit(2026, 9, 11, 0, 5), jetzt)).toBe("Heute");
  });

  it("lässt das Jahr weg, solange es dasselbe ist", () => {
    expect(tagesMarke(ortszeit(2026, 3, 2), jetzt)).toBe("2. März");
    expect(tagesMarke(ortszeit(2025, 12, 24), jetzt)).toBe("24. Dezember 2025");
  });
});

describe("nachTagen", () => {
  const ortszeit = (j: number, m: number, t: number, h = 12) =>
    new Date(j, m - 1, t, h).toISOString();
  const jetzt = new Date(2026, 8, 11, 10, 0);

  it("fasst aufeinanderfolgende Einträge desselben Tages zusammen", () => {
    const gruppen = nachTagen(
      [
        { ts: ortszeit(2026, 9, 11, 14), text: "a" },
        { ts: ortszeit(2026, 9, 11, 9), text: "b" },
        { ts: ortszeit(2026, 9, 10, 17), text: "c" },
        { ts: ortszeit(2026, 9, 8, 16), text: "d" },
      ],
      jetzt,
    );
    expect(gruppen.map((g) => g.marke)).toEqual([
      "Heute",
      "Gestern",
      "8. September",
    ]);
    expect(gruppen[0].eintraege.map((e) => e.text)).toEqual(["a", "b"]);
    expect(gruppen[2].eintraege).toHaveLength(1);
  });

  it("sortiert nicht um – eine bewusste Reihenfolge bleibt stehen", () => {
    const gruppen = nachTagen(
      [
        { ts: ortszeit(2026, 9, 11) },
        { ts: ortszeit(2026, 9, 10) },
        { ts: ortszeit(2026, 9, 11) },
      ],
      jetzt,
    );
    // Drei Gruppen, nicht zwei: der dritte Eintrag steht nicht neben dem ersten.
    expect(gruppen.map((g) => g.marke)).toEqual(["Heute", "Gestern", "Heute"]);
  });

  it("gibt für nichts auch nichts zurück", () => {
    expect(nachTagen([], jetzt)).toEqual([]);
  });
});

describe("uhrzeitText", () => {
  it("zeigt zweistellige Stunden und Minuten", () => {
    expect(uhrzeitText(new Date(2026, 8, 11, 9, 5).toISOString())).toBe(
      "09:05",
    );
  });
});

describe('hervorheben', () => {
  it('markiert jeden Treffer, auch mehrfach', () => {
    expect(hervorheben('Beton und Beton', 'beton')).toEqual([
      { t: 'Beton', treffer: true },
      { t: ' und ', treffer: false },
      { t: 'Beton', treffer: true },
    ]);
  });

  it('lässt den Text ganz, wenn nichts passt', () => {
    expect(hervorheben('Schalung geprüft', 'beton')).toEqual([{ t: 'Schalung geprüft', treffer: false }]);
    expect(hervorheben('Schalung', '  ')).toEqual([{ t: 'Schalung', treffer: false }]);
  });

  it('behandelt Sonderzeichen als Zeichen, nicht als Muster', () => {
    // Mit einem regulären Ausdruck wäre „.“ jedes Zeichen und „(“ ein Syntaxfehler.
    expect(hervorheben('a.b', '.')).toEqual([
      { t: 'a', treffer: false },
      { t: '.', treffer: true },
      { t: 'b', treffer: false },
    ]);
    expect(() => hervorheben('f(x)', '(')).not.toThrow();
    expect(hervorheben('abc', '.')).toEqual([{ t: 'abc', treffer: false }]);
  });

  it('findet unabhängig von Groß- und Kleinschreibung', () => {
    expect(hervorheben('Bewehrung', 'BEWEH')).toEqual([
      { t: 'Beweh', treffer: true },
      { t: 'rung', treffer: false },
    ]);
  });

  it('schneidet nicht falsch, wenn Kleinschreiben die Länge ändert', () => {
    // „İ“ (türkisches I mit Punkt) wird zu zwei Zeichen – dann stimmen die Positionen
    // im Original nicht mehr, und lieber gar nicht hervorheben als daneben.
    const text = 'İstanbul';
    expect(hervorheben(text, 'stan')).toEqual([{ t: text, treffer: false }]);
  });

  it('markiert einen Treffer am Anfang ohne leeres Vorstück', () => {
    expect(hervorheben('Betonmischer', 'beton')).toEqual([
      { t: 'Beton', treffer: true },
      { t: 'mischer', treffer: false },
    ]);
  });
});

describe('ersteZeile', () => {
  it('nimmt nur die erste Zeile', () => {
    expect(ersteZeile('Kontext: dies\nEntschieden: das')).toBe('Kontext: dies');
    expect(ersteZeile('einzeilig')).toBe('einzeilig');
  });
});

describe('zeileMitTreffer', () => {
  const eintrag = [
    'Kontext: Zahlungen bisher nur manuell abgeglichen.',
    'Entschieden: Stripe für Kartenzahlungen anbinden.',
    'Verworfen weil: eigene Abwicklung wäre Overkill.',
  ].join('\n');

  it('nimmt die Zeile mit dem Begriff, nicht die erste', () => {
    expect(zeileMitTreffer(eintrag, 'stripe')).toBe('Entschieden: Stripe für Kartenzahlungen anbinden.');
    expect(zeileMitTreffer(eintrag, 'overkill')).toBe('Verworfen weil: eigene Abwicklung wäre Overkill.');
  });

  it('fällt auf die erste Zeile zurück', () => {
    expect(zeileMitTreffer(eintrag, 'gibtsnicht')).toBe('Kontext: Zahlungen bisher nur manuell abgeglichen.');
    expect(zeileMitTreffer(eintrag, '')).toBe('Kontext: Zahlungen bisher nur manuell abgeglichen.');
  });

  it('nimmt die erste passende Zeile, wenn mehrere passen', () => {
    expect(zeileMitTreffer('a zahl\nb zahl', 'zahl')).toBe('a zahl');
  });
});
