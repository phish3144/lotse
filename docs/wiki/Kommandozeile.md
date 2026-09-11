# Kommandozeile

`lotse` kann alles, was die App kann: erfassen, suchen, Tresor, Ordner durchsuchen,
exportieren, abgleichen. Beide arbeiten auf demselben Datenordner.

Ein Kniff macht sie im Alltag schnell: **`lotse` erkennt das Projekt am
Arbeitsverzeichnis.** Stehst du in einem Ordner, der als Vorhaben übernommen wurde,
weiß `lotse log` von selbst, wohin der Eintrag gehört.

```
Usage: lotse [OPTIONS] <COMMAND>

Options:
      --home <HOME>   Datenordner (Standard: $LOTSE_HOME oder das Nutzer-Datenverzeichnis)
  -h, --help          Hilfe
  -V, --version       Version
```

Das Master-Passwort wird abgefragt. Für Skripte geht es über die Umgebungsvariable
`LOTSE_PASSWORD` – mit den üblichen Vorbehalten (Prozessliste, Shell-Verlauf).

---

## Einrichten

```bash
lotse init
```

Legt Konto und Datenbank auf diesem Gerät an. Zeigt Wiederherstellungscode und
Desktop-Schlüssel **genau einmal** – siehe **[[Erste Schritte|Erste-Schritte]]**.

## Erfassen

```bash
lotse log "Schalung geprüft. Nächster Schritt: Beton bestellen."
lotse log "@gartenhaus Bewehrung klären" --art offen
lotse log "Streifenfundament statt Platte" --projekt Gartenhaus --art entscheidung
```

Das Ziel wird in dieser Reihenfolge bestimmt: `--projekt` → `@projekt` im Text →
Arbeitsverzeichnis → Postkorb.

Arten: `notiz` (Standard), `offen`, `entscheidung`, `uebergabe`.

## Überblick

```bash
lotse hafen            # alle Vorhaben mit Auffälligkeit
lotse offen            # offene Fäden, projektübergreifend
lotse erledigt <id>    # Faden abhaken
lotse suche "bewehrung"
```

## Vorhaben

```bash
lotse projekt neu "Gartenhaus" --vorlage haus_garten --kurs "Fundament bis Oktober."
lotse projekt liste
lotse projekt zeige Gartenhaus        # Wo-war-ich-Brief plus Logbuch
lotse projekt kurs Gartenhaus "Neuer Kurs."
lotse projekt status Gartenhaus pausiert --notiz "Warte auf Beton."
lotse projekt loeschen Gartenhaus
```

`pausiert` und `wartet` verlangen `--notiz` – die Übergabenotiz.

Vorlagen: `software`, `hardware_maker`, `haus_garten`, `kreativ`,
`finanzen_verwaltung`, `lernen_forschung`, `reise_veranstaltung`, `generisch`.

Projekte lassen sich überall per ID **oder per Titel** ansprechen; ein Teilstück des
Titels genügt, solange es eindeutig ist.

## Referenzen

```bash
lotse ref add Gartenhaus ordner ~/Bau/gartenhaus --rolle material
lotse ref add Gartenhaus url https://github.com/name/repo
lotse ref add Gartenhaus physisch "Keller, Regal 3, blaue Kiste"
lotse ref liste Gartenhaus
lotse ref pruefen <id>
```

`pruefen` sieht nach, ob es das Ziel noch gibt: Pfade auf der Platte, Adressen im Netz
(GitHub und GitLab über ihre API, also auch private Repos, wenn ein Token hinterlegt
ist).

## Tresor

```bash
lotse tresor add "Fritzbox Gartenhaus" --projekt Gartenhaus passwort=xyz user=admin
lotse tresor add "Vereinskonto" --stufe nur_desktop --projekt Verein pin=1234
lotse tresor liste
lotse tresor zeige <id> passwort
lotse tresor loeschen <id>
```

`zeige` entschlüsselt **genau ein Feld** und gibt es aus. Siehe **[[Tresor]]**.

## Ordner

```bash
lotse scan ~/code ~/Bau        # Kandidaten finden (Hafeneinfahrt)
lotse uebernehmen ~/code/lampe --titel "Lampe ESP32"
lotse beobachten ~/code        # läuft bis Strg+C
```

`beobachten` schreibt geänderte Dateien und neue Commits verdichtet ins Logbuch – eine
Zeile pro Vorhaben und Tag. Nur Namen und Zeiten, keine Inhalte. Siehe **[[Beobachter
und Erkennung|Beobachter-und-Erkennung]]**.

## Gegenseite und Kalender

```bash
lotse gegenseite               # GitHub/GitLab abfragen, verdichtet ins Logbuch
lotse termine --tage 90        # anstehende Termine aus .ics-Abos
```

## Abgleich

```bash
lotse sync register --url https://api.example --email ich@example
lotse sync login    --url https://api.example --email ich@example
lotse sync jetzt               # pushen, dann pullen
lotse sync status
lotse sync geraete
lotse sync widerrufen <geraet-id>
```

Siehe **[[Abgleich]]**.

## Export

```bash
lotse export spiegel ~/Backup/lotse-markdown
lotse export bundle ~/Backup/lotse.json.age
```

Der Spiegel ist Klartext-Markdown **ohne Tresor**. Das Bundle enthält alles, mit `age`
verschlüsselt – und lässt sich auch ohne Lotse öffnen. Siehe **[[Export und
Fluchtweg|Export-und-Fluchtweg]]**.

## Updates

```bash
lotse update
```

Sieht nach, ob es eine neuere Version gibt, und nennt die Datei für dein System. **Lädt
nichts herunter** – der Selbst-Austausch ist der Desktop-App vorbehalten, weil nur sie
sich sinnvoll selbst ersetzen kann. Siehe **[[Updates]]**.

## Assistenten

```bash
lotse mcp
```

Startet den MCP-Server über stdin/stdout. Für den Alltag ist der Zugang aus der
laufenden App meist bequemer – siehe **[[Assistenten (MCP)|Assistenten-MCP]]**.

---

## Rezepte

**Beim Verlassen eines Projekts eine Zeile schreiben** – als Shell-Funktion:

```bash
feierabend() { lotse log "$*"; }
# feierabend "Schalung fertig. Morgen: Beton bestellen."
```

**Am Morgen sehen, was liegengeblieben ist:**

```bash
lotse hafen && lotse offen
```

**Git-Hook, der jeden Commit mitschreibt** (`.git/hooks/post-commit`):

```bash
#!/bin/sh
lotse log "Commit: $(git log -1 --format=%s)" >/dev/null 2>&1 || true
```

Meist unnötig: der **Beobachter** macht das von selbst und verdichtet auf eine Zeile pro
Tag statt einer pro Commit.
