# Lotse – Konzept

Stand: 2026-09-06. Dieses Dokument ist die verbindliche Beschreibung dessen, was Lotse ist,
wie es aufgebaut ist und in welcher Reihenfolge es entsteht. Änderungen an den Grundsatz-
entscheidungen (Abschnitt 12) werden hier protokolliert, nicht stillschweigend im Code.

## 1. Leitidee

Lotse ist das Logbuch für alle Vorhaben einer Person: Software, Werkstatt, Haus, Musik,
Steuer, Verein, Reise. Für jedes Projekt beantwortet Lotse an derselben Stelle:

*Worum geht es, wo stehe ich, was war zuletzt, was ist der nächste Schritt, wo liegt das
Material, wie komme ich rein?*

Lotse ist eine **dünne Kontext- und Referenzschicht** über Dingen, die woanders liegen.
Der Code bleibt im Repo, die Fotos im Ordner, die Schrauben in der Kiste im Keller.
Lotse zieht keine Inhalte nach innen, es zeigt zuverlässig darauf und hält fest, was man
beim letzten Mal im Kopf hatte.

Lotse ist ausdrücklich **nicht**:

- ein Notion- oder Obsidian-Nachbau (keine freie Block-Engine, kein Wiki, kein Backlink-Graph),
- ein Jira oder Linear (kein Sprint-, Ticket- oder Backlog-Vokabular),
- ein Dokumenten-Silo,
- ein Ersatz für einen Passwortmanager mit Browser-Autofill (Proton Pass bleibt zuständig
  für Web-Logins; Lotse hält projektgebundene Geheimnisse).

Die vier Kernprobleme, an denen jedes Feature gemessen wird:

1. **Wiedereinstieg** – nach Wochen oder Jahren wieder in ein Projekt hineinkommen.
2. **Faden halten** – zwischen und innerhalb von Projekten nicht den Überblick verlieren.
3. **Rad nicht neu erfinden** – Wissen, Bausteine und Kontakte projektübergreifend wiederfinden.
4. **Geheimnisse sicher aufbewahren** – Zugangsdaten und sensible Notizen verschlüsselt, aber
   griffbereit, am Projekt.

Die Lotsen-Metapher wird sparsam eingesetzt, genau fünf Begriffe:
**Hafen** (Startseite), **Kurs** (Zielsatz eines Projekts), **Logbuch** (chronologischer
Verlauf), **vor Anker** (bewusst pausiert), **Hafeneinfahrt** (erkannte, noch nicht
bestätigte Projekte). Alles andere heißt, wie es heißt.

## 2. Rollen der Komponenten

| Komponente | Rolle | Läuft wo |
|---|---|---|
| **Desktop-App** | Das mächtige Werkzeug. Beobachtet Ordner, erkennt Projekte, hält den Tresor, synchronisiert, exportiert. | Windows, macOS, Linux (Tauri 2, Rust-Kern) |
| **Web-App** | Vollwertiger Client für unterwegs und fremde Rechner. Lesen, schreiben, Tresor-Stufe "überall". Kein Ordner-Beobachter. | Browser (Svelte + Rust-Kern als WebAssembly), Cloudflare Pages |
| **Sync-Dienst** | Speichert ausschließlich Ciphertext und Metadaten, verteilt Änderungen zwischen Geräten. | Cloudflare Worker + D1 + R2 (Gratisplan) |
| **CLI** | Schnellerfassung aus dem Terminal, Projekt-Erkennung aus dem Arbeitsverzeichnis. | überall, wo die Desktop-App läuft |
| **MCP-Server** | Brücke zu KI-Assistenten (Claude Code u. a.): Projektkontext lesen, Logbuch schreiben. Kein Tresor-Zugriff. | lokal, Teil der Desktop-App |

Alle Geräte sind gleichberechtigte Clients derselben Sync-Engine. Es gibt keinen "Haupt-
rechner". Die Wahrheit ist die lokale Datenbank jedes Geräts plus das Änderungsprotokoll;
der Sync-Dienst ist Verteiler und Backup, nie Quelle im Klartext.

## 3. Datenmodell

### Grundsatz

Nicht "welcher Typ Projekt ist das", sondern "was braucht jedes Projekt". Das Kernmodell
ist für ein Gartenbeet identisch mit dem für einen Compiler. Domänenspezifisches kommt
additiv über Vorlagen und später Facetten, nie über Vererbung oder ein Schema pro Projektart.

