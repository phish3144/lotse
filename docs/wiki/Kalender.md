# Kalender

Lotse liest Kalender. Es schreibt keine hinein und will kein Kalender werden
([[Nicht-Ziele]]). Was es beantworten will, ist die zweite Hälfte von »wo stehe ich«:
*was steht für dieses Vorhaben an?*

Dafür genügt lesender Zugriff auf einen iCalendar-Datenstrom – die `.ics`-Adresse, die
jeder Kalenderdienst als Abonnement ausgibt.

---

## Einrichten

Ein Kalender ist eine **Referenz** am Vorhaben, keine Einstellung. Typ `url`, Ziel die
Abonnement-Adresse:

```
lotse ref add Gartenhaus url "https://kalender.example.org/feeds/bau.ics" --rolle doku
```

Erkannt wird als Kalender, was auf `.ics` endet (Abfrageteil wird ignoriert) oder mit
`webcal://` beginnt. Alles andere ist eine normale Adresse.

`webcal://` ist `https://` mit anderem Namen – Lotse ersetzt es und fragt normal.

### Wo die Adresse steht

| Dienst | Wo |
|---|---|
| Google Kalender | Kalendereinstellungen → *Geheime Adresse im iCal-Format* |
| Apple iCloud | Kalender freigeben → *Öffentlicher Kalender* → Adresse kopieren |
| Nextcloud | Kalender → … → *Link kopieren*, `?export` anhängen |
| Proton Calendar | Einstellungen → Kalender freigeben → *Link mit allen* |
| Outlook / Microsoft 365 | Kalender freigeben → *ICS-Link veröffentlichen* |

> Diese Adressen sind **Geheimnisse**: wer sie hat, liest deinen Kalender. Sie stehen als
> Referenz im Klartext in der Datenbank – das ist derselbe Schutz wie für alles andere
> dort (verschlüsselt, aber ohne die zusätzliche Hülle des Tresors). Wer sie strenger
> behandeln will, legt sie in den [[Tresor]] und die Referenz auf einen unverfänglichen
> Namen.

Ein **lokaler Pfad** auf eine `.ics`-Datei geht auch – nützlich für einen Kalender, der
per Datei-Sync ohnehin auf dem Rechner liegt.

---

## Was angezeigt wird

```
lotse termine
lotse termine Gartenhaus
lotse termine --tage 30
```

Standardmäßig 90 Tage nach vorn. In der App stehen die nächsten Termine auf der
Projektseite rechts.

Je Termin: Titel, Datum, Uhrzeit (oder »ganztägig«), Ort, und ob er sich wiederholt.
Ganztägige Termine kommen vor den Terminen mit Uhrzeit desselben Tages.

### Wann geholt wird

Beim Öffnen der Projektseite, danach höchstens alle 15 Minuten neu. Nicht in einer
Hintergrundschleife: ein Kalender, der alle 30 Sekunden abgefragt wird, kostet Bandbreite
für nichts.

---

## Wiederholungen

Lotse rechnet `RRULE` aus, aber nur den Teil, den es sicher kann.

| Unterstützt | Beispiel |
|---|---|
| `FREQ=DAILY`, `WEEKLY`, `MONTHLY`, `YEARLY` | täglich, wöchentlich, monatlich, jährlich |
| `INTERVAL` | jede zweite Woche |
| `COUNT` | zehn Vorkommen insgesamt |
| `UNTIL` | bis zu einem Datum |
| `BYDAY` bei wöchentlichem Takt | montags und donnerstags |
| `WKST` | mit welchem Tag die Woche beginnt – entscheidet bei `INTERVAL > 1`, welche Tage noch zur selben Woche gehören |

Enthält eine Regel etwas, das Lotse **nicht** ausrechnet – `BYDAY=2MO` (zweiter Montag),
`BYSETPOS`, `BYMONTHDAY` –, dann rechnet es **nichts** hoch. Es sagt nur, dass sich der
Termin wiederholt, und zeigt das erste Vorkommen.

Lieber keine Angabe als eine falsche: ein Termin am falschen Tag ist schlimmer als ein
Termin ohne Datum, weil man sich darauf verlässt.

---

## Zeitzonen

**Lotse rechnet Zeitzonen nicht um.** Steht im Kalender

```
DTSTART;TZID=Europe/Berlin:20260909T100000
```

dann zeigt Lotse **zehn Uhr**. Genau das steht dort, und genau das ist gemeint.

Zeiten, die im Kalender als UTC stehen (`…Z`), werden als solche **gekennzeichnet**, nicht
umgerechnet.

### Warum

Weil eine Umrechnung ohne Zeitzonendatenbank geraten wäre – und die Datenbank ändert
sich: Länder schaffen die Sommerzeit ab, verschieben Umstellungstermine, ändern ihren
Versatz. Eine mitgelieferte Tabelle wäre nach zwei Jahren falsch, und dann zeigt Lotse
Termine zuverlässig eine Stunde verschoben an.

Für den Zweck – *was steht an?* – ist die Anzeige der Kalenderzeit richtig. Wer über
Zeitzonen hinweg plant, braucht einen Kalender, und Lotse ist keiner.

---

## Was Lotse **nicht** tut

| Nicht | Warum |
|---|---|
| Termine ins Logbuch schreiben | Dann stünde derselbe Termin an zwei Orten, und einer wäre irgendwann falsch. |
| Termine anlegen oder ändern | Lesender Zugriff genügt für den Zweck; Schreibzugriff wäre ein ungleich größeres Risiko. |
| Erinnern | Lotse ist kein Wecker. `VALARM` wird gelesen, aber ignoriert – seine Felder gehören nicht zum Termin. |
| CalDAV sprechen | Ein `.ics`-Abonnement kann jeder Dienst; CalDAV wäre Aufwand für dasselbe Ergebnis. |
| Einladungen beantworten | Siehe *kein Kalender*. |

Der Unterschied zur [[Wiedervorlage|Vorhaben]]: ein Termin ist eine Tatsache aus einem
anderen System, eine Wiedervorlage ist eine Marke in Lotse. Beides nebeneinander ist
richtig.

---

## Wenn nichts erscheint

| Meldung | Ursache |
|---|---|
| `Kein Projekt hat eine Kalender-Referenz` | Keine Referenz, die auf `.ics` endet oder mit `webcal://` beginnt ([[Referenzen]]). |
| `Netzwerk: …` | Adresse nicht erreichbar. Im Browser öffnen – kommt eine `.ics`-Datei? |
| Antwort ist HTML | Die Adresse zeigt auf die Kalender-**Seite**, nicht auf den Datenstrom. Bei Google ist es die »geheime Adresse im iCal-Format«, nicht der Link aus der Adresszeile. |
| Kalender kommt, keine Termine | Vielleicht liegen alle Termine mehr als 90 Tage voraus: `lotse termine --tage 365`. |

Das Modul liest keine Tresor-Werte – nachgewiesen durch `scripts/modulgrenzen.sh`
([[Sicherheit]]).
