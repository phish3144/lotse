# Erkennungsregeln

Wie Lotse entscheidet, ob ein Ordner ein Vorhaben ist – und welche Art. Die Regeln
stehen vollständig hier, weil ein Vorschlag, den man nicht nachvollziehen kann, kein
Vorschlag ist, sondern ein Orakel.

Was die Erkennung praktisch bedeutet, steht in **[[Beobachter und Erkennung|Beobachter-und-Erkennung]]**.

---

## Was gelesen wird

Nur **Verzeichniseinträge und Dateinamen**. Keine Inhalte – mit einer Ausnahme: die
ersten Zeilen einer `README`, als Vorschlag für den Kurs. Das ist keine Höflichkeit,
sondern eine Modulgrenze: `detect` darf den Tresor nicht anfassen und tut es
nachweisbar nicht (`scripts/modulgrenzen.sh`).

Gezählt wird pro Ordner, nicht rekursiv. Ein `cargo.toml` zwei Ebenen tiefer macht den
oberen Ordner nicht zum Software-Vorhaben.

## Wie tief

Vier Ebenen unter jedem Wurzelordner. Ein erkannter Ordner wird **nicht** weiter
betreten: Unterordner eines Vorhabens sind keine eigenen Vorhaben. Symbolische
Verknüpfungen werden nicht verfolgt.

## Wo nie hineingesehen wird

Ordner, die mit `.` beginnen, plus diese Liste:

```
node_modules   target        .git          build         dist
out            .venv         venv          __pycache__   .cache
.next          .idea         .vscode       .gradle       Library
AppData        .Trash        Pods          DerivedData
```

Zwei Gründe: Bauartefakte und Abhängigkeiten sind keine Vorhaben, und `Library`,
`AppData` sowie `.Trash` würden einen Scan über das Benutzerverzeichnis sonst Minuten
kosten.

## Was nie gelesen wird

Diese Dateien liest Lotse an keiner Stelle – nicht bei der Erkennung, nicht beim
Beobachten, nicht beim Suchindex, und auch nicht, wenn du sie ausdrücklich zum Deuten
hergibst ([[Datei deuten|Datei-deuten]]):

```
.env    *.pem    *.key    id_rsa*    id_ed25519*    *.kdbx    *.p12
```

Eine Stelle im Code, alle Aufrufer. Der Beobachter kann sie also nicht umgehen, weil er
dieselbe Liste benutzt.

---

## Die Regeln, in dieser Reihenfolge

Die erste passende Regel gewinnt. Spezifische vor allgemeinen – sonst wäre jedes Repo
»Software« und ein KiCad-Projekt mit Git nie »Hardware«.

### 1 · Hardware & Maker

Trifft, wenn **eines** davon im Ordner liegt:

| Marke | Was es ist |
|---|---|
| `platformio.ini` | PlatformIO-Projekt (Mikrocontroller) |
| `.kicad_pro` | KiCad-Leiterplatte |
| `.ino` | Arduino-Sketch |
| `.scad` | OpenSCAD-Modell |
| `.f3d` | Fusion 360 |
| `.3mf` | 3D-Druck-Projekt |
| `.stl` | ab **3** Dateien – eine einzelne STL ist ein Download, drei sind ein Vorhaben |

### 2 · Software

| Marke |
|---|
| `cargo.toml`, `package.json`, `pyproject.toml`, `go.mod` |
| `.sln`, `cmakelists.txt`, `pom.xml`, `build.gradle`, `mix.exs`, `gemfile` |

### 3 · Lernen & Forschung

`.ipynb` (Jupyter), `.bib` (Literaturverzeichnis) oder `.tex` (LaTeX).

Steht vor »Software«? Nein – **nach**. Ein Repo mit `pyproject.toml` *und* Notebooks ist
Software; ein Ordner mit nur Notebooks ist Forschung.

### 4 · Ein Git-Repo ohne weitere Marke

Liegt ein `.git` daneben und hat keine Regel davor getroffen: Software. Die Marke heißt
dann `.git`, damit in der Hafeneinfahrt zu sehen ist, worauf der Vorschlag beruht.

### 5 · Kreativ

| Marke | Programm |
|---|---|
| `.als` | Ableton Live |
| `.logicx` | Logic Pro |
| `.flp` | FL Studio |
| `.rpp` | Reaper |
| `.scriv` | Scrivener |
| `.psd` | Photoshop |
| `.afphoto` | Affinity Photo |
| **5 oder mehr RAW-Fotos** | `.cr2`, `.cr3`, `.nef`, `.arw`, `.dng`, `.raf`, `.orf` |

### 6 · Finanzen & Verwaltung

**Drei oder mehr** PDF-Dateien, deren Name eine Jahreszahl enthält – vier Ziffern, die
mit `19` oder `20` beginnen. Das trifft `rechnung-2024.pdf` und
`2026-01 Nebenkosten.pdf`, aber nicht `anleitung.pdf`.

Warum die Jahreszahl? Ein Ordner voller PDFs kann eine Sammlung Anleitungen sein.
Datierte PDFs sind fast immer Verwaltung.

### 7 · Haus & Garten

**Acht oder mehr** Bilder (`.jpg`, `.jpeg`, `.png`, `.heic`, `.webp`) **und**
mindestens eine `.pdf`. Das Muster eines Bauvorhabens: Fotos vom Fortschritt plus
Angebot, Rechnung oder Genehmigung.

### 8 · Allgemein

Eine Datei, deren Name mit `readme` beginnt (in jeder Schreibweise). Marke: `readme`.

### Sonst

Kein Kandidat. Der Ordner wird ignoriert und nicht gemerkt – ein späterer Scan sieht
erneut hin.

---

## Die Markerdatei

Beim Übernehmen legt Lotse im Ordner eine Datei `.lotse-projekt` an. Darin steht die ID
des Vorhabens.

Sie erfüllt genau eine Aufgabe: den Ordner nach dem **Umbenennen oder Verschieben**
wiederzuerkennen. Ohne sie wäre `~/Projekte/gartenhaus` nach dem Umzug nach
`~/Archiv/gartenhaus-2026` ein neuer Kandidat, und du hättest dasselbe Vorhaben zweimal.

Ein Ordner mit Markerdatei erscheint in der Hafeneinfahrt mit gesetztem `bekannte_id`
und wird nicht erneut zum Übernehmen angeboten.

Die Datei darf gelöscht werden. Dann verliert Lotse die Verknüpfung, das Vorhaben
bleibt. Sie enthält keine Geheimnisse und darf in Git.

---

## Selbst nachsehen

```
lotse scan ~/Projekte
```

zeigt, was erkannt würde, ohne etwas anzulegen. Die Spalte hinter dem Namen ist die
vorgeschlagene Vorlage; die gefundenen Marken stehen in der App am Kandidaten.

Stimmt ein Vorschlag nicht: übernimm ihn trotzdem und ändere Vorlage und
Erwartungsintervall danach. Beides ist frei änderbar, die Vorlage wirkt nur beim
Anlegen ([[Datenmodell]]).
