import { describe, expect, it, vi } from 'vitest';
import { ablageLeeren, type Zwischenablage } from './zwischenablage';

function ablage(inhalt: string | Error, schreibfehler = false): Zwischenablage & { inhalt: string } {
  return {
    inhalt: inhalt instanceof Error ? '' : inhalt,
    async readText() {
      if (inhalt instanceof Error) throw inhalt;
      return this.inhalt;
    },
    async writeText(t: string) {
      if (schreibfehler) throw new Error('kein Schreibrecht');
      this.inhalt = t;
    },
  };
}

describe('ablageLeeren', () => {
  it('entfernt den eigenen Wert', async () => {
    const a = ablage('geheim');
    expect(await ablageLeeren('geheim', a)).toBe('geleert');
    expect(a.inhalt).toBe('');
  });

  it('lässt liegen, was jemand anderes kopiert hat', async () => {
    const a = ablage('eine Adresse');
    const schreiben = vi.spyOn(a, 'writeText');
    expect(await ablageLeeren('geheim', a)).toBe('fremd');
    expect(a.inhalt).toBe('eine Adresse');
    expect(schreiben).not.toHaveBeenCalled();
  });

  it('leert auch dann, wenn sich die Ablage nicht lesen lässt', async () => {
    // Ein Passwort, das stehen bleibt, wiegt schwerer als ein verlorener Kopiervorgang.
    const a = ablage(new Error('Leserecht fehlt'));
    expect(await ablageLeeren('geheim', a)).toBe('blind-geleert');
    expect(a.inhalt).toBe('');
  });

  it('meldet, wenn auch das Schreiben scheitert', async () => {
    const a = ablage('geheim', true);
    expect(await ablageLeeren('geheim', a)).toBe('fehlgeschlagen');
  });
});