### Entitäten (Version 1)

**Projekt** (Wurzel)

| Feld | Bedeutung |
|---|---|
| `id` | ULID |
| `titel` | Klartextname |
| `kurs` | 1–2 Sätze in eigenen Worten: worum geht es, was ist das Ziel |
| `status` | `idee` · `aktiv` · `pausiert` · `wartet` · `abgeschlossen` · `eingemottet` |
| `wiedervorlage` | optionales Datum, für `pausiert` und `wartet` |
| `erwartungsintervall_tage` | ab wann Stille auffällig ist; Garten 90, Software 14; Default aus der Vorlage |
| `tags` | freie Liste |
| `vorlage` | Kennung der Start-Vorlage (nur informativ) |
| `abgeleitet_von` | optionale Projekt-ID |
| `angelegt`, `zuletzt_beruehrt` | Zeitstempel |

`status` ist ein festes, kleines, nicht anklagendes Vokabular. `eingemottet` heißt
bewusst beendet, nicht gescheitert. `pausiert` und `wartet` tauchen in keiner
Rückstands-Statistik auf. Jeder Statuswechsel ist ein Logbuch-Eintrag mit Typ `status`.

**Notiz** (Logbuch-Eintrag – die wichtigste Entität)

| Feld | Bedeutung |
|---|---|
| `id`, `projekt_id`, `ts` | |
| `quelle` | `mensch` · `cli` · `datei` · `git` · `import` · `mcp` · `ki` |
| `art` | `log` · `offen` · `entscheidung` · `status` · `uebergabe` |
| `text` | Markdown |
| `erledigt_am` | nur für `offen` |

Faden, Logbuch-Eintrag, Entscheidung und Übergabenotiz sind bewusst **eine** Entität mit
einem Art-Feld. Eine Eingabezeile, keine Typ-Frage. Notizen werden nie überschrieben;
Korrekturen sind neue Notizen. Eine Entscheidung ist eine Notiz mit vorausgefüllter
Textvorlage (Kontext / Entschieden / Verworfen weil / Neu bewerten wenn), ohne Pflicht-
Unterfelder.

**Referenz** (zeigt nach außen)

| Feld | Bedeutung |
|---|---|
| `id`, `projekt_id` | |
| `typ` | `ordner` · `git_repo` · `url` · `datei` · `physisch` · `geraet` · `passwortmanager` |
| `ziel` | Pfad, URL oder Ortsbeschreibung ("Keller, Regal 3, blaue Kiste") |
| `rolle` | `material` · `ergebnis` · `doku` |
| `geraet_id` | für Pfade: auf welchem Gerät der Pfad gilt |
| `zuletzt_geprueft`, `pruefstatus` | `ok` · `nicht_erreichbar` · `nicht_pruefbar` |

`nicht_pruefbar` ist ein ehrlicher Zustand, kein Fehler. Prüfung erfolgt auf Klick oder
beim Öffnen, nicht per Hintergrundjob.

**Tresor-Eintrag**

| Feld | Bedeutung |
|---|---|
| `id`, `titel`, `projekt_ids` | Titel und Zuordnung im Klartext, damit Suche und Cockpit funktionieren |
| `stufe` | `ueberall` · `nur_desktop` |
| `felder` | Liste aus Name + verschlüsseltem Wert |

**Gerät**

`id`, `name`, `plattform`, `angelegt`, `zuletzt_sync`. Nötig für gerätegebundene Pfade
und für die Geräteverwaltung (widerrufen).

### Später (mit Auslöser, siehe Roadmap)

- **Baustein** – projektunabhängiges Wissen (`snippet` · `checkliste` · `verfahren` ·
  `bezugsquelle` · `kontakt`), per Referenz eingebunden, mit automatischer Herkunfts- und
  Verwendungsanzeige.
- **Facette** – strukturierte Domänenfelder (Stückliste, Kostenaufstellung) als versionierte
  Config, kein Laufzeit-Schema-Editor.
- **Projekt-Kanten** – `teilprojekt_von`, `verwandt_mit`.

### Vorlagen

Acht Start-Vorlagen, die nur Standard-Tags, Erwartungsintervall, typische Referenzarten und
eine Start-Checkliste setzen. Das Datenmodell bleibt für alle gleich.

