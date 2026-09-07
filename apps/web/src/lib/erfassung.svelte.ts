// Steuerung der globalen Schnellerfassung. Liegt außerhalb der Komponente, damit
// Kopfzeile, Projektseite und Tastenkürzel dasselbe Feld öffnen können.
import type { NotizArt } from './data/types';

class Erfassung {
  offen = $state(false);
  /** Vorbelegter Text, z. B. "@Projektname " beim Öffnen aus einem Projekt heraus. */
  vorbelegung = $state('');
  art = $state<NotizArt>('log');

  oeffnen(vorbelegung = '', art: NotizArt = 'log') {
    this.vorbelegung = vorbelegung;
    this.art = art;
    this.offen = true;
  }

  schliessen() {
    this.offen = false;
  }

  umschalten() {
    if (this.offen) {
      this.schliessen();
    } else {
      this.oeffnen();
    }
  }
}

export const erfassung = new Erfassung();
