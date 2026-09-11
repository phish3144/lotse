# Kommandozeile: vollständige Referenz

Jedes Kommando, jede Option, jeder Standardwert. Für den Einstieg ist
**[[Kommandozeile]]** der bessere Ort – hier steht alles, auch das, was man selten
braucht.

Die Ausgaben auf dieser Seite sind echt: sie stammen aus einem Wegwerf-Datenordner, in
dem die Beispiele der Reihe nach ausgeführt wurden.

---

## Globale Option

Sie gilt für **jedes** Kommando und steht vor oder nach dem Kommando.

| Option | Bedeutung |
|---|---|
| `--home <PFAD>` | Datenordner. Standard: `$LOTSE_HOME`, sonst das Datenverzeichnis des Systems (siehe [[Datenablage]]). |
| `-h`, `--help` | Hilfe zu diesem Kommando. Funktioniert auf jeder Ebene: `lotse projekt status --help`. |
| `-V`, `--version` | Version, sonst nichts. |

Alle Umgebungsvariablen stehen in **[[Umgebungsvariablen]]**. Für Skripte sind
`LOTSE_PASSWORD` und `LOTSE_HOME` die wichtigsten.

---

## Einrichten

### `lotse init`

Legt Konto und Datenbank auf diesem Gerät an. Einmal pro Gerät. Fragt das
Master-Passwort ab und zeigt danach **einmalig** Wiederherstellungscode und
Desktop-Schlüssel.

| Option | Standard | Bedeutung |
|---|---|---|
| `--geraet <NAME>` | `Dieser Rechner` | Name dieses Geräts. Steht später in `lotse sync geraete`. |
| `--ohne-bestaetigung` | aus | Überspringt das Abtippen des Wiederherstellungscodes. Nur für Skripte; gleichbedeutend mit `LOTSE_SKIP_CONFIRM=1`. |
| `--sync-url <URL>` | – | Registriert sofort beim Sync-Dienst. Nur zusammen mit `--email`. |
| `--email <ADRESSE>` | – | Adresse für die Registrierung. |

```
$ lotse init --geraet Laptop
Leite Schlüssel ab …

Wiederherstellungscode (einmalige Anzeige, in Proton Pass ablegen):
    KPFV-P8HV-1ZX2-XFCQ-3KE2-5VP3-9C

Desktop-Schlüssel (einmalige Anzeige, in Proton Pass ablegen; nie auf fremden Rechnern eingeben):
    ZG0H-DTZ1-7FMA-SRXB-0YPK-F9G5-NC-8BE1-KNGW-WMAP-Y451-04TP-RHV4-E8
    (kein Schlüsselbund verfügbar; für »nur Desktop«-Einträge LOTSE_DESKTOP_KEY setzen)

Eingerichtet in /home/du/.local/share/lotse
```

Die Zeile über den Schlüsselbund erscheint nur, wenn keiner erreichbar ist – auf
Servern, in Containern, in Desktop-Sitzungen ohne Schlüsselbund-Dienst. Beide Codes
sind ab hier **nicht mehr abrufbar**; mehr dazu in [[Schlüssel und Krypto|Schluessel-und-Krypto]].

`init` legt ein Projekt an: *Lotse* selbst. Damit ist die Startseite nicht leer und du
hast einen Ort für Notizen über das Werkzeug.

---

## Erfassen

### `lotse log [TEXT]…`

Schreibt einen Logbuch-Eintrag. Das Kommando, das man am häufigsten braucht.

| Option | Bedeutung |
|---|---|
| `-p`, `--projekt <PROJEKT>` | Zielprojekt als ID, Titel oder Titelanfang. |
| `--offen` | Legt den Eintrag als offenen Faden an (Art `offen`). |
| `--entscheidung` | Legt ihn als Entscheidung an (Art `entscheidung`), mit Vorlage für Begründung und Alternativen. |

Das Projekt wird in dieser Reihenfolge bestimmt:

1. `--projekt`
2. ein `@projekt` im Text (`lotse log "@Gartenhaus Beton bestellt"`)
3. der Ordner, in dem du stehst, falls er als Referenz eines Projekts hinterlegt ist
4. sonst der **Postkorb** (siehe [[Begriffe]])

