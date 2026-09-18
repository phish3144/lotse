// Ein Test für den Anlege-Dialog, weil es dafür keinen gab – und deshalb in 0.8.0 eine
// Fassung ausgeliefert wurde, in der beide Knöpfe aussahen wie tot.
//
// Der Fehler war ein Effekt, der eine `bind:this`-Referenz gelesen hat. Svelte hängt die
// beim Wechsel zum Befund aus, der Effekt lief dadurch erneut und setzte den Dialog im
// selben Frame zurück. Nichts davon fällt bei einer Typprüfung auf; man muss den Dialog
// bedienen. Genau das tut dieser Test – mit `happy-dom`, das ohnehin dabei ist.
import { flushSync, mount, unmount } from 'svelte';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import NeuesProjekt from './NeuesProjekt.svelte';

// Ohne Tauri läuft die Oberfläche gegen `mock.ts`; dort ist jede Eingabe ein Titel.
// Genau der Weg, den ein Mensch im Browser geht – und der Schrittwechsel ist derselbe.

let wurzel: HTMLElement;
let komponente: ReturnType<typeof mount> | undefined;

beforeEach(() => {
  wurzel = document.createElement('div');
  document.body.appendChild(wurzel);
});

afterEach(() => {
  if (komponente) unmount(komponente);
  komponente = undefined;
  wurzel.remove();
});

function oeffnen() {
  komponente = mount(NeuesProjekt, { target: wurzel, props: { offen: true } });
  flushSync();
}

const formular = (name: string) => wurzel.querySelector<HTMLFormElement>(`form[aria-label="${name}"]`);
const knopf = (text: string) =>
  [...wurzel.querySelectorAll('button')].find((b) => b.textContent?.trim().startsWith(text));

function tippen(wert: string) {
  const feld = wurzel.querySelector<HTMLInputElement>('input.gross');
  expect(feld, 'das eine Feld muss da sein').toBeTruthy();
  feld!.value = wert;
  feld!.dispatchEvent(new Event('input', { bubbles: true }));
  flushSync();
}

async function weiter() {
  knopf('Weiter')!.click();
  // Der Übergang läuft über Promises – und bei einem bloßen Namen legt derselbe Klick
  // gleich an, was noch einmal so viele Umläufe braucht. Lieber großzügig warten als
  // eine Zahl, die bei der nächsten Änderung nicht mehr reicht.
  for (let i = 0; i < 12; i++) await Promise.resolve();
  flushSync();
}

describe('Anlege-Dialog', () => {
  it('fragt zuerst nur eines und zeigt ein Feld', () => {
    oeffnen();
    expect(wurzel.querySelector('.dialog h2')?.textContent).toBe("Was gibt's?");
    expect(wurzel.querySelector('input.gross')).toBeTruthy();
    // Kein zweiter Weg daneben: das war der Grund für den Umbau.
    expect(knopf('Ohne Quelle')).toBeUndefined();
  });

  it('bleibt beim Befund stehen, statt zurückzuspringen', async () => {
    oeffnen();
    tippen('~/Projekte/gartenhaus');
    await weiter();

    expect(formular('Befund'), 'der Befund muss erscheinen').toBeTruthy();

    // Der Kern der Sache: noch einmal auslösen, was den Effekt erneut laufen ließ.
    // Beim Fehler in 0.8.0 stand hier wieder das Feld.
    flushSync();
    await Promise.resolve();
    flushSync();

    expect(formular('Befund'), 'der Befund muss stehen bleiben').toBeTruthy();
    expect(formular('Neues Vorhaben'), 'nicht zurück zum Feld').toBeFalsy();
  });

  it('übernimmt den getippten Text als Titelvorschlag', async () => {
    oeffnen();
    tippen('  ~/Projekte/hochbeet-suedseite  ');
    await weiter();
    const titel = formular('Befund')?.querySelector<HTMLInputElement>('input[type="text"]');
    expect(titel?.value).toBe('hochbeet suedseite');
  });

  /**
   * Ein getippter Name braucht keinen zweiten Bildschirm.
   *
   * Vorher kam dort ein Formular mit Titel, Vorlage und Kurs – und fragte damit nach
   * dem, was der Mensch gerade geschrieben hatte. Genau das war am Anlegen »komisch«.
   */
  it('legt einen getippten Namen in einem Schritt an', async () => {
    oeffnen();
    tippen('Dachboden ausbauen');
    await weiter();

    expect(formular('Befund'), 'kein zweiter Bildschirm für einen bloßen Namen').toBeFalsy();
    expect(formular('Neues Vorhaben'), 'und auch nicht zurück ins Feld').toBeFalsy();
  });

  /** Was Lotse gefüllt hat, ist als solches zu erkennen. */
  it('kennzeichnet geratene Felder und nimmt die Marke beim Tippen weg', async () => {
    oeffnen();
    tippen('~/Projekte/gartenhaus');
    await weiter();

    const marken = () => [...wurzel.querySelectorAll('.geraten')].map((e) => e.textContent?.trim());
    expect(marken().length, 'Titel, Vorlage und Kurs kommen aus dem Befund').toBeGreaterThan(0);

    const titel = formular('Befund')!.querySelector<HTMLInputElement>('input[type="text"]')!;
    const vorher = marken().length;
    titel.value = 'Selbst getippt';
    titel.dispatchEvent(new Event('input', { bubbles: true }));
    flushSync();
    expect(marken().length, 'was angefasst wurde, gilt nicht mehr als geraten').toBe(vorher - 1);
  });

  it('sagt etwas, wenn das Feld leer ist, und springt nicht weiter', async () => {
    oeffnen();
    await weiter();
    expect(wurzel.querySelector('.fehler')?.textContent).toContain('Schreib etwas hinein');
    expect(formular('Befund')).toBeFalsy();
  });

  it('führt von „Zurück" wieder zum Feld', async () => {
    oeffnen();
    tippen('~/Projekte/gartenhaus');
    await weiter();
    // Über die Beschriftung, nicht über die Position: im Befund stehen inzwischen auch
    // Knöpfe für Tags, und der erste ist längst nicht mehr »Zurück«.
    knopf('Zurück')!.click();
    flushSync();
    expect(wurzel.querySelector('.dialog h2')?.textContent).toBe("Was gibt's?");
    expect(formular('Befund')).toBeFalsy();
  });
});