| Vorlage | Erwartungsintervall | Erkennungsmarken (Beobachter) |
|---|---|---|
| Software | 14 Tage | `.git`, `Cargo.toml`, `package.json`, `pyproject.toml`, `go.mod`, `*.sln` |
| Hardware & Maker | 30 Tage | `platformio.ini`, `*.kicad_pro`, `*.ino`, `*.stl`-Sammlung, `*.f3d`, `*.scad` |
| Haus & Garten | 60 Tage | Ordner mit Fotos + PDFs, keine Code-Marken |
| Kreativ (Musik, Foto, Text) | 30 Tage | `*.als`, `*.logicx`, `*.flp`, RAW-Sammlungen, `*.scriv`, `*.md`-Sammlung |
| Finanzen & Verwaltung | 90 Tage | PDF-Sammlungen mit Jahreszahlen im Namen |
| Lernen & Forschung | 30 Tage | `*.ipynb`, `*.bib`, `*.tex` |
| Reise & Veranstaltung | 120 Tage | keine |
| Generisch | 30 Tage | `README*` ohne andere Marke |

## 4. Kern-Features der ersten Version

**Der Hafen (Startseite).** Drei Gruppen: *Auf See* (aktiv, sortiert nach Auffälligkeit),
*Vor Anker* (pausiert/wartet, eingeklappt), *Ideen*. Jede Karte zeigt Kurs-Einzeiler,
letzten Logbuch-Satz und dessen Alter. Rot wird nur, wer sein eigenes Erwartungsintervall
reißt oder dessen Wiedervorlage verstrichen ist. Oben maximal drei Einträge "Heute wichtig".
Dazu die **Hafeneinfahrt**: vom Beobachter erkannte Projektkandidaten, die noch nicht
bestätigt sind.

**Die Projektseite mit dem "Wo war ich"-Brief.** Beim Öffnen eines Projekts, das länger als
sein Erwartungsintervall ruhte, steht oben eine Karte: Zeit seit letztem Kontakt, letzte
Übergabenotiz im Wortlaut, offene Fäden, Aktivität seit dem letzten Besuch (Dateien,
Commits), fällige Referenzprüfungen. Darunter Kurs, Status, offene Fäden, Referenzen,
Zugänge, Logbuch. Eine Seite, für alle Projektarten identisch aufgebaut.

**Schnellerfassung.** Ein Feld, Enter. In-App-Tastenkürzel und `lotse log "…"` im Terminal.
`@projekt` ordnet zu, ohne erkanntes Projekt landet der Gedanke im Postkorb. Kein Dialog
vor dem Speichern.

**Pausieren mit Übergabe.** Statuswechsel auf *pausiert* oder *wartet* verlangt eine
Übergabenotiz, vorbefüllt aus offenen Fäden und der Aktivität seit dem letzten Besuch, mit
Ein-Klick-Default ("nichts Neues, siehe letzte Notiz"). Pflicht, aber billig. Der Trigger
ist ein seltenes, ohnehin bewusstes Ereignis, deshalb überlebt er die dritte Woche.

**Sicherheitsnetz.** Wurde ein Projekt aktiv gelassen und lange nicht angefasst, fragt Lotse
beim nächsten Öffnen einmalig und wegklickbar: "Kurz nachtragen, wo du warst?"

**Ordner-Beobachter** (nur Desktop). Registrierte Wurzelordner werden live beobachtet
(`notify`) und beim Start nachgescannt. Gesammelt werden **Metadaten**, keine Inhalte:
geänderte und neue Dateien, Commits. Daraus entstehen pro Tag verdichtete Notizen mit
Quelle `datei` ("14 Dateien geändert in src/, zuletzt motor.rs"). Feste Ausschlussliste
(`node_modules`, `target`, `.git`, `build`, `dist`, `.venv`, `__pycache__`, `*.tmp`).
Inhalte indexieren (Markdown, Text) ist Opt-in pro Projekt. Dateien wie `.env`, `*.pem`,
`id_rsa*` werden nie inhaltlich gelesen.