```
$ lotse log "Schalung fertig, Beton für Montag bestellt" -p Gartenhaus
→ Gartenhaus (log)

$ lotse log --offen "Statiker fragen wegen Dachlast" -p Gartenhaus
→ Gartenhaus (offen)
```

Ohne Text öffnet sich `$EDITOR`. Mehrere Wörter brauchen keine Anführungszeichen, aber
eine Shell frisst sonst `!`, `"` und `#` – in der Praxis sind Anführungszeichen
bequemer.

### `lotse erledigt <NOTIZ_ID>`

Hakt einen offenen Faden ab. Die ID liefert `lotse offen`. Der Eintrag bleibt im
Logbuch stehen und bekommt `erledigt_am`; nichts wird gelöscht.

---

## Überblick

### `lotse hafen`

Die Startseite im Terminal: alle Vorhaben, gruppiert und sortiert nach dem, was
Aufmerksamkeit braucht.

```
$ lotse hafen
Auf See
   Lotse                           0 T   0 offen  Konto eingerichtet. Lotse ist Projekt Nr. 1 in Lotse.

Vor Anker
   Gartenhaus                      0 T   1 offen  Warte auf Angebot vom Statiker
```

Die Spalten: Titel, Tage seit dem letzten Kontakt, Zahl offener Fäden, letzte Notiz.
*Auf See* sind die aktiven Vorhaben, *Vor Anker* die ruhenden. Wie sich die Reihenfolge
ergibt, steht in [[Vorhaben]] unter *Auffälligkeit*.

### `lotse offen`

Alle offenen Fäden, über alle Vorhaben hinweg. Mit `-p`/`--projekt` nur eines.

```
$ lotse offen
01M28KAQB1308AJGF0326ZDMZT  [Gartenhaus] Statiker fragen wegen Dachlast
```

Die ID vorn ist das Argument für `lotse erledigt`.

### `lotse suche [TEXT]…`

Volltextsuche über Titel, Kurse, Logbücher und Referenzen. Der Treffer wird in
eckigen Klammern markiert.

```
$ lotse suche beton
notiz    Gartenhaus               Schalung fertig, [Beton] für Montag bestellt
```

Tresor-Inhalte sind **nie** enthalten – weder Werte noch Titel. Siehe [[Tresor]].

---

## Vorhaben

### `lotse projekt neu <TITEL>`

| Option | Standard | Bedeutung |
|---|---|---|
| `--vorlage <V>` | `generisch` | Eine von `software`, `hardware`, `haus`, `kreativ`, `finanzen`, `lernen`, `reise`, `generisch`. Bestimmt Erwartungsintervall und Standard-Tags ([[Vorhaben]]). |
| `--kurs <SATZ>` | – | Der eine Satz, worum es geht. Nachträglich mit `lotse projekt kurs`. |

```
$ lotse projekt neu "Gartenhaus" --vorlage haus --kurs "Fundament, Wände, Dach bis Oktober"
Angelegt: Gartenhaus (01M28KAQ3BYYG0BPNHRHQXB8XR)
```

### `lotse projekt liste`

Alle Vorhaben mit ID, Status und Vorlage.

```
$ lotse projekt liste
01M28KBGZ451M7PXVBMEXFSHKS  aktiv        Software               meinapp
01M28KAQ3BYYG0BPNHRHQXB8XR  wartet       Haus & Garten          Gartenhaus
01M28KAPZG862XWZBTR8V68C2D  aktiv        Software               Lotse
```

### `lotse projekt zeige <PROJEKT>`

Alles über ein Vorhaben: Kurs, Status, offene Fäden, Logbuch.

```
$ lotse projekt zeige Gartenhaus
Gartenhaus  [aktiv]  Haus & Garten
Kurs: Fundament, Wände, Dach bis Oktober

Offen:
  - Statiker fragen wegen Dachlast

Logbuch:
  2026-09-11 16:03  offen        cli     Statiker fragen wegen Dachlast
  2026-09-11 16:03  log          cli     Schalung fertig, Beton für Montag bestellt
```

Die vierte Spalte im Logbuch ist die **Quelle** – wer geschrieben hat (`cli`, `mensch`,
`datei`, `git`, `mcp`, `ki`, `import`, `sync`). Siehe [[Logbuch]].

