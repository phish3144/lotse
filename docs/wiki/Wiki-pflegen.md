# Wiki pflegen

Dieses Wiki soll bei **jedem** Update mitwachsen. Als Vorsatz ist das wertlos – nach drei
Fassungen stimmt die Hälfte nicht mehr, und dann glaubt niemand mehr die andere Hälfte.

Deshalb ist es kein Vorsatz, sondern eine Prüfung, die rot wird.

---

## Wo der Text steht

In `docs/wiki/` im Hauptrepository, eine Markdown-Datei je Seite. Der Workflow
`.github/workflows/wiki.yml` schreibt sie ins GitHub-Wiki, sobald auf `main` etwas darunter
geändert wird.

**Änderungen hier im Browser werden beim nächsten Lauf überschrieben.** Bearbeite die
Dateien im Repository.

### Warum im Repository und nicht im Wiki

Das Wiki ist ein eigenes Git-Repository (`lotse.wiki.git`). Darin gibt es keine Pull
Requests, keine Prüfung, keine CI – nichts, was auffällt, wenn jemand Unsinn hineinschreibt
oder eine Seite veraltet.

Im Hauptrepository liegt der Text bei der Sache, die er beschreibt. Wer ein Feld umbenennt,
sieht die Wiki-Seite in derselben Änderung.

### Namen und Verweise

Dateiname = Seitenname. Umlaute nicht im Dateinamen, sondern im Verweistext:

```
[[Oberfläche|Oberflaeche]]
[[Für Entwickler|Fuer-Entwickler]]
```

`_Sidebar.md` ist die Navigation, `_Footer.md` die Fußzeile. Beide werden von GitHub auf
jeder Seite angezeigt.

---

## Die Prüfung

```
scripts/wiki_pruefen.sh
```

Läuft ohne Rust-Werkzeugkette, nur mit `python3`, und in CI bei jedem Push.

Sie liest die **tatsächliche** Oberfläche aus dem Quelltext und verlangt, dass jedes Stück
davon im Wiki vorkommt:

| Abgelesen aus | Muss stehen in |
|---|---|
| Kommandobaum aus den clap-Enums in `main.rs` | [[Kommandozeile-Referenz]] |
| Alle `#[tauri::command]`-Funktionen | irgendeiner Seite |
| MCP-Werkzeugnamen aus `mcp/mod.rs` | [[Assistenten (MCP)\|Assistenten-MCP]] |
| Alle `LOTSE_*`-Variablen | [[Umgebungsvariablen]] |
| Alle `META_*`-Einstellungsschlüssel | [[Datenablage]] |
| Reiter aus `Einstellungen.svelte` | [[Einstellungen]] |
| `/v1/…`-Endpunkte aus dem Sync-Worker | [[Sync-Protokoll]] |
| Erkennungsmarken aus `detect.rs` | [[Erkennungsregeln]] |
| Jede Struktur, jedes Feld, jeder Aufzählungswert aus `model.rs` | [[Datenmodell]] |
| Jeder Fehlertext aus `error.rs` | [[Fehlermeldungen]] |
| Erwartungsintervalle je Vorlage | [[Vorhaben]] |
| Die laufende Versionsnummer | [[Änderungen\|Aenderungen]] |
| Jeder Seitenverweis | muss auf eine vorhandene Seite zeigen |

Ausgabe im guten Fall:

```
$ scripts/wiki_pruefen.sh
39 Wiki-Seiten, 420 Prüfungen.
Das Wiki deckt die gesamte abgelesene Oberfläche ab.
```

Im schlechten:

```
  ✗ Datenmodell: Projekt-Feld »prioritaet« ist nirgends beschrieben.
  ✗ Umgebungsvariablen: Variable »LOTSE_PROXY« ist nirgends beschrieben.

2 Lücke(n). Das Wiki hinkt der Anwendung nach –
ergänze die genannten Stellen in docs/wiki/ und lauf die Prüfung erneut.
```

### Was sie kann und was nicht

Sie prüft, dass die Zeichenkette **irgendwo auf der zuständigen Seite steht**. Absichtlich
stumpf: ob der Satz drumherum richtig ist, kann keine Maschine wissen.

