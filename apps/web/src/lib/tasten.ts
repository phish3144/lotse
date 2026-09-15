// Tastenkürzel werden auf jedem System anders geschrieben. Die Anwendung horcht
// ohnehin auf beide Modifikatoren (metaKey || ctrlKey); hier geht es nur darum, das
// Richtige anzuzeigen. Ein ⌘ auf einem Linux-Rechner ist schlicht falsch.
//
// `navigator.platform` ist abgekündigt, `userAgentData` gibt es nicht überall –
// deshalb die Zeichenkette, die alle Browser liefern.
const istApple = /Mac|iPhone|iPad|iPod/.test(
  typeof navigator === 'undefined' ? '' : navigator.userAgent,
);

/** »⌘« auf Apple-Geräten, sonst »Strg«. */
export const MOD = istApple ? '⌘' : 'Strg';

/** Anzeige eines Kürzels: `kuerzel('K')` → `⌘K` oder `Strg+K`. */
export function kuerzel(taste: string): string {
  return istApple ? `${MOD}${taste}` : `${MOD}+${taste}`;
}
