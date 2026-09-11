# Logbuch

Der Verlauf eines Vorhabens. Was passiert ist, in der Reihenfolge, in der es passiert
ist – von Hand geschrieben oder automatisch mitgeschrieben.

Das Logbuch ist der Kern von Lotse. Der [[Wo-war-ich-Brief|Wo-war-ich-Brief]], die
Auffälligkeit im Hafen, die Suche: alles liest daraus. Ein Vorhaben ohne Logbuch ist ein
Titel.

---

## Wie man hineinschreibt

**In der App:** Strg/Cmd + K öffnet die Schnellerfassung, überall, auch mitten in einer
anderen Ansicht. Text eintippen, Enter. Ohne Zielangabe geht es an das Vorhaben, das
gerade offen ist, sonst in den Postkorb.

**Im Terminal:**

```
lotse log "Schalung fertig, Beton für Montag bestellt" -p Gartenhaus
lotse log "@Gartenhaus Beton bestellt"
cd ~/bau && lotse log "Beton bestellt"
```

Die drei Wege sind gleichwertig. Der dritte funktioniert, weil `~/bau` als Referenz des
Vorhabens hinterlegt ist ([[Referenzen]]).

**Von allein:** der Ordner-Beobachter, Git, die Gegenseite, ein KI-Assistent über MCP.
Siehe *Quellen* unten.

---

## Arten

Fünf Arten. Sie unterscheiden, was für ein Eintrag es ist – nicht, wie wichtig.

### Log

Der Normalfall. Etwas ist passiert.

> Schalung fertig, Beton für Montag bestellt.

Gut sind Einträge, die einen **Zustand** oder einen **nächsten Schritt** nennen.
Schlecht sind Einträge, die nur ein Gefühl festhalten (»viel geschafft heute«) – die
helfen in drei Monaten nicht.

### Offen

Ein offener Faden. Etwas, das noch zu tun oder zu klären ist. Bleibt in
**[[Offene Punkte|Offene-Punkte]]** stehen, bis er abgehakt ist.

```
lotse log --offen "Statiker fragen wegen Dachlast" -p Gartenhaus
lotse offen
01M28KAQB1308AJGF0326ZDMZT  [Gartenhaus] Statiker fragen wegen Dachlast
lotse erledigt 01M28KAQB1308AJGF0326ZDMZT
```

Abhaken löscht nichts: der Eintrag bleibt im Logbuch und bekommt `erledigt_am`. Dass
etwas erledigt wurde, ist selbst eine Information.

> **Offene Fäden sind keine Aufgabenliste.** Sie sind das, was *dieses* Vorhaben
> blockiert oder offen lässt. Wer eine Aufgabenverwaltung will, soll eine benutzen – Lotse
> baut absichtlich keine ([[Nicht-Ziele]]).

### Entscheidung

Eine Entscheidung mit Begründung. Wird in der App hervorgehoben, weil man genau danach
später sucht.

```
lotse log --entscheidung "Ziegel statt Blech" -p Gartenhaus
```

Mit `--entscheidung` gibt es eine Vorlage für Alternativen und Begründung. Das ist der
Eintrag, der in zwei Jahren die Frage »warum eigentlich Ziegel?« beantwortet – die Frage,
die am häufigsten kommt und am schlechtesten dokumentiert ist.

### Status

Ein Statuswechsel. Schreibt Lotse selbst, damit im Logbuch steht, wann etwas pausiert
wurde und wann es wieder anlief.

### Übergabe

Die Notiz beim Pausieren oder Warten. Das Erste, was der Brief zeigt.

Sie entsteht aus `--notiz` bei `lotse projekt status` und ist dort Pflicht – siehe
[[Vorhaben]].

---

## Quellen

Wer geschrieben hat. Acht Werte, und sie sind der Grund, warum das Logbuch mehr ist als
ein Notizzettel.