**Automatische Projekterkennung.** Ein Ordner unter einer Wurzel wird Kandidat, wenn er
eine Erkennungsmarke trägt (siehe Vorlagen-Tabelle). Kandidaten erscheinen in der
Hafeneinfahrt mit vorgeschlagener Vorlage. Ein Klick bestätigt: Lotse legt das Projekt an,
importiert die Git-Historie als rückdatierte Notizen (verdichtet) und schlägt die README
als Kurs **vor**, übernimmt sie nie automatisch. Lotse schreibt eine Marker-Datei
`.lotse-projekt` mit der Projekt-ID in den Ordner, um ihn nach Umbenennen oder Verschieben
wiederzuerkennen. Mehrere Ordner können zu einem Projekt gehören. Der Massen-Import beim
ersten Start ist derselbe Mechanismus: alle Kandidaten auf einmal, Auswahl per Checkbox.

**Tresor.** Pro Projekt eine Liste verschlüsselter Einträge mit zwei Stufen. Werte
verdeckt bis Klick, Kopieren mit Auto-Löschung der Zwischenablage nach 20 s. Details in
Abschnitt 7 und in `THREAT_MODEL.md`.

**Suche.** Ein Feld über Projekte, Logbücher, Referenz-Titel und Tresor-Titel (SQLite FTS5
im verschlüsselten lokalen Index). Keine Embeddings in v1.

**Offene Punkte.** Projektübergreifende Liste aller offenen Fäden. Die einzige
Aufgabenliste, die Lotse braucht.

**Klartext-Spiegel** (Desktop, Opt-in). Lotse schreibt laufend einen Ordner mit
`projekt.md` (YAML-Frontmatter + Kurs) und `logbuch/JJJJ-MM.md` pro Projekt. Für `grep`,
Obsidian, Notfall. Der Spiegel ist abgeleitet, nicht Wahrheit; Änderungen darin fließen
in v1 nicht zurück.

## 5. Wiedereinstieg und Faden halten – der Mechanismus

Der Brief speist sich aus vier Quellen, in dieser Reihenfolge der Verlässlichkeit:

1. **Die Übergabenotiz** beim Pausieren – der einzige erzwungene Moment.
2. **Passive Signale** – letzte Notiz, letzter offener Faden, Beobachter-Verdichtung,
   `git log --since=<letzter Besuch>` beim Öffnen.
3. **Der passive Nachtrag** – wegklickbare Frage nach langer Stille.
4. **Optional KI-Verdichtung** – wenn seit dem letzten Besuch sehr viel Aktivität anliegt,
   fasst ein Modell auf fünf Zeilen zusammen. Das Rohlog bleibt die Wahrheit, die
   Zusammenfassung ist eine markierte Schicht darüber. Siehe Abschnitt 9.

Bewusst **kein** Ritual bei jedem Sitzungsende, kein Wochenrückblick-Zwang, kein globaler
OS-Hotkey. Alles davon ist wiederkehrende Pflicht und stirbt nach drei Wochen.

## 6. Wiederverwendung – der Mechanismus

In v1 bewusst minimal:

- **Suche über alles.** Der häufigste Fall ist "wie hab ich das damals gemacht?".
- **Neues Projekt aus bestehendem.** Kopiert Kurs-Gerüst, Tags, Referenzstruktur und offene
  Checkpunkte, setzt `abgeleitet_von`.
- **Ein Abschluss-Moment.** Beim Setzen auf *abgeschlossen* ein optionales Feld:
  "Eine Sache fürs nächste Mal?" Enter überspringt.

**Auslöser für die Bausteine-Bibliothek:** derselbe Inhalt wurde zum dritten Mal von Hand
aus einem alten Projekt kopiert. Dann wird daraus eine referenzierbare Entität mit
Herkunfts- und Verwendungsanzeige, Vorschlägen beim Anlegen (Tag-Überlappung, reines SQL)
und Fork statt Versionierung.

## 7. Sicherheit und Verschlüsselung – Überblick

Vollständig in `THREAT_MODEL.md`. Die Kurzfassung:

- **Ende-zu-Ende.** Der Sync-Dienst sieht nur Ciphertext, IDs, Zeitstempel und Größen.
- **Schlüsselhierarchie.** Master-Passwort → Argon2id → HKDF → Auth-Schlüssel (geht zum
  Server) und Wrap-Schlüssel (bleibt lokal). Ein zufälliger **Account-Schlüssel** verschlüsselt
  alle Daten; er ist mit dem Wrap-Schlüssel und mit einem **Wiederherstellungscode** gewrappt.
  Passwortwechsel = neu wrappen, keine Neuverschlüsselung.
