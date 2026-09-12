# Roadmap

Was gebaut ist, was ansteht, und **woran es hängt**. Der letzte Teil ist der wichtigste:
in Lotse hat jedes verschobene Feature einen Auslöser – ein Ereignis, das eintreten muss,
bevor es gebaut wird. Ohne Auslöser ist eine Roadmap eine Wunschliste, und eine
Wunschliste wird abgearbeitet, statt dass gefragt wird, ob sie noch stimmt.

Verbindlich sind `docs/CONCEPT.md` Abschnitt 11 (Phasen), `docs/NON_GOALS.md` (Auslöser
und Abbruchkriterium) und `docs/BUSINESS.md` (alles Kommerzielle). Diese Seite führt die
drei zusammen und trägt den Stand nach.

Was in welcher Fassung tatsächlich fertig wurde: [[Änderungen|Aenderungen]]. Was
**dauerhaft** nicht kommt: [[Nicht-Ziele]].

---

## Phase 0 · Fundament — **fertig**

Datenmodell, Kryptografie, verschlüsselter Speicher, Sync-Umschläge und -Client, die
Kommandozeile, das Worker-Skelett, CI.

Das Abschlusskriterium war nicht »es kompiliert«, sondern: ein Tresor-Eintrag der Stufe
`nur_desktop` ist ohne Desktop-Schlüssel **nachweislich** nicht entschlüsselbar, und der
Bestand lässt sich mit dem generischen `age`-Werkzeug außerhalb von Lotse öffnen. Beides
ist als Test da, nicht als Behauptung ([[Schlüssel und Krypto|Schluessel-und-Krypto]],
[[Export und Fluchtweg|Export-und-Fluchtweg]]).

---

## Phase 1 · Desktop-MVP — **funktional fertig, Erfolgsmaß offen**

| Soll | Stand |
|---|---|
| Hafen, Projektseite mit Brief, Schnellerfassung | läuft ([[Oberfläche\|Oberflaeche]], [[Wo-war-ich-Brief]]) |
| Pausieren mit Übergabe | läuft – die Notiz ist Pflicht ([[Vorhaben]]) |
| Ordner-Beobachter, Erkennung mit Massen-Import | läuft ([[Beobachter und Erkennung\|Beobachter-und-Erkennung]]) |
| Tresor, beide Stufen | läuft ([[Tresor]]) |
| Suche, offene Punkte, Klartext-Spiegel | läuft ([[Offene Punkte\|Offene-Punkte]]) |
| Abgleich gegen den Worker | läuft, mit Test gegen einen echten Dienst ([[Abgleich]]) |

### Das Erfolgsmaß steht nicht im Repository

> Fünf echte Vorhaben, davon mindestens eines **ohne Dateien**, vier Wochen aktiv genutzt.

Das lässt sich am Code nicht ablesen – es steht in der eigenen Lotse-Datenbank. Lotse ist
Projekt Nr. 1 in Lotse; sein Logbuch ist laut `NON_GOALS.md` der Nachweis.

Daran hängen zwei Dinge: der **Auslöser für Phase 2** und das **Abbruchkriterium**.

> Werden drei Monate nach Alltagstauglichkeit des Desktop-MVP nicht mindestens fünf echte
> Projekte aktiv geführt und Lotse nicht wöchentlich freiwillig geöffnet, wird Lotse
> **eingefroren statt weiter ausgebaut**.

Das ist kein Ritual. Ein Werkzeug, das seine eigene Erfinderin nicht benutzt, wird durch
mehr Features nicht besser – es wird nur größer.

---

## Phase 2 · Web-Client und Assistenten — **halb**

*Auslöser: das Erfolgsmaß aus Phase 1.*

| Soll | Stand |
|---|---|
| MCP-Server | **läuft** – über stdin/stdout *und* aus der laufenden App auf `127.0.0.1`, fünf Werkzeuge, Tresor unsichtbar ([[Assistenten (MCP)\|Assistenten-MCP]]) |
| `lotse-core` als WebAssembly | **halb** – der Kern baut ohne das Feature `native` und das wird in CI geprüft; eine Anbindung an den Browser gibt es nicht |
| Web-App im Browser | **fehlt** – `apps/web` läuft dort gegen Beispieldaten, nicht gegen den Kern |
| Modus »fremder Rechner« ohne Persistenz | **fehlt** |

