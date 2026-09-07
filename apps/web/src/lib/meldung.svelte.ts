// Kurze Rückmeldungen nach einer Aktion. Ohne sie sieht man nach dem Speichern nur
// eine neu gezeichnete Liste und weiß nicht, ob etwas passiert ist.

export type MeldungsArt = 'gut' | 'fehler';

export interface Meldung {
  id: number;
  text: string;
  art: MeldungsArt;
}

const DAUER_MS = 3200;

class Meldungen {
  liste = $state<Meldung[]>([]);
  private naechsteId = 1;

  zeigen(text: string, art: MeldungsArt = 'gut') {
    const id = this.naechsteId++;
    this.liste = [...this.liste, { id, text, art }];
    setTimeout(() => this.schliessen(id), DAUER_MS);
  }

  /** Bequemer Weg für catch-Blöcke: nimmt alles entgegen, was geworfen wurde. */
  fehler(e: unknown) {
    this.zeigen(e instanceof Error ? e.message : String(e), 'fehler');
  }

  schliessen(id: number) {
    this.liste = this.liste.filter((m) => m.id !== id);
  }
}

export const meldungen = new Meldungen();