- **Zwei Tresor-Stufen.** `ueberall`-Einträge brauchen den Account-Schlüssel. `nur_desktop`-
  Einträge brauchen zusätzlich den **Desktop-Schlüssel**: zufällig, einmal angezeigt, nur
  im OS-Schlüsselbund der Desktops, nie auf dem Server, nie im Browser. Ein fremder Rechner
  mit Keylogger bekommt damit selbst mit Master-Passwort keine `nur_desktop`-Einträge.
- **Lokal verschlüsselt.** Die lokale Datenbank ist SQLCipher; ein gestohlener Laptop gibt
  nichts preis. Der Klartext-Spiegel ist Opt-in und ausdrücklich unverschlüsselt.
- **Kryptografie nur aus geprüften Bausteinen** (RustCrypto: `argon2`, `hkdf`,
  `chacha20poly1305`; `zeroize`). Keine eigenen Primitive, kein eigenes Schema jenseits
  der Komposition. Formatversion in jedem Ciphertext-Header.
- **Fluchtweg.** Export des gesamten Bestands als `age`-Datei plus dokumentiertes Format.
- **Web-Client.** Strikte CSP, keine Fremdskripte, Cloudflare Access vor der App-Adresse,
  Modus "fremder Rechner" ohne Persistenz. Restrisiko: kompromittierte Auslieferung des
  Web-Codes; dokumentiert, mit `nur_desktop` als Antwort für das Wertvollste.
- **Tresor-Isolation.** Beobachter, MCP-Server, KI-Funktionen und Export der Leseansicht haben
  keinen Codepfad zum Tresor. Modulgrenze, nicht Konvention.
- **Signierte Updates** (Tauri-Updater, minisign), `cargo-deny` und `cargo-audit` in CI.

## 8. Sync-Architektur

Vollständig in `SYNC_PROTOCOL.md`. Die Kurzfassung:

- Jedes Gerät hat eine lokale SQLCipher-Datenbank als Arbeitskopie plus ein
  Änderungsprotokoll. Jeder Datensatz trägt ULID, Hybrid-Logical-Clock-Zeitstempel und
  Geräte-ID.
- **Push:** eigene Änderungen seit dem letzten Stand als verschlüsselte Datensätze
  hochladen. **Pull:** fremde Änderungen seit Server-Sequenz X holen.
- **Konflikte:** Notizen sind append-only und kollidieren nie. Für alle anderen Datensätze
  gilt in v1 Last-Writer-Wins auf Datensatzebene nach HLC. Feldweises Mergen kommt erst bei
  nachgewiesenem Bedarf.
- **Löschen** sind Tombstones (Sync braucht sie). Tresor-Einträge werden zusätzlich per
  Crypto-Shredding unlesbar gemacht.
- **Anhänge** werden clientseitig verschlüsselt und als Blobs in R2 abgelegt; über einer
  Größenschwelle (Default 25 MB) nur referenziert, nicht kopiert.
- **Server:** Cloudflare Worker (TypeScript, ~300 Zeilen) + D1 + R2. Jeder Datensatz hat
  eine `account_id`; der Dienst ist vom ersten Tag mandantenfähig. Dieselbe API kann als
  Rust-Binary auf einem eigenen Server laufen (Roadmap), das Protokoll gehört uns.
- **Batching:** Der Client bündelt Änderungen (Debounce 2 s, max. 500 Datensätze pro
  Request), damit die D1-Tageslimits des Gratisplans nie relevant werden.

## 9. KI-Integration

Gezielt, in dieser Reihenfolge, immer ohne Tresor-Zugriff und ohne stillen Datenabfluss:

1. **MCP-Server** (lokal, Teil der Desktop-App): `list_projects`, `get_project_context`,
   `log_activity`, `search`, `list_open_threads`. Claude Code liest beim Start den Kontext
   und schreibt am Ende einer Sitzung eine Zusammenfassung mit Quelle `mcp`. Kostet über
   das bestehende Abo nichts. Einträge sind filter- und sammelweise löschbar.
2. **Verdichteter Brief.** Bei viel Aktivität seit dem letzten Besuch. Modell wahlweise
   lokal (Ollama) oder per eigenem API-Key. Opt-in, pro Aufruf sichtbar, was gesendet wird.