Das ist die größte echte Lücke. Die Vorarbeit ist getan – die Oberfläche spricht überall
durch **eine** Schnittstelle, und der Kern ist WASM-tauglich gehalten ([[App-Schnittstelle]]).
Was fehlt, ist die Brücke dazwischen.

Für den Tresor ist der Fall schon vorgesehen: die Stufe `nur_desktop` ist im Browser
konstruktionsbedingt nicht lesbar, weil der Desktop-Schlüssel nie dorthin kommt.

---

## Phase 3 · Wiederverwendung — **nichts davon**

| Soll | Auslöser |
|---|---|
| Bausteine mit Referenzen und Verwendungsliste | derselbe Inhalt **dreimal** von Hand kopiert |
| Vorlagen aus bestehenden Vorhaben | – |
| Abschluss-Moment | – |

Gemeint sind mit »Bausteinen« projektunabhängige Dinge: eine Checkliste, ein Verfahren,
eine Bezugsquelle, ein Kontakt – einmal hinterlegt, in mehreren Vorhaben eingebunden, mit
Anzeige, wo sie verwendet werden.

Der Auslöser ist streng gewählt. Zweimal kopieren ist Zufall; dreimal ist ein Muster.

---

## Phase 4 · Domänentiefe — **zwei von fünf, beide vorgezogen**

| Soll | Stand | Auslöser |
|---|---|---|
| Kalender lesend (`.ics`) | **läuft** | war: »ein terminlastiges Projekt existiert real« – vorgezogen |
| KI-Verdichtung des Briefs | **läuft** | war: »Brief regelmäßig länger als ein Bildschirm« – vorgezogen |
| Facetten (Stückliste, Kostenaufstellung) als Config | fehlt | dreimal eine Tabelle in Freitext gequetscht |
| Projekt-Kanten `teilprojekt_von`, `verwandt_mit` | fehlt | Unterprojekte treten real mehrfach auf |
| Kanban-Ansicht | fehlt | ein Vorhaben mit **dauerhaft** mehr als 15 offenen Fäden |

Beide vorgezogenen Punkte sind in `NON_GOALS.md` unter *Vorzeitig aufgenommen* mit
Aufnahmetest protokolliert – bewusst überschrieben, nicht übersehen. Das ist der
Unterschied zwischen einer Ausnahme und einem Leck.

Dasselbe gilt für zwei weitere Punkte, die gar nicht in den Phasen standen: der Abruf von
[[Gegenseite|Gegenseite]] (GitHub, GitLab) und [[Datei deuten|Datei-deuten]].

`abgeleitet_von` gibt es im Modell ([[Datenmodell]]) – das ist eine Spur, keine Kante.
Facetten wären ausdrücklich eine **versionierte Konfiguration**, kein Schema-Editor zur
Laufzeit; letzterer wäre genau die Pflegearbeit, die Lotse abschaffen will.

---

## Phase 5 · Mobil — **nichts**

Lesend wäre es ab Phase 2 über die Web-App da. Eine **native** App erst nach sechs
Monaten stabiler Desktop- und Web-Nutzung.

---

## Außerhalb der Phasen

### Vor einer Veröffentlichung nötig

Aus `BUSINESS.md`, Abschnitt 4. Kein Punkt davon ist erledigt.