| Quelle | Wer | Woher |
|---|---|---|
| `mensch` | du, in der App | Schnellerfassung, Projektseite |
| `cli` | du, im Terminal | `lotse log` |
| `datei` | der Ordner-Beobachter | Dateien geändert, verdichtet auf eine Notiz pro Tag |
| `git` | ein lokales Repo | Commits seit dem letzten Blick, eine Notiz pro Tag |
| `mcp` | ein KI-Assistent | [[Assistenten (MCP)\|Assistenten-MCP]] |
| `ki` | eine Verdichtung, die **du** übernommen hast | [[KI]] |
| `import` | von außen eingelesen | |
| `sync` | vom Abgleich hereingekommen | auch Konfliktnotizen ([[Sync-Protokoll]]) |

### Die Grenze, die zählt

`mensch` und `cli` gelten als **dein** Kontakt. Alles andere ist Aktivität, die passiert
ist, während du nicht hingesehen hast.

Genau diese Grenze macht den Brief nützlich: er zählt nicht »10 Einträge«, sondern »seit
deinem letzten Besuch: 40 Dateiänderungen, 12 Commits«. Das ist die Antwort auf »ist hier
was passiert, während ich weg war«.

Deshalb setzt der Ordner-Beobachter `datei` und nicht `mensch`, auch wenn *du* die
Dateien geändert hast: Lotse weiß nicht, ob du hingesehen hast, nur dass sich etwas
bewegt hat.

### Warum verdichtet und nicht jede Zeile

Ein Beobachter, der jede Dateiänderung einzeln notiert, erzeugt pro Tag hundert Einträge
und macht das Logbuch unlesbar. Deshalb eine Notiz pro Projekt und Tag, die im Laufe des
Tages fortgeschrieben wird:

> 14 Dateien geändert, 2 neu: src/main.rs, src/store.rs, …

Dasselbe bei Git: eine Notiz pro Tag, nicht 200 Commit-Zeilen. Und dasselbe bei der
Gegenseite: eine Zeile mit Pull Requests, Issue-Zahl und Prüfstand, keine Ticketliste.

Dieses Muster hat sich beim Ordner-Beobachter bewährt und ist seitdem die Regel für
jede automatische Quelle.

---

## Die Zeitachse in der App

Auf der Projektseite ist das Logbuch eine Zeitachse: nach Tagen gruppiert, jüngstes
oben. Jeder Punkt ist nach Quelle gefärbt, sodass man auf einen Blick sieht, was von dir
kam und was von allein.

Die Gruppierung rechnet in **lokaler** Zeit. Ein Eintrag um 00:30 gehört zum
angebrochenen Tag, nicht zum vergangenen – auch wenn er in UTC noch vom Vortag ist.

---

## Suchen

```
lotse suche beton
notiz    Gartenhaus               Schalung fertig, [Beton] für Montag bestellt
```

Volltextsuche über Titel, Kurse, Logbücher und Referenzen. Der Treffer wird markiert – in
der App farbig, im Terminal in eckigen Klammern.

**Tresor-Inhalte sind nie enthalten.** Nicht, weil sie ausgefiltert werden, sondern weil
sie gar nicht im Index stehen ([[Datenablage]]). Das ist der Unterschied zwischen einer
Zusage und einer Garantie.

---

## Was nicht geht

**Bearbeiten.** Ein Logbuch, das man nachträglich glättet, ist kein Logbuch. Was falsch
ist, wird durch einen neuen Eintrag richtiggestellt.

**Löschen einzelner Einträge.** Nur das ganze Vorhaben lässt sich löschen.

**Anhängen von Dateien an eine Notiz.** Dateien werden als [[Referenzen]] hinterlegt –
sie bleiben dann dort, wo sie hingehören, statt in einer Datenbank zu verschwinden.

**Verschieben** geht: eine Notiz kann in der App einem anderen Vorhaben zugeordnet
werden. Dafür ist der Postkorb da.