3. **Klassifizierung von Kandidaten**, wenn die Heuristik keine Vorlage erkennt.
4. **Semantische Suche**, wenn der Bestand groß genug ist.

Nicht: KI als Gedächtnis statt Log, Chat als Hauptoberfläche, automatisch erzeugte Aufgaben.

Für jede dieser Stufen und alle künftigen gilt dieselbe Leitplanke: standardmäßig aus,
einzeln freigegeben, vor dem Senden sichtbar, kein Tresor-Zugriff, kein Aufbewahren des
Gesendeten, ein Deckel je Anfrage. Ausgeführt in `THREAT_MODEL.md` 6b. Der Zugang über
den eigenen Schlüssel (lokal oder gehostet) bleibt vollwertig; ein später vom Betreiber
bereitgestellter Zugang ist ein eigener Dienst und nie Teil des Sync-Dienstes.

## 10. Tech-Stack

| Schicht | Wahl | Begründung |
|---|---|---|
| Kern | Rust, Crate `lotse-core` | Datenmodell, SQLCipher-Speicher, Krypto, Sync-Client, Export, Erkennung. Einmal geschrieben, läuft nativ und als WebAssembly. |
| Desktop | Tauri 2 | Kleine Binaries, Tray, echter Dateizugriff, OS-Schlüsselbund, signierter Updater. |
| Oberfläche | Svelte 5 + Vite, TypeScript | Eine Codebasis für Desktop-Webview und Browser. |
| Web-Client | dieselbe Oberfläche + `lotse-core` als WASM | Krypto und Sync im Browser aus demselben Code. |
| Sync-Dienst | Cloudflare Worker (TypeScript) + D1 + R2 | Gratisplan reicht dauerhaft für Einzelnutzer; mandantenfähig für später. |
| Web-Hosting | Cloudflare Pages + Access | kostenlos, Login per Einmalcode vor der App. |
| CLI | Rust, Crate `lotse-cli` | Schnellerfassung, Projekt-Erkennung aus dem Arbeitsverzeichnis. |
| Lokale DB | SQLite über `rusqlite` mit SQLCipher, FTS5 | verschlüsselt im Ruhezustand, Volltextsuche. |
| Beobachter | `notify` | plattformübergreifende Dateisystem-Ereignisse. |
| Git | `git log` als Subprozess | kein libgit2, kein Auth, kein Netz. |
| Krypto | RustCrypto (`argon2`, `hkdf`, `chacha20poly1305`), `zeroize`, `age` für Export | geprüfte Bausteine. |
| Updates | Tauri-Updater, GitHub Releases | kostenlos, signiert. |

## 11. Roadmap

**Phase 0 – Fundament.** Dieses Dokument, `THREAT_MODEL.md`, `NON_GOALS.md`,
`SYNC_PROTOCOL.md`, Rust-Workspace, `lotse-core` mit Datenmodell, verschlüsseltem Speicher,
Krypto und Sync-Client, CLI, Worker-Skelett, UI-Skelett, CI.
*Abschluss:* Ein Projekt wird per CLI angelegt, Notizen geschrieben, der Bestand mit `age`
exportiert und außerhalb von Lotse entschlüsselt. Ein Tresor-Eintrag der Stufe `nur_desktop`
ist ohne Desktop-Schlüssel nachweislich nicht entschlüsselbar.

**Phase 1 – Desktop-MVP** (Zielbudget: 4–6 Feierabend-Wochen). Hafen, Projektseite mit Brief,
Schnellerfassung, Pausieren mit Übergabe, Beobachter, Projekterkennung mit Massen-Import,
Tresor beide Stufen, Suche, offene Punkte, Klartext-Spiegel, Sync gegen den Worker.
*Betriebsregeln ab Tag 1:* Lotse ist Projekt Nr. 1 in Lotse. `NON_GOALS.md` wird nur beim
wöchentlichen Soll-Ist-Check geändert. Jedes neue Feature braucht (a) Bezug zu einem der vier
Kernprobleme, (b) keine laufende Pflegearbeit, (c) einen konkreten Mangel beim dritten echten
Projekt.
*Erfolgsmaß:* fünf echte Projekte, davon mindestens eines ohne Dateien, vier Wochen aktiv
genutzt. *Abbruchkriterium:* wird das drei Monate nach Alltagstauglichkeit nicht erreicht,
wird Lotse eingefroren statt erweitert.