### `lotse projekt status <PROJEKT> <STATUS>`

Wechselt den Status. Erlaubt sind `idee`, `aktiv`, `pausiert`, `wartet`,
`abgeschlossen`, `eingemottet`.

| Option | Bedeutung |
|---|---|
| `--notiz <TEXT>` | Übergabenotiz. **Pflicht** bei `pausiert` und `wartet`. |
| `--wiedervorlage <JJJJ-MM-TT>` | Ab diesem Tag wird das Vorhaben wieder überfällig, unabhängig vom Status. |

```
$ lotse projekt status Gartenhaus wartet --notiz "Warte auf Angebot vom Statiker" --wiedervorlage 2026-10-01
Gartenhaus → wartet
```

Warum die Notiz Pflicht ist: ein Vorhaben ohne Grund zu pausieren erzeugt in drei
Monaten genau die Frage, die Lotse beantworten soll. Die Notiz landet als Art
`uebergabe` im Logbuch und ist das, was der [[Wo-war-ich-Brief|Wo-war-ich-Brief]] zuerst zeigt.

### `lotse projekt kurs <PROJEKT> <KURS>`

Setzt den Kurs neu. Ein Satz, keine Beschreibung.

### `lotse projekt loeschen <PROJEKT>`

Löscht Vorhaben, Logbuch und Referenzen. Tresor-Einträge des Vorhabens werden
crypto-geshreddert (siehe [[Tresor]]). Fragt nach.

---

## Referenzen

Referenzen sind Zeiger nach draußen: Ordner, Repos, Adressen, physische Dinge. Mehr in
**[[Referenzen]]**.

### `lotse ref add <PROJEKT> <TYP> <ZIEL>`

Typ ist einer von `ordner`, `git_repo`, `url`, `datei`, `physisch`, `geraet`,
`passwortmanager`, `anhang`.

| Option | Standard | Bedeutung |
|---|---|---|
| `--rolle <R>` | `material` | `material` (Eingang), `ergebnis` (Ausgang), `doku`. |

```
$ lotse ref add Gartenhaus ordner /home/du/bau --rolle material
Referenz 01M28KB1CS4X6NSVFFXDK063EN angelegt.
$ lotse ref add Gartenhaus url https://example.com/statik --rolle doku
Referenz 01M28KB1GMB5R9FV7F1TZ7THG8 angelegt.
```

### `lotse ref liste <PROJEKT>`

```
$ lotse ref liste Gartenhaus
01M28KB1CS4X6NSVFFXDK063EN  ordner          nicht_pruefbar  /home/du/bau
01M28KB1GMB5R9FV7F1TZ7THG8  url             nicht_pruefbar  https://example.com/statik
```

Die dritte Spalte ist der Prüfstatus: `ok`, `nicht_erreichbar` oder `nicht_pruefbar`.
Frisch angelegte Referenzen sind `nicht_pruefbar`, bis geprüft wurde.

### `lotse ref pruefen <ID>`

Prüft eine Referenz: Ordner und Dateien im Dateisystem, Adressen über das Netz.
Physische Dinge sind `nicht_pruefbar` – da kann kein Programm nachsehen.

---

## Tresor

Zugangsdaten am Vorhaben. Vollständig in **[[Tresor]]**.

### `lotse tresor add <TITEL> [FELDER]…`

Felder als `name=wert`.

| Option | Bedeutung |
|---|---|
| `-p`, `--projekt <PROJEKT>` | Vorhaben, an dem der Eintrag hängt. |
| `--nur-desktop` | Stufe `nur_desktop`: nur mit Desktop-Schlüssel lesbar, nie auf dem Handy. |

```
$ lotse tresor add "Fritzbox Gartenhaus" --projekt Gartenhaus passwort=geheim123 user=admin
Tresor-Eintrag »Fritzbox Gartenhaus« angelegt (ueberall).
```

> Ein Passwort im Klartext in der Kommandozeile landet in der Shell-Historie. Für
> etwas, das dort nicht stehen soll, ist die App der bessere Weg – oder vorher ein
> Leerzeichen, wenn die Shell `HISTCONTROL=ignorespace` kennt.

### `lotse tresor liste`

