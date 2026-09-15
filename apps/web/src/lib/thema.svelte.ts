// Darstellung. Der Standard bleibt der Systemstandard – Lotse mischt sich nicht ein,
// solange niemand es verlangt. Wer sich entscheidet, bekommt seine Wahl bei jedem Start
// zurück.
//
// Die Wahl liegt in localStorage und nicht in der Datenbank: Der Sperrbildschirm
// erscheint, bevor irgendetwas entschlüsselt ist, und er soll schon richtig aussehen.
// Es ist eine Anzeige-Einstellung dieses Geräts, kein Inhalt – sie wird nicht abgeglichen.

export type Thema = 'system' | 'hell' | 'dunkel';

export const THEMEN: { id: Thema; name: string }[] = [
  { id: 'system', name: 'Wie das System' },
  { id: 'hell', name: 'Hell' },
  { id: 'dunkel', name: 'Dunkel' },
];

const SCHLUESSEL = 'lotse.thema';

function lesen(): Thema {
  try {
    const wert = localStorage.getItem(SCHLUESSEL);
    if (wert === 'hell' || wert === 'dunkel' || wert === 'system') return wert;
  } catch {
    // Privates Fenster, gesperrter Speicher: dann eben der Systemstandard.
  }
  return 'system';
}

/** Setzt die Marke am Wurzelelement; »system« lässt sie weg und überlässt es dem CSS. */
export function anwenden(t: Thema): void {
  const wurzel = document.documentElement;
  if (t === 'system') wurzel.removeAttribute('data-thema');
  else wurzel.setAttribute('data-thema', t);
}

export const thema = $state<{ wert: Thema }>({ wert: lesen() });

export function setzen(t: Thema): void {
  thema.wert = t;
  try {
    localStorage.setItem(SCHLUESSEL, t);
  } catch {
    // Nicht speicherbar: die Wahl gilt dann nur für diese Sitzung.
  }
  anwenden(t);
}