**Phase 2 – Web-Client und MCP.** `lotse-core` als WASM, Web-App auf Pages hinter Access,
Modus "fremder Rechner", MCP-Server. *Auslöser:* Erfolgsmaß aus Phase 1.

**Phase 3 – Wiederverwendung.** Bausteine mit Referenzen und Verwendungsliste, Vorlagen aus
Projekten, Abschluss-Moment. *Auslöser:* drittes Handkopieren desselben Inhalts.

**Phase 4 – Domänentiefe.** Facetten als Config, ICS-Kalender lesend, Projekt-Kanten,
KI-Verdichtung des Briefs, Kanban-Ansicht bei > 15 offenen Fäden in einem Projekt.
*Auslöser jeweils:* konkreter Mangel in einem echten Projekt.

**Phase 5 – Mobil.** Lesend über die Web-App ist es ab Phase 2 da. Native App nur nach sechs
Monaten stabiler Nutzung.

## 12. Entscheidungsprotokoll

| Datum | Entscheidung | Begründung |
|---|---|---|
| 2026-09-07 | KI-Verdichtung gegen eine OpenAI-kompatible Schnittstelle statt gegen einen einzelnen Anbieter. | Ollama (lokal, ohne Schlüssel), Gemini, Groq, Mistral und OpenRouter sprechen dieselbe Schnittstelle. Ein Client, ein Formular, austauschbares Ziel – und der voreingestellte Fall bleibt der, bei dem die Daten den Rechner nicht verlassen. |
| 2026-09-07 | Zwei Wege zur KI, als getrennte Dienste: der eigene Schlüssel (lokal oder gehostet) bleibt kostenlos und vollwertig; ein später vom Betreiber bereitgestellter Zugang ist ein **eigener** Dienst, nie Teil des Sync-Dienstes. | Nur der zweite Weg lässt sich verkaufen, weil dort der Betreiber die Rechnung trägt. Getrennt gehalten bleibt »der Sync-Dienst sieht nur Umschläge« wahr, während ein zweiter, klar benannter Dienst Klartext verarbeitet, den ihm jemand ausdrücklich gegeben hat. Zusammengelegt wäre beides gleichzeitig unwahr und unverkäuflich. |
| 2026-09-07 | Jede KI-Fähigkeit ist standardmäßig aus, wird einzeln freigegeben und zeigt vorher, was gesendet würde. Der gesendete Text wird nicht aufbewahrt; protokolliert wird nur, dass und wohin gesendet wurde. | Ein privates Logbuch kann besondere Kategorien personenbezogener Daten enthalten. Einzelfreigabe erledigt diesen Fall in der Bauart statt in den AGB, und ein Protokoll ohne Inhalt ist der Nachweis, den Rechenschaftspflicht verlangt, ohne selbst eine neue Sammlung zu sein. Ausgeführt in `THREAT_MODEL.md` 6b. |
| 2026-09-07 | Verbrauch wird von Anfang an gezählt und angezeigt, auch ohne Grenze; ein Deckel je Anfrage ist von Anfang an eingebaut. | Die Token-Zahlen kommen in jeder Antwort ohnehin an. Ohne Zählung ab dem ersten Tag gäbe es später keine Grundlage für eine faire Grenze, und ein Limit, das niemand je gesehen hat, wirkt beim Einführen wie ein Rückzieher. |
| 2026-09-07 | Update-Hinweis statt selbsttätigem Updater, solange die Bauten unsigniert sind. | Sich selbst mit unsignierten Binärdaten zu überschreiben wäre der bequemste Angriffsweg auf ein Programm, das Geheimnisse hütet. Lotse sagt Bescheid und nennt die passende Datei; den letzten Schritt macht der Mensch. Mit signierten Bauten wird daraus ein echter Updater. |
| 2026-09-07 | Kalender werden gelesen und angezeigt, nie ins Logbuch kopiert. | Ein Termin gehört dem Kalender. Kopiert, wäre er nach der ersten Verschiebung falsch, und Lotse hätte eine Pflegeaufgabe geschaffen, die es abschaffen will. Gezeigt wird deshalb live, was in den nächsten 90 Tagen ansteht. |
| 2026-09-07 | Uhrzeiten aus Kalendern werden nicht in die Ortszeit umgerechnet. | Ohne Zeitzonendatenbank wäre die Umrechnung geraten. Angezeigt wird, was im Kalender steht; UTC-Zeiten werden gekennzeichnet. Das ist für „was steht an“ genau genug und nie falsch. |
| 2026-09-07 | TLS gegen den Wurzelspeicher des Systems statt gegen eine mitgelieferte Liste. | In Netzen mit TLS-Prüfung fiele Lotse sonst aus, während Browser und `git` daneben laufen. Der Abgleich hängt nicht daran: der Dienst sieht ohnehin nur Umschläge. Vermerkt in `THREAT_MODEL.md` 6a. |
| 2026-09-07 | Das Repo auf der Gegenseite wird am Git-Remote eines Ordners erkannt, nicht von Hand eingetragen. | Eine Adresse, die jemand pflegen muss, veraltet. `git remote -v` weiß es ohnehin — und weiß es richtig, auch nach einem Umzug. Eine ausdrücklich eingetragene Adresse hat weiterhin Vorrang. |
| 2026-09-07 | GitLab genauso angebunden wie GitHub, gegen aufgezeichnete echte Antworten geprüft. | Der Einwand gegen GitLab war „ungetestet gegen eine Schnittstelle, die hier niemand ausprobieren kann“. Mit echten Antworten in `tests/daten/` fällt er weg. Verschachtelte Gruppen und `/-/` in Weblinks sind dabei berücksichtigt; Merge Requests heißen im Logbuch auch so. |
| 2026-09-07 | Remote-Git lesend angebunden, obwohl der Auslöser aus `NON_GOALS.md` nicht eingetreten war. | Vom Nutzer priorisiert. Der Abruf schreibt eine verdichtete Zeile pro Repo, keine Aufgaben — Tickets und Backlogs bleiben ausgeschlossen. Der Token liegt im Tresor, das Modul `forge` liest ihn nie selbst. |
| 2026-09-06 | Desktop-App ist das Hauptwerkzeug, Web-App ein vollwertiger Zweitclient. | Zugriff von fremden Rechnern ist Anforderung; Beobachter und OS-Integration gehen nur nativ. |
| 2026-09-06 | Sync über eigene Engine mit Änderungsprotokoll statt Git. | Git merged Textzeilen, keine Datensätze; Binärdateien; Identität an GitHub gebunden; Historie im Klartext bei Dritten. |
| 2026-09-06 | Ende-zu-Ende-Verschlüsselung, Server sieht nur Ciphertext. | Erledigt die Klartext-Frage, senkt Haftung, ist Verkaufsargument. |
| 2026-09-06 | Cloudflare Worker + D1 + R2 + Pages + Access als Gratis-Zuhause. | Keine laufenden Kosten, kein Server zu pflegen; Protokoll bleibt portabel. |
| 2026-09-06 | Wahrheit ist die lokale Datenbank mit Änderungsprotokoll, Klartext ist Spiegel. | Datensätze lassen sich synchronisieren, Textdateien nicht. Fluchtweg bleibt über Spiegel und Export. |
| 2026-09-06 | Zufälliger Account-Schlüssel mit Wrap durch Passwort und Wiederherstellungscode. | Passwortwechsel ohne Neuverschlüsselung; Schutz vor vergessenem Passwort. |
| 2026-09-06 | Desktop-Schlüssel für Tresor-Stufe `nur_desktop`. | Fremde Rechner dürfen die wertvollsten Geheimnisse nie entschlüsseln können. |
| 2026-09-06 | SQLCipher für die lokale Datenbank. | Gestohlener Laptop ist das realistischste Bedrohungsszenario. |
| 2026-09-06 | `account_id` auf jedem Datensatz, Feature-Flags am Konto. | Spätere Monetarisierung ohne Migration (siehe `BUSINESS.md`). |
| 2026-09-06 | Proton Pass wird nur referenziert, nicht live angebunden. | Web-Logins bleiben dort; Lotse hält Projektgebundenes. CLI-Anbindung mit scoped Tokens ist Roadmap-Option. |
| 2026-09-06 | Lizenz noch nicht festgelegt, Repo bleibt privat. | Entscheidung vor erster Veröffentlichung, siehe `BUSINESS.md`. |