Titel und Feldnamen, **nie** Werte. Mit `-p`/`--projekt` nur eines Vorhabens.

```
$ lotse tresor liste
ueberall     Fritzbox Gartenhaus            passwort, user
```

### `lotse tresor zeige <TITEL> <FELD>`

Gibt **ein** Feld aus, nichts sonst – damit `… | pbcopy` oder `… | wl-copy` geht.

```
$ lotse tresor zeige "Fritzbox Gartenhaus" user
admin
```

### `lotse tresor loeschen <TITEL>`

Crypto-Shredding: der gewrappte Eintragsschlüssel wird entfernt. Die Werte sind danach
unlesbar, auch in alten Sync-Kopien und alten Sicherungen.

---

## Ordner

### `lotse scan [WURZELN]…`

Durchsucht Ordner nach Vorhaben und legt Funde in der Hafeneinfahrt ab. Ohne Argument
werden die gemerkten Wurzelordner genommen. Welche Marken zählen, steht in
**[[Erkennungsregeln]]**.

| Option | Bedeutung |
|---|---|
| `--uebernehmen` | Legt alle Kandidaten sofort als Vorhaben an, ohne Rückfrage. |

```
$ lotse scan ~/Projekte
2 Kandidaten in der Hafeneinfahrt:
  fotos2024                Haus & Garten          /home/du/Projekte/fotos2024
  meinapp                  Software               /home/du/Projekte/meinapp
Übernehmen mit `lotse uebernehmen <pfad>` oder alle mit `lotse scan --uebernehmen …`.
```

### `lotse uebernehmen <PFAD>`

Macht aus einem Kandidaten ein Vorhaben: Titel aus dem Ordnernamen, Vorlage aus der
Erkennung, der Ordner wird als Referenz hinterlegt und eine Markerdatei
`.lotse-projekt` angelegt, damit der Ordner nach dem Umbenennen wiedererkannt wird.

```
$ lotse uebernehmen ~/Projekte/meinapp
Angelegt: meinapp (Software)
```

### `lotse beobachten [WURZELN]…`

Beobachtet Ordner und schreibt Änderungen verdichtet ins Logbuch. Läuft bis `Strg+C`.
Nur Pfade und Zeitpunkte, **nie** Inhalte – siehe [[Beobachter und Erkennung|Beobachter-und-Erkennung]].

| Option | Standard | Bedeutung |
|---|---|---|
| `--merken` | aus | Speichert die Wurzelordner dauerhaft, damit künftige Aufrufe ohne Argument gehen. |
| `--intervall <SEK>` | `30` | Sekunden zwischen zwei Schreibvorgängen ins Logbuch. |

---

## Gegenseite und Kalender

### `lotse gegenseite [PROJEKT]`

Holt den Stand bei GitHub oder GitLab – offene Pull Requests, Zahl offener Issues,
Zustand der Prüfläufe – und schreibt eine verdichtete Zeile ins Logbuch. Ohne Argument
alle Vorhaben mit erkanntem Repo. Siehe [[Gegenseite]].

| Option | Bedeutung |
|---|---|
| `--trocken` | Nur zeigen, nichts schreiben. |

Den Token liefert `LOTSE_FORGE_TOKEN`, `LOTSE_GITHUB_TOKEN`, `LOTSE_GITLAB_TOKEN` oder
der in der App hinterlegte Tresor-Eintrag.

### `lotse termine [PROJEKT]`

Zeigt anstehende Termine aus abonnierten Kalendern. Lotse schreibt sie nicht ins
Logbuch – sie bleiben, wo sie gepflegt werden. Siehe [[Kalender]].

| Option | Standard | Bedeutung |
|---|---|---|
| `--tage <N>` | `90` | Wie weit nach vorn geschaut wird. |

```
$ lotse termine
Fehler: Kein Projekt hat eine Kalender-Referenz. Die Abonnement-Adresse des Kalenders
als Referenz vom Typ `url` anlegen (endet auf .ics oder beginnt mit webcal://).
```

---

## Abgleich

Vollständig in **[[Abgleich]]**, Protokoll in **[[Sync-Protokoll]]**.

### `lotse sync register --url <URL> --email <ADRESSE>`

