// Einfacher reaktiver Zähler: nach jeder Mutation über den Provider hochgezählt,
// damit Bildschirme, die Daten anzeigen, per $effect neu laden können, ohne dass
// der Provider selbst Subscriptions anbieten muss (das übernimmt später der
// WASM-Kern über echte Änderungsbenachrichtigungen).
class DatenVersion {
  wert = $state(0);
  bump() {
    this.wert += 1;
  }
}

export const datenVersion = new DatenVersion();
