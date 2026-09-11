# Begriffe

Lotse hat wenige Bausteine. Wer sie kennt, kennt die Anwendung.

---

## Vorhaben (Projekt)

Alles, was länger dauert als ein Nachmittag und einen Verlauf hat. Eine Software, ein
Gartenhaus, die Vereinskasse, ein Kurs, eine Reise.

Ein Vorhaben trägt:

| Feld | Bedeutung |
|---|---|
| **Titel** | Wie du es nennst. |
| **Kurs** | Ein Satz: worum geht es, was ist das Ziel. |
| **Status** | Wo es gerade steht (siehe unten). |
| **Vorlage** | Grobe Art (Software, Haus & Garten, Kreativ …). Beeinflusst Vorschläge, sonst nichts. |
| **Erwartungsintervall** | Nach wie vielen Tagen Stille das Vorhaben auffallen soll. Standard 14. |
| **Wiedervorlage** | Ein Datum, an dem es sich melden soll. Optional. |
| **Tags** | Freie Schlagworte. |

### Kurs

Der wichtigste Satz im ganzen Programm. Nicht der Titel erklärt dir in drei Monaten,
was du wolltest – der Kurs tut es.

Gut: *„Fundament bis Oktober, danach Winterpause, Aufbau im Frühjahr.“*
Schlecht: *„Gartenhaus bauen.“*

### Status

| Status | Bedeutet | Fällt auf? |
|---|---|---|
| **Idee** | Noch kein Kurs, liegt still. | nein |
| **Aktiv** | Läuft. | ja, wenn es zu lange still ist |
| **Pausiert** | Bewusst unterbrochen. | nur bei Wiedervorlage |
| **Wartet** | Hängt an jemand anderem. | nur bei Wiedervorlage |
| **Abgeschlossen** | Fertig. | nein |
| **Eingemottet** | Aufgegeben, aber nicht gelöscht. | nein |

*Pausiert* und *wartet* heißen im Hafen zusammen **vor Anker**.

Der Wechsel nach *pausiert* oder *wartet* verlangt eine **Übergabenotiz**. Das ist der
einzige Zwang in Lotse, und er zahlt sich beim Wiederaufnehmen aus.

### Auffälligkeit

Lotse rechnet je Vorhaben einen von drei Zuständen aus. Er steht als farbiger Punkt an
der Karte und als Kante links:

| | Wann |
|---|---|
| 🟢 **ruhig** | Alles im Rahmen. |
| 🟡 **auffällig** | Mehr als 70 % des Erwartungsintervalls still. |
| 🔴 **überfällig** | Erwartungsintervall überschritten, oder die Wiedervorlage ist vorbei. |

*Abgeschlossen*, *eingemottet*, *pausiert* und *wartet* sind immer ruhig – sie ruhen ja
mit Absicht. Nur die Wiedervorlage greift auch dort.

---

## Logbuch

Der Verlauf eines Vorhabens, nach Tagen gruppiert. Jeder Eintrag hat eine **Art** und
eine **Quelle**.

### Arten

| Art | Wofür |
|---|---|
| **Notiz** | Was passiert ist. Der Normalfall. |
| **Offener Faden** | Eine Frage, die beim nächsten Mal im Weg steht. Abhakbar. |
| **Entscheidung** | Was entschieden wurde – und warum das *andere* verworfen wurde. |
| **Übergabe** | Der Stand beim Verlassen. Entsteht beim Statuswechsel. |
| **Status** | Automatisch bei Statuswechseln. |

### Quellen

| Quelle | Woher |
|---|---|
| **Mensch** | Von Hand in der Oberfläche geschrieben. |
| **CLI** | Über `lotse log`. |
| **Datei** | Vom Ordner-Beobachter: geänderte Dateien, verdichtet. |
| **Git** | Commits, eine Zeile pro Tag. |
| **Import** | Beim Übernehmen aus der Hafeneinfahrt. |
| **MCP** | Von einem KI-Assistenten eingetragen. |
| **KI** | Aus einer Verdichtung übernommen. |

Im Logbuch trägt jeder Eintrag einen Punkt in der Farbe seiner Quelle: Bernstein für
das, was von Hand kam, Blau für das Maschinelle.

### Offene Fäden

Ein offener Faden ist **keine Aufgabe**. Es gibt keine Prioritäten, keine Zuweisungen,
keine Fristen. Es ist eine Frage, die beim nächsten Mal im Weg steht – mehr nicht.

Alle offenen Fäden über alle Vorhaben stehen unter **Offene Punkte**, gebündelt nach
Vorhaben. Wer Tickets braucht, braucht ein Ticketsystem; siehe **[[Nicht-Ziele]]**.

---

## Referenz

Zeigt nach außen: wo das Material liegt. Lotse zieht **nichts** nach innen – es merkt
sich nur den Weg.

| Typ | Beispiel | Prüfbar? |
|---|---|---|
| **Ordner** | `~/Bau/gartenhaus` | ja, auf diesem Gerät |
| **Git-Repo** | `~/code/lotse` oder `https://github.com/…` | ja – lokal über den Pfad, per Adresse übers Netz |
| **URL** | `https://…`, auch `.ics`-Kalender | ja, per Abfrage |
| **Datei** | ein einzelnes Dokument | ja, auf diesem Gerät |
| **Physisch** | „Keller, Regal 3, blaue Kiste“ | nein, und das ist in Ordnung |

Jede Referenz hat eine **Rolle**: *Material*, *Ergebnis* oder *Doku*.

Pfade gelten nur auf dem Gerät, auf dem sie angelegt wurden – ein Pfad ist auf einem
anderen Rechner nichts wert. Adressen gelten überall.

---

## Zugang (Tresor)

Zugangsdaten, die an einem Vorhaben hängen: der Router im Gartenhaus, der Admin-Login
der eigenen App, die Kontonummer des Vereins. Ende-zu-Ende verschlüsselt, mit einem
eigenen Schlüssel je Eintrag.

Nicht dafür gedacht: die Web-Logins des Alltags. Siehe **[[Tresor]]**.

---

## Hafen

Die Startseite. Zeigt in dieser Reihenfolge:

1. **Heute wichtig** – bis zu drei Vorhaben, die auffallen
2. **Auf See** – die übrigen aktiven
3. **Vor Anker** – pausiert und wartet, eingeklappt
4. **Ideen** – ohne Kurs, liegen still

Rechts daneben die **Hafeneinfahrt**: erkannte Ordner, die auf deine Entscheidung
warten. Sie stehen daneben und nicht dazwischen, damit sie den Blick auf die Vorhaben
nicht zerschneiden.

## Postkorb

Ein Vorhaben wie jedes andere, das Lotse selbst anlegt. Dort landet alles aus der
Schnellerfassung, dem kein Projekt zugeordnet war. Aufräumen kannst du später – mit
*Zuordnen zu* an jedem Eintrag.

## Wo-war-ich-Brief

Erscheint oben auf der Projektseite, wenn ein Vorhaben länger geruht hat als sein
Erwartungsintervall. Er fasst zusammen: letzter Kontakt, offene Fäden, was sich seit dem
letzten Besuch getan hat, nach Quellen aufgeschlüsselt.

Er wird **gerechnet, nicht gespeichert** – und ohne KI. Die optionale Verdichtung
(**[[KI]]**) macht daraus auf Wunsch einen Fließtext, aber die Zahlen stehen auch ohne.