Meldet ein **bestehendes** lokales Konto beim Dienst an. Fragt den
Wiederherstellungscode ab, weil daraus der Wiederherstellungsweg gewrappt wird.

### `lotse sync login --url <URL> --email <ADRESSE>`

Meldet ein **neues Gerät** an einem bestehenden Konto an und holt alles herunter.

| Option | Standard |
|---|---|
| `--geraet <NAME>` | `Dieser Rechner` |

### `lotse sync jetzt`

Gleicht ab: erst pushen, dann pullen.

### `lotse sync status`

```
$ lotse sync status
Ausstehende Änderungen: 12
Zuletzt gepusht: lokale Sequenz 0
Zuletzt gepullt: Server-Sequenz 0
Kein Sync eingerichtet (`lotse sync register` oder `lotse sync login`).
```

### `lotse sync geraete`

Alle Geräte des Kontos mit Name, Plattform und letztem Abgleich.

### `lotse sync widerrufen <ID>`

Widerruft ein Gerät; seine Sitzungen verfallen sofort. Für ein verlorenes Notebook.
Die Daten auf dem Gerät bleiben verschlüsselt liegen – Widerruf ist kein Fernlöschen.

---

## Export

Vollständig in **[[Export und Fluchtweg|Export-und-Fluchtweg]]**.

### `lotse export spiegel <ZIEL>`

Schreibt den Bestand als Markdown: je Vorhaben `projekt.md` mit YAML-Kopf und
`logbuch/JJJJ-MM.md` je Monat. **Ohne Tresor.**

```
$ lotse export spiegel ~/lotse-spiegel
3 Projekte nach /home/du/lotse-spiegel gespiegelt (Klartext, ohne Tresor).
```

Der Kopf einer `projekt.md`:

```yaml
---
lotse_schema: 1
id: 01M28KAQ3BYYG0BPNHRHQXB8XR
titel: "Gartenhaus"
status: wartet
vorlage: haus_garten
wiedervorlage: 2026-10-01
erwartungsintervall_tage: 60
tags: ["haus"]
angelegt: 2026-09-11 16:03
zuletzt_beruehrt: 2026-09-11 16:03
referenzen:
  - {typ: ordner, ziel: "/home/du/bau", rolle: material, pruefstatus: nicht_pruefbar}
  - {typ: url, ziel: "https://example.com/statik", rolle: doku, pruefstatus: nicht_pruefbar}
---
```

### `lotse export bundle <ZIEL>`

Der Fluchtweg: alles als JSON, mit `age` und Passphrase verschlüsselt, **mit Tresor**.
Entschlüsselbar mit dem generischen `age`-Werkzeug, ohne Lotse. Die Passphrase kommt
aus der Abfrage oder `LOTSE_EXPORT_PASSPHRASE`.

---

## Assistenten

### `lotse mcp`

Spricht das Model-Context-Protocol über stdin/stdout. Für Claude Code:

```
claude mcp add lotse -- lotse mcp
```

Der Handschlag und die Antwort auf `tools/call`:

```
$ printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}' | lotse mcp
{"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{}},"instructions":"Lotse ist das Logbuch …","protocolVersion":"2025-06-18","serverInfo":{"name":"lotse","version":"0.6.1"}}}
```

Die fünf Werkzeuge und die Absicherung stehen in **[[Assistenten (MCP)|Assistenten-MCP]]**.

---

## Updates

### `lotse update`

Sieht nach, ob es eine neuere Fassung gibt. Lädt nichts herunter und führt nichts aus.

```
$ lotse update
Version 0.6.1 ist die neueste.
```

Eine Kommandozeile, die sich selbst ersetzt, wäre eine Überraschung im falschen
Moment. Die App tauscht sich dagegen selbst aus – siehe [[Updates]].

---

## Exit-Codes

| Code | Bedeutung |
|---|---|
| `0` | Alles gut. |
| `1` | Fehler. Der Text geht nach stderr, brauchbare Ausgaben nach stdout – `lotse tresor zeige … | wl-copy` bleibt damit sauber. |
| `2` | Aufruf falsch (unbekanntes Kommando, fehlendes Argument). Von clap. |

Was die Fehlermeldungen bedeuten: **[[Fehlermeldungen]]**.