Was sie kann, ist das **Vergessen unmöglich machen**. Wer ein Feld hinzufügt und das Wiki
nicht anfasst, bekommt eine rote CI – nicht ein halbes Jahr später eine Frage im Postfach.

Sie ersetzt also nicht das Nachdenken. Sie stellt nur sicher, dass es stattfindet.

---

## Beim Ändern der Anwendung

Zur Liste in `CLAUDE.md` (*Prüfen vor jedem Commit*) gehört:

```
scripts/wiki_pruefen.sh
```

Meldet sie eine Lücke, ist das die Arbeitsliste. Zwei Regeln dabei:

**Nichts behaupten, was nicht läuft.** Dieselbe Regel wie für `site/index.html`. Ein Wiki,
das ein geplantes Feature beschreibt, ist schlimmer als eines, das es weglässt – der Leser
sucht dann nach einem Knopf, den es nicht gibt.

**Fehler benennen, nicht verschweigen.** Wenn eine Fassung etwas nicht kann, was sie
können sollte, steht das in [[Änderungen|Aenderungen]] und auf der betroffenen Seite. Die
Anmerkung zu 0.6.0 ist das Muster: sie kostet zwei Sätze und erspart dem Leser eine
Stunde Suche.

---

## Bei einer neuen Fassung

1. In [[Änderungen|Aenderungen]] einen Abschnitt anlegen: **was sich geändert hat** und
   **was das für den Leser bedeutet**. Der zweite Teil fehlt in den meisten Änderungstexten
   und ist der, der gelesen wird.
2. Die alte Fassung aus dem Abschnitt »laufende Fassung« nach unten schieben.
3. Auf betroffenen Seiten die Fassungsangaben nachziehen (»ab 0.6.1 …«, »bis 0.5.0 war
   …«).
4. `scripts/wiki_pruefen.sh` – die Versionsnummer wird geprüft und schlägt an, wenn Schritt
   1 fehlt.
5. Feature-Stand-Badges in `site/index.html` mitziehen, wie es `CLAUDE.md` verlangt.

---

## Beim Anlegen einer neuen Seite

1. Datei in `docs/wiki/` anlegen, Name ohne Umlaute.
2. In `_Sidebar.md` eintragen – sonst findet sie niemand.
3. Von mindestens einer bestehenden Seite darauf verweisen. Eine Seite, auf die nichts
   zeigt, ist eine Seite, die nicht existiert.
4. Prüfung laufen lassen; sie findet Verweise auf nicht vorhandene Seiten.

---

## Wenn der Workflow nicht schreibt

Das Wiki-Repository entsteht erst, wenn im Tab *Wiki* **einmal von Hand** eine Seite
angelegt wurde. Vorher gibt es `lotse.wiki.git` nicht, und der Workflow bricht mit genau
dieser Anleitung ab.

Danach ersetzt er den Inhalt vollständig: alles außer `.git` wird entfernt und aus
`docs/wiki/` neu geschrieben. So verschwinden auch gelöschte Seiten – ein Wiki mit
Geisterseiten, die aus der Navigation verschwunden, aber noch erreichbar sind, wäre
schlimmer als keines.

Von Hand anstoßen: Actions → *Wiki* → *Run workflow*.

---

## Der Ton

Zwei Dinge, die dieses Wiki von der Mehrzahl unterscheiden sollen:

**Begründen, nicht nur beschreiben.** »Pausieren verlangt eine Notiz« ist eine Regel.
»Weil ein Vorhaben, das ohne Grund ruht, in drei Monaten genau die Frage erzeugt, die
Lotse beantworten soll« ist der Grund – und nur den akzeptiert man.

**Die Grenzen nennen.** Jede Seite sagt auch, was **nicht** geht. Wer das erst nach zwei
Stunden Suche erfährt, ist zu Recht verärgert. Die fünf Metaphern – Hafen, Kurs, Logbuch,
vor Anker, Hafeneinfahrt – bleiben die einzigen; jede weitere ist eine, die der Leser
lernen muss.
