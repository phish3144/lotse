# Vorhaben

Ein Vorhaben ist die Klammer um alles, was zu einer Sache gehört: das Gartenhaus, der
Rust-Compiler-Patch, die Vereinskasse, der Italienischkurs. In Lotse heißt es im Code
*Projekt*, in der Oberfläche *Vorhaben* – gemeint ist dasselbe, und *Projekt* klingt nach
Arbeit, während die Hälfte der Vorhaben keine ist.

Jedes Feld im Einzelnen: [[Datenmodell]].

---

## Der Kurs

Ein Satz. Worum geht es, was ist das Ziel.

> Fundament, Wände, Dach bis Oktober.
> Vereinskasse 2026 abschließen und dem Prüfer vorlegen.
> Italienisch bis B1, damit die Reise im Mai etwas bringt.

Der Kurs ist das wertvollste Feld in Lotse und das, das am häufigsten leer bleibt. Er ist
die Zeile, die dir in drei Monaten sagt, was du eigentlich wolltest – nicht was du getan
hast, das steht im [[Logbuch]].

Ein guter Kurs nennt ein **Ende**. »Am Gartenhaus arbeiten« ist kein Kurs, sondern eine
Beschreibung des Zustands. »Dach dicht bis zum ersten Frost« ist einer: man kann
feststellen, ob er erreicht ist.

Setzen: beim Anlegen mit `--kurs`, später mit `lotse projekt kurs <PROJEKT> <KURS>` oder
in der App oben auf der Projektseite.

---

## Status

Sechs Zustände, und nur einer kann auffallen.

| Status | Wann | Verlangt eine Notiz |
|---|---|---|
| **Idee** | Noch nicht angefangen. Notiert, damit es nicht vergessen wird. | nein |
| **Aktiv** | Läuft. Erscheint im Hafen unter *Auf See*. | nein |
| **Pausiert** | Bewusst unterbrochen. Du hast entschieden, es liegen zu lassen. | **ja** |
| **Wartet** | Hängt an etwas außerhalb: Antwort, Lieferung, Genehmigung, Termin. | **ja** |
| **Abgeschlossen** | Fertig. Bleibt lesbar, fällt nicht mehr auf. | nein |
| **Eingemottet** | Aufgegeben oder auf unbestimmte Zeit weggelegt. | nein |

### Warum Pausieren eine Notiz verlangt

Weil ein Vorhaben, das ohne Grund ruht, in drei Monaten genau die Frage erzeugt, die
Lotse beantworten soll. Die Notiz landet als Art `uebergabe` im Logbuch und ist das
Erste, was der [[Wo-war-ich-Brief|Wo-war-ich-Brief]] zeigt, wenn du zurückkommst.

Ein Satz genügt, und er darf banal sein:

```
lotse projekt status Gartenhaus wartet --notiz "Warte auf Angebot vom Statiker"
```

Das ist der Unterschied zwischen »irgendwas war da noch« und »ah ja, der Statiker«.

### Der Unterschied zwischen *pausiert* und *wartet*

**Pausiert** heißt: *ich* mache nicht weiter. Die Entscheidung liegt bei dir, und du
nimmst es wieder auf, wenn du willst.

**Wartet** heißt: ich *kann* nicht weitermachen. Es hängt an jemand anderem. Das ist die
Unterscheidung, die zählt, wenn man eine Liste durchgeht: bei *wartet* lohnt eine
Nachfrage, bei *pausiert* eine Entscheidung.

---

## Wiedervorlage

Ein Datum. Ab diesem Tag wird das Vorhaben **überfällig – unabhängig vom Status**.

Das ist der einzige Weg, wie ein ruhendes Vorhaben von sich aus wieder auf sich
aufmerksam macht. Ohne Wiedervorlage ist *pausiert* endgültig still.

```
lotse projekt status Kasse wartet \
  --notiz "Beleg vom Verein fehlt, angemahnt" \
  --wiedervorlage 2026-10-01
```

Am 2. Oktober steht die Kasse oben im Hafen, auch wenn sie *wartet*.

Eine Wiedervorlage ist **kein Termin**. Lotse ist kein Kalender ([[Nicht-Ziele]]) und
erinnert nicht um neun Uhr. Sie ist eine Marke, ab wann das Vorhaben wieder auffallen
soll. Echte Termine kommen aus dem [[Kalender]].

---

## Erwartungsintervall

Nach wie vielen Tagen ohne Kontakt ein Vorhaben auffällt. Das Feld, das den Hafen
brauchbar macht: ohne es wäre alles gleich dringend, also nichts.

Vorbelegt aus der Vorlage, danach frei änderbar – eine Vereinskasse, die man monatlich
anfasst, bekommt 30 statt 90 Tage.