| Punkt | Warum es nicht wartet |
|---|---|
| **Lizenz festlegen** | Das Repository ist **öffentlich ohne `LICENSE`**. Damit gilt »alle Rechte vorbehalten« bei einsehbarem Code – laut `BUSINESS.md` »ein Zustand, kein Plan«. Zur Wahl stehen AGPL-3.0, FSL/BSL oder proprietär mit einsehbarer Quelle. |
| Impressum, Datenschutzerklärung, AV-Vertrag mit Cloudflare | Rechtlich zwingend, sobald jemand anderes den Dienst benutzt. |
| Markenrecherche »Lotse« (DPMA, EUIPO) und Domain | Je später, desto teurer eine Umbenennung. |
| Zahlungsabwicklung über einen Merchant of Record | Damit die EU-Umsatzsteuer nicht selbst abzuführen ist. |
| **Code-Signierung und Notarisierung** (Windows, macOS) | Nicht zu verwechseln mit den Update-Signaturen, die es **gibt**: die schützen den Austausch, nicht die Erstinstallation. SmartScreen und Gatekeeper warnen weiterhin ([[Updates]], [[Installation]]). |
| Externe Prüfung der Krypto-**Komposition** | Laut `BUSINESS.md` bevor fremde Geheimnisse damit verwaltet werden. Nicht die Primitive – die sind geprüft –, sondern ihre Zusammensetzung. |
| Cloudflare Workers Paid Plan | Ab dem ersten zahlenden Nutzer, weil die Gratis-Grenzen dann verbindlich überschritten werden dürfen. |

Solange das Abbruchkriterium nicht bestanden ist, wird laut `BUSINESS.md` bewusst
**nichts** davon gebaut: kein Bezahl-Code, keine Lizenzschlüssel, keine Kontenpläne mit
Wirkung. Zuerst muss Lotse für **eine** Person unverzichtbar sein.

### Kleinere offene Punkte

| Punkt | Stand |
|---|---|
| Symbol im Infobereich, globales Erfassen-Kürzel | Das Tauri-Feature `tray-icon` ist in `Cargo.toml` aktiviert, aber im Code nirgends benutzt – eine Abhängigkeit ohne Funktion. Bis dahin ist die [[Kommandozeile]] der schnellste Weg von außen. |
| Konto im Dienst löschen | Nicht gebaut. Bis dahin: Datenordner löschen und beim Dienst alle Geräte widerrufen ([[Einstellungen]]). |
| Selbst gehostete Git-Instanzen (GitHub Enterprise, eigenes GitLab) | Werden nicht erkannt. Der lokale Git-Log funktioniert dort ([[Gegenseite]]). |
| Feldweises Mergen beim Abgleich | Auslöser: ein **zweiter** realer Datenverlust durch Last-Writer-Wins. Heute schreibt Lotse die unterlegene Fassung als Notiz ins Logbuch, damit nichts stumm verschwindet ([[Sync-Protokoll]]). |
| Semantische Suche, Embeddings | Auslöser: die Volltextsuche findet **nachweislich** nicht mehr. |
| Proton-Pass-Anbindung über dessen CLI | Auslöser: Verweise werden häufig angeklickt und nerven. |
| Rückfluss aus dem Klartext-Spiegel | Auslöser: der Spiegel wird regelmäßig extern bearbeitet. |
| Sync-Dienst als eigenes Rust-Binary | Auslöser: Cloudflare ändert die Bedingungen, oder Selbsthosting wird gewünscht. |

---

## Wie diese Seite aktuell bleibt

Die Prüfung `scripts/wiki_pruefen.sh` kann sie **nicht** prüfen: sie liest ablesbare
Oberfläche aus dem Quelltext, und ein Vorhaben ist nicht ablesbar
([[Wiki pflegen|Wiki-pflegen]]).

Diese Seite hängt deshalb an der Hand. Bei jeder Fassung:

1. Was fertig wurde, hier auf »läuft« setzen und in [[Änderungen|Aenderungen]] eintragen.
2. Ein eingetretener Auslöser wird hier notiert – auch wenn noch nichts gebaut wird. Ein
   Auslöser, der eintritt und niemandem auffällt, ist keiner.
3. Wurde etwas **vor** seinem Auslöser gebaut, gehört es in `NON_GOALS.md` unter
   *Vorzeitig aufgenommen*, mit Aufnahmetest. Sonst ist die Liste in einem Jahr Zierde.

Der Aufnahmetest für jedes neue Feature steht in [[Nicht-Ziele]] und hat drei Fragen. Die
zweite ist die, an der die meisten Ideen scheitern: *entsteht dadurch laufende
Pflegearbeit?*
