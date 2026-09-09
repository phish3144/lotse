/**
 * Die Zwischenablage wieder leeren – das Versprechen aus `THREAT_MODEL.md`, Abschnitt 2:
 * ein kopiertes Passwort soll dort nicht liegen bleiben.
 *
 * Vorher wird nachgesehen, ob überhaupt noch der eigene Wert darin steht. Wer inzwischen
 * etwas anderes kopiert hat, soll es behalten. Lässt sich die Zwischenablage nicht lesen
 * – je nach System und Berechtigung kommt das vor –, wird trotzdem geleert: ein Passwort,
 * das stehen bleibt, wiegt schwerer als ein verlorener Kopiervorgang.
 */
export interface Zwischenablage {
  readText(): Promise<string>;
  writeText(text: string): Promise<void>;
}

export type Ausgang =
  /** Der eigene Wert stand noch drin und wurde entfernt. */
  | 'geleert'
  /** Es stand etwas anderes drin; nicht angefasst. */
  | 'fremd'
  /** Nicht lesbar, deshalb ungeprüft geleert. */
  | 'blind-geleert'
  /** Auch das Schreiben ging nicht. */
  | 'fehlgeschlagen';

export async function ablageLeeren(wert: string, ablage: Zwischenablage): Promise<Ausgang> {
  let blind = false;
  try {
    if ((await ablage.readText()) !== wert) return 'fremd';
  } catch {
    blind = true;
  }
  try {
    await ablage.writeText('');
  } catch {
    return 'fehlgeschlagen';
  }
  return blind ? 'blind-geleert' : 'geleert';
}