| Vorlage | Standard |
|---|---|
| Software | 14 Tage |
| Hardware & Maker | 30 Tage |
| Kreativ | 30 Tage |
| Lernen & Forschung | 30 Tage |
| Allgemein | 30 Tage |
| Haus & Garten | 60 Tage |
| Finanzen & Verwaltung | 90 Tage |
| Reise & Veranstaltung | 120 Tage |

Die Zahlen sind keine Wissenschaft, sondern Erfahrung: ein Software-Projekt, das zwei
Wochen still ist, hat meist ein Problem. Ein Gartenhaus, das zwei Monate still ist, hat
Winter.

---

## Auffälligkeit

Wie dringend ein Vorhaben aussieht. Drei Stufen, aus vier Angaben berechnet – nichts
davon stellst du direkt ein.

| Stufe | Wann |
|---|---|
| **Ruhig** | Alles in Ordnung, oder das Vorhaben ruht bewusst. |
| **Auffällig** | Status `aktiv` und länger still als das Erwartungsintervall. |
| **Überfällig** | Status `aktiv` und länger still als das **doppelte** Intervall – **oder** die Wiedervorlage ist verstrichen, egal in welchem Status. |

»Tage seit letztem Kontakt« zählt ab der letzten Notiz, welcher Quelle auch immer. Gibt
es keine, ab `zuletzt_beruehrt`, mindestens aber ab dem Anlegedatum.

Ein Gartenhaus mit 60 Tagen Intervall ist also nach 61 Tagen auffällig und nach 121 Tagen
überfällig. Eine verstrichene Wiedervorlage setzt beides außer Kraft und macht sofort
überfällig.

### Warum nur *aktiv* auffallen kann

Weil sonst jedes abgeschlossene Vorhaben nach einem Jahr rot wäre und die Farbe nichts
mehr bedeutete. Ein Vorhaben bewusst ruhen zu lassen, muss ohne schlechtes Gewissen
möglich sein – das ist der halbe Zweck von *vor Anker*.

---

## Vorlagen

Acht Arten. Die Vorlage wirkt **nur beim Anlegen**: sie belegt Erwartungsintervall und
Tags vor. Danach ist alles frei, und die Vorlage ist nur noch eine Beschriftung.

| Kurzname (CLI) | In der Oberfläche |
|---|---|
| `software` | Software |
| `hardware` | Hardware & Maker |
| `haus` | Haus & Garten |
| `kreativ` | Kreativ |
| `finanzen` | Finanzen & Verwaltung |
| `lernen` | Lernen & Forschung |
| `reise` | Reise & Veranstaltung |
| `generisch` | Allgemein |

Beim Übernehmen eines Kandidaten schlägt die Erkennung eine Vorlage vor und nennt, woran
sie es erkannt hat ([[Erkennungsregeln]]). Ein falscher Vorschlag ist kein Problem:
übernehmen, Vorlage ändern, weiter.

---

## Der Postkorb

Ein Vorhaben, das Lotse bei Bedarf selbst anlegt. Dort landet, was du notierst, ohne dass
ein Vorhaben erkennbar ist:

```
lotse log "Idee: Regenwassertonne an die Nordseite"
Kein Projekt erkannt – landet im Postkorb.
```

Der Postkorb ist der Ort, an dem man nicht nachdenken muss, wohin etwas gehört. Aufräumen
geht später: in der App eine Notiz per *Verschieben* dem richtigen Vorhaben zuordnen.

Er erscheint nicht in der Hafen-Übersicht, damit er nicht ständig auffällt.

---

## Tags

Freie Schlagworte, aus der Vorlage vorbelegt. Sie sortieren nichts und lösen nichts aus –
sie sind zum Suchen da. Wer sie nicht braucht, ignoriert sie; Lotse verlangt keine
Taxonomie.

---

## Abgeleitete Vorhaben

Wird aus einem Vorhaben ein zweites (ein Nebenprodukt wird eigenständig), merkt Lotse
sich das in `abgeleitet_von`. Der Verlauf des ursprünglichen bleibt, wo er ist – Notizen
werden nicht kopiert. Das Feld ist eine Spur, kein Mechanismus.

---

## Löschen

```
lotse projekt loeschen Gartenhaus
```

Löscht Vorhaben, Logbuch und Referenzen. Tresor-Einträge werden crypto-geshreddert: der
gewrappte Eintragsschlüssel verschwindet, und damit sind die Werte auch in alten
Sync-Kopien und alten Sicherungen unlesbar ([[Tresor]]).

Meist ist **Eingemottet** die bessere Wahl. Ein eingemottetes Vorhaben fällt nicht auf,
kostet nichts und ist in drei Jahren noch lesbar, wenn die Frage kommt, wie man das
damals gemacht hat.
