# Beobachter und Erkennung

Lotse kann Ordner durchsuchen und im Hintergrund mitschreiben. Beides ist optional und
liest **keine Dateiinhalte**.

---

## Erkennung: was zählt als Vorhaben

Lotse steigt bis zu **vier Ebenen** tief in die angegebenen Wurzelordner und sucht nach
Marken. Ein erkannter Ordner wird nicht weiter durchsucht – Unterordner eines Vorhabens
sind keine eigenen Vorhaben.

Übersprungen werden Ordner, die mit `.` beginnen, und diese:

`node_modules` · `target` · `.git` · `build` · `dist` · `out` · `.venv` · `venv` ·
`__pycache__` · `.cache` · `.next` · `.idea` · `.vscode` · `.gradle`

### Marken und Vorlagen

Die Reihenfolge ist die Priorität – spezifische Marken vor allgemeinen.

| Vorlage | Marken |
|---|---|
| **Hardware & Maker** | `platformio.ini`, `*.kicad_pro`, `*.ino`, `*.scad`, `*.f3d`, `*.3mf`, ab 3 × `*.stl` |
| **Software** | `Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, `*.sln`, `CMakeLists.txt`, `pom.xml`, `build.gradle`, `mix.exs`, `Gemfile` |
| **Lernen & Forschung** | `*.ipynb`, `*.bib`, `*.tex` |
| **Software** (schwach) | nur `.git` |
| **Kreativ** | `*.als`, `*.logicx`, `*.flp`, `*.rpp`, `*.scriv`, `*.psd`, `*.afphoto`, ab 5 RAW-Bildern |
| **Finanzen & Verwaltung** | ab 3 PDFs mit Jahreszahl im Namen |
| **Haus & Garten** | ab 8 Bildern **und** mindestens 1 PDF |
| **Generisch** | eine Datei, die mit `readme` beginnt |

Findet sich nichts davon, ist der Ordner kein Kandidat. Das ist Absicht: lieber zu wenig
vorschlagen als eine Hafeneinfahrt voller Rauschen.

### Übernehmen

Kandidaten stehen im Hafen rechts und warten. Beim Übernehmen legt Lotse an:

- das Vorhaben mit der erkannten Vorlage,
- einen **offenen Faden** „Kurs festlegen“ (mit Vorschlag aus der README, falls vorhanden),
- eine **Referenz** auf den Ordner,
- bei `.git`: die letzten Commits, verdichtet auf eine Zeile pro Tag,
- eine Markierungsdatei `.lotse-projekt` im Ordner mit der Projekt-ID.

Die Markierungsdatei ist der Grund, warum `lotse log` aus diesem Ordner heraus weiß,
wohin der Eintrag gehört – und warum ein umbenannter Ordner nicht die Zuordnung verliert.

```bash
lotse scan ~/code ~/Bau
lotse uebernehmen ~/code/lampe --titel "Lampe ESP32"
```

---

## Beobachter: laufend mitschreiben

Einstellungen → *Ordner* → *Laufend beobachten*. Oder:

```bash
lotse beobachten ~/code      # läuft bis Strg+C
```

### Was er schreibt

Eine **verdichtete Zeile pro Vorhaben und Tag**, nicht eine pro Ereignis:

```
Geändert: statik-nachweis.pdf, fundament-v3.dwg
3 Commits: Fundamentplan korrigiert · Maße nachgetragen · Materialliste ergänzt
```

Bei mehr als acht Commits an einem Tag kürzt Lotse auf die ersten acht plus „… und N
weitere“.

### Was er nicht liest

**Keine Dateiinhalte.** Nur Namen, Zeitpunkte und – bei Git – Commit-Betreffzeilen aus
`git log`.

Auf einer festen Ausschlussliste stehen außerdem, so dass sie nicht einmal im
Logbuch auftauchen:

`.env` · `*.pem` · `*.key` · `id_rsa*` · `id_ed25519*` · `*.kdbx` · `*.p12`

Dieselbe Liste gilt auch dann, wenn du eine solche Datei unter *Datei deuten*
ausdrücklich auswählst.

### Wie er läuft

Der Beobachter läuft in einem eigenen Thread und greift nur zum Schreiben kurz auf den
Speicher zu – die Oberfläche steht dabei nicht an. Er endet, sobald Lotse sperrt:
gesperrt heißt gesperrt.

Ist **[[Verbindungen|Verbindungen]]** eingeschaltet, nimmt er die Gegenseite in großem
Abstand mit (höchstens alle 30 Minuten).

### Wurzelordner wählen

Weniger ist mehr. `~/code` und `~/Bau` sind gute Wurzeln. `~` ist eine schlechte: vier
Ebenen tief unter dem Heimatverzeichnis ist viel, und der Beobachter hätte dann sehr
viel zu ignorieren.

---

## Datei deuten

Auf der Projektseite rechts: eine **einzelne, ausdrücklich gewählte** Datei aufmachen
und ihren Text auslesen. PDF, Text, Markdown, CSV oder JSON, bis 20 MB.

Der Auszug wird **angezeigt, nicht gespeichert** und nicht gesendet. Erst wenn du ihn an
die **[[KI]]** weitergibst, verlässt er den Rechner – und auch dann siehst du vorher
genau, was gesendet würde.

Der Unterschied zum Beobachter ist der Punkt: der Beobachter liest nie Inhalte; *Datei
deuten* ist der eine Fall, in dem ein Mensch eine Datei ausdrücklich hergibt.

PDFs werden in einem abgesicherten Bereich geparst – eine beschädigte Datei von außen
ergibt einen Fehler, keinen Absturz.
