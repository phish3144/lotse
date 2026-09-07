// Steuerung des Projekt-Sprungs (Strg/Cmd+P), analog zur Schnellerfassung.
class Sprung {
  offen = $state(false);

  oeffnen() {
    this.offen = true;
  }

  schliessen() {
    this.offen = false;
  }

  umschalten() {
    this.offen = !this.offen;
  }
}

export const sprung = new Sprung();
