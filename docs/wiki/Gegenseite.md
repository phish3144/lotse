# Gegenseite: GitHub und GitLab

Der lokale Git-Log sagt, was **du** getan hast. Die Gegenseite sagt, was **andere** getan
haben und was auf dich wartet: offene Pull Requests, die Zahl offener Issues, den Zustand
der Prüfläufe. Bei einem Software-Vorhaben beantwortet das »wo stehe ich« oft besser als
der letzte Commit.

Lotse liest dort nur. Es schreibt nichts, kommentiert nichts, schließt nichts.

---

## Verbinden

Ein Feld. In den [[Einstellungen]] unter *Verbindungen*:

1. *Token erstellen …* öffnet die richtige Seite beim Hoster.
2. Token kopieren.
3. *Token einfügen*.

Das Token landet als Tresor-Eintrag; in den Einstellungen steht nur ein **Zeiger** darauf
([[Datenablage]]). Damit gilt für das Token derselbe Schutz wie für jedes andere
Geheimnis in Lotse.

### Nötige Rechte

| Hoster | Rechte |
|---|---|
| GitHub | Feingranulares Token mit *Contents: Read* und *Pull requests: Read* für die betroffenen Repos. Für öffentliche Repos genügt ein Token ohne jedes Recht. |
| GitLab | `read_api`. |

Mehr wäre unnötig. Lotse braucht keine Schreibrechte, und ein Token mit Schreibrechten
wäre ein unnötiges Risiko in einer Anwendung, die nur lesen will.

### Auf der Kommandozeile

Dort kommt das Token aus der Umgebung – `LOTSE_GITHUB_TOKEN`, `LOTSE_GITLAB_TOKEN` oder
als Ausweichweg `LOTSE_FORGE_TOKEN` ([[Umgebungsvariablen]]). Ist keine gesetzt, nimmt
die Kommandozeile denselben Tresor-Eintrag, den die App hinterlegt hat. Beide Oberflächen
teilen also eine Einstellung.

### Trennen

*Trennen* löscht den Zeiger und den Tresor-Eintrag. Das Token beim Hoster bleibt gültig –
dort musst du es selbst widerrufen, wenn du das willst. Lotse tut das nicht, weil ein
Programm, das fremde Zugangsdaten ungefragt ungültig macht, mehr kaputt macht als es
hilft.

---

## Wie Lotse das Repo findet

Aus einer Referenz vom Typ `git_repo` oder `url` am Vorhaben. Erkannt werden:

```
git@github.com:owner/repo.git
https://github.com/owner/repo
https://github.com/owner/repo.git
ssh://git@github.com/owner/repo
```

und dieselben Schreibweisen für `gitlab.com`. Auch `www.github.com` – dieselbe Adresse,
der Rest des Pfades bleibt, wie er ist.

Beim Übernehmen eines Kandidaten mit `.git` legt Lotse die Referenz selbst an, wenn ein
Remote eingetragen ist.

Selbst gehostete Instanzen (GitHub Enterprise, eigenes GitLab) erkennt Lotse noch nicht.
Der lokale Git-Log funktioniert dort natürlich.

---

## Wann abgefragt wird

| Anlass | Bedingung |
|---|---|
| Von Hand | *Jetzt abfragen* auf der Projektseite, oder `lotse gegenseite` |
| Mit dem Beobachter | Nur wenn *Mit dem Ordner-Beobachter mitlaufen lassen* eingeschaltet ist – höchstens alle 30 Minuten |

Nicht beim Öffnen jeder Projektseite. Ein Programm, das bei jedem Klick eine
Netzanfrage stellt, ist im Zug unbenutzbar und verbraucht das Anfragekontingent des
Hosters für nichts.

Wann zuletzt abgefragt wurde, steht in `meta` unter `forge_zuletzt` – daran hängt die
30-Minuten-Grenze.

---

## Was im Logbuch landet

**Eine** verdichtete Zeile mit Quelle `git`, nicht eine Liste:

> 3 offene PRs (»Updater automatisch«, »Wiki im Repo«, …), 12 offene Issues, Prüflauf grün
> auf `main`

Erfasst werden:

| Angabe | Inhalt |
|---|---|
| Offene Pull Requests | Anzahl und die ersten Titel |
| Offene Issues | nur die **Anzahl** |
| Prüfläufe | Zustand auf dem Standard-Branch: grün, rot, läuft |
| Standard-Branch | dessen Name |

### Warum Issues nur gezählt werden

Weil sie sonst zu offenen Fäden würden, und damit wäre Lotse ein zweites Ticketsystem mit
schlechterer Suche. Tickets und Backlogs stehen dauerhaft auf der Nicht-Liste
([[Nicht-Ziele]]). Eine Zahl sagt »hier liegt etwas«; das Bearbeiten gehört dorthin, wo
die Tickets leben.

Dasselbe Muster wie beim Ordner-Beobachter und beim Git-Leser: eine Zeile pro Tag statt
zweihundert Zeilen ([[Logbuch]]).

---

## Auf der Kommandozeile

```
lotse gegenseite                    # alle Vorhaben mit erkanntem Repo
lotse gegenseite Lotse              # nur eines
lotse gegenseite --trocken          # nur zeigen, nichts ins Logbuch
```

`--trocken` ist der richtige Weg, um zu sehen, ob Token und Erkennung stimmen, ohne das
Logbuch zu verschmutzen.

---

## Was nach außen geht

Nur die Abfrage: `GET` auf die API des Hosters, mit deinem Token, für die Repos, die als
Referenz hinterlegt sind. Sonst nichts. Keine Projektnamen, keine Logbuch-Einträge, keine
Kurse.

Das Modul, das diese Abfrage macht, kann den Tresor **nicht** lesen – es importiert nicht
aus `vault`, und `scripts/modulgrenzen.sh` prüft das bei jedem Bau. Das Token reicht der
Aufrufer herein; wer es holt, ist die Oberfläche oder die Kommandozeile. Siehe
[[Sicherheit]].

---

## Wenn es nicht geht

| Meldung | Ursache |
|---|---|
| `401` | Token falsch oder abgelaufen. Neu erstellen. |
| `403` mit Kontingent-Hinweis | Anfragekontingent erschöpft. Warten; die Kopfzeile nennt bis wann. |
| `403` ohne Kontingent-Hinweis | Rechte fehlen. Bei Feingranular-Tokens: ist das Repo ausgewählt? |
| `404` | Bei privaten Repos ohne Token normal – GitHub verrät nicht, dass sie existieren. Mit Token: Adresse prüfen. |
| `Kein Repo erkannt` | Keine Referenz vom Typ `git_repo` oder `url` auf github.com/gitlab.com am Vorhaben ([[Referenzen]]). |

Lotse unterscheidet bei `403` bewusst zwischen »keine Berechtigung« und »Kontingent
erschöpft«, weil die Antworten darauf entgegengesetzt sind: einmal Token ändern, einmal
warten.
