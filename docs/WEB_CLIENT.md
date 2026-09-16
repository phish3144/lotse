# Der Web-Client: auf jedem Gerät anmelden und seine Vorhaben sehen

Stand: 2026-09-16, Phase W1 erledigt. Dieses Dokument plant die größte offene Lücke aus `CONCEPT.md`
Abschnitt 11, Phase 2. Verbindlich bleiben `THREAT_MODEL.md` (was der Dienst nie sieht),
`SYNC_PROTOCOL.md` (die Schnittstelle) und `NON_GOALS.md` (der Aufnahmetest).

## 1. Das Ziel, in einem Satz

> Auf jedem Gerät – lokal installiert oder im Browser – mit E-Mail und Master-Passwort
> anmelden und seine Vorhaben sehen.

Kein Link kopieren, keine Dienstadresse eintragen, keine Datei mitnehmen.

## 2. Was heute geht, und was nicht

| | Stand |
|---|---|
| Zweites **installiertes** Gerät | **läuft.** `lotse sync login --email …`, seit dem eingebauten Standarddienst ohne Adresse. Der E2E-Test führt zwei Geräte vor, die sich gegenseitig abgleichen. |
| Konto mit E-Mail und Passwort | **läuft.** Nullwissen: das Passwort verlässt das Gerät nie, der Dienst kennt nur einen Hash des abgeleiteten `auth_key` und den gewrappten Kontoschlüssel. |
| Konto löschen | **läuft** (seit 0.9.x). |
| Konto **anlegen** | **läuft** am Desktop und in der Kommandozeile (`lotse init`); im Browser geplant für W2 (4d). |
| Kern im Browser | **läuft** (W1). `crates/lotse-wasm` bindet Krypto, Umschläge, Logikuhr und Brief an; 18 Prüfungen laufen in einem echten Chromium in CI. |
| **Browser-Oberfläche** | **fehlt.** `apps/web` läuft dort weiter gegen Beispieldaten – die Anbindung ist da, der Datenweg noch nicht. |

Gemessen, nicht geschätzt:

| Baustein | WASM-fähig? |
|---|---|
| `crypto` (Argon2id, Schlüsselhierarchie), `model`, `vault`, `hlc` | **ja**, heute schon |
| `sync` – Umschlag `seal`/`open`, HLC, Konfliktregel `gewinnt` | **ja**, heute schon |
| `brief` – Wo-war-ich-Brief und Auffälligkeit | **ja**, und **reine Funktionen** ohne eine Zeile SQL |
| `store` (rusqlite/SQLCipher, 45 Methoden) | nein |
| `sync::client` (ureq) | nein |
| `detect`, `deuten`, `git`, `forge`, `konto`, `export`, `watcher`, `mcp` | nein, brauchen Dateisystem |

`cargo check -p lotse-core --no-default-features --target wasm32-unknown-unknown` läuft in
CI und ist grün. Die Vorarbeit ist also echt.

## 3. Die entscheidende Einsicht

**Der Browser braucht `store` nicht.**

Die naheliegende Lesart – »SQLCipher im Browser nachbauen« – wäre eine zweite
Implementierung von 45 Methoden samt Schema, FTS und Migrationen. Zwei Wahrheiten über
dieselben Daten, und die zweite wäre die, die niemand testet.

Der Vertrag der Oberfläche ist aber nicht `store`, sondern
`apps/web/src/lib/data/provider.ts` mit **27** Methoden. Und was der Browser braucht, um
Vorhaben zu zeigen, ist:

1. `login` → Salt, KDF-Parameter, gewrappter Kontoschlüssel
2. Argon2id über das Passwort → Kontoschlüssel entwrappen
3. `pull` → alle Umschläge
4. `Umschlag::open` je Datensatz → `Projekt`, `Notiz`, `Referenz` im Speicher
5. `brief::*` → Hafen, Auffälligkeit, Brief

Kein SQLite, kein Schema, keine Migration. Die Suche ist ein linearer Durchlauf über die
Notizen im Speicher – bei einem persönlichen Bestand schneller als der Netzaufruf, der ihn
geholt hat.

## 4. Entscheidungen

### 4a. Speicher im Browser: verschlüsselte Umschläge in IndexedDB, Index im Speicher

| Weg | Dafür | Dagegen | |
|---|---|---|---|
| **A** Nichts speichern, nur Arbeitsspeicher | Einfachster Fall, nichts bleibt liegen | Bei jedem Öffnen der ganze Bestand über das Netz | Grundlage |
| **B** SQLite in WASM (OPFS) | Echtes SQL, FTS | `rusqlite` kann `wasm32-unknown-unknown` nicht; also eine Fremdanbindung und die SQL-Schicht doppelt | **verworfen** |
| **C** Umschläge in IndexedDB, Index im Speicher | Kein zweites Schema, Cache ist undurchsichtiger Ciphertext, Wiederbesuch sofort da | Etwas mehr Code als A | **gewählt** |

Der Kern von C: **gespeichert wird, was der Dienst liefert** – versiegelte Umschläge, Byte
für Byte. Entschlüsselt wird beim Öffnen in den Arbeitsspeicher. Damit gilt:

- Kein zweites Datenschema, das auseinanderlaufen kann.
- Was im Browser liegt, ist genau das, was der Dienst ohnehin hat. Der Ruhezustand wird
  nicht schlechter als das Bedrohungsmodell schon beschreibt.
- Der Schlüssel liegt **nie** in IndexedDB, nur im Arbeitsspeicher der Sitzung.
- »Fremder Rechner« ist keine Sonderbauweise, sondern ein Schalter: Cache aus.

### 4b. Argon2id läuft in einem Web Worker

Die Produktionsparameter sind `m = 64 MiB, t = 3, p = 1`. Auf dem Hauptthread friert das
die Seite ein. Also: Worker, sichtbarer Fortschritt, und die Parameter kommen vom Dienst
(`prelogin`), nicht aus dem Client – ein Gerät darf die Kosten nicht senken.

Auf schwachen Telefonen dauert das mehrere Sekunden. Das ist zu **zeigen**, nicht zu
verstecken, und es ist der Preis dafür, dass der Dienst das Passwort nie sieht.

### 4c. Das Protokoll bleibt an einer Stelle – der Weg dorthin wurde korrigiert

`sync::client` spricht ureq und ist damit nativ. Die Protokolllogik darf aber nicht zweimal
existieren – ein in TypeScript nachgebautes Protokoll ist der sicherste Weg zu zwei
Wahrheiten. Das gilt unverändert.

Geplant war dafür ein schmales `Transport`-Trait im Kern. Beim Bauen von W1 kam ein
Hindernis heraus, das im Plan fehlte: **`fetch` ist asynchron, `ureq` ist blockierend.**
Ein gemeinsames Trait muss also `async` sein, und damit werden alle Client-Methoden
`async` – samt der rund zwanzig nativen Aufrufstellen in CLI und Hülle, die dann einen
`block_on` brauchen. Das ist machbar, aber es ist ein Eingriff in den fertigen, nativ
getesteten Weg, und er ließe sich erst prüfen, wenn der Browser ihn wirklich benutzt.

Deshalb steht das Trait jetzt in **W2**, dort wo es gebraucht und mitgetestet wird. In W1
wurde absichtlich keine Abstraktion gebaut, die nichts benutzt: sie wäre grün, ohne etwas
zu beweisen. Die Richtung bleibt: **eine** Protokollwahrheit, im Kern.

### 4d. Der Browser ist zuerst lesend – plus Schnellerfassung

Ein Vorhaben **anlegen** heißt im Browser: Ordner deuten, Git lesen, Dateien finden. All
das gibt es dort nicht. Statt einer halben Fassung dieser Dinge:

| Im Browser | Stand 1 |
|---|---|
| Hafen, Projektseite, Brief, offene Punkte, Suche, Referenzen ansehen | **ja** |
| Faden abhaken, Notiz schreiben, Status setzen | **ja** – das sind Umschläge, kein Dateisystem |
| Vorhaben anlegen (Befund), Ordner scannen, Export, Gegenseite, MCP | **nein**, sichtbar ausgegraut mit Grund |
| Konto anlegen und anmelden | **ja** – mit abgetipptem Wiederherstellungscode, siehe unten |
| Tresor Stufe `ueberall` lesen | **ja** |
| Tresor Stufe `nur_desktop` | **konstruktionsbedingt nein** – der Desktop-Schlüssel kommt nie in einen Browser. Wird als solches angezeigt, nicht als Fehler. |

**Konto anlegen geht auch im Browser** – mit derselben Strenge wie am Desktop. Der
Wiederherstellungscode entsteht dabei im Browser und verlässt ihn nie; er wird angezeigt
**und muss abgetippt werden**, bevor es weitergeht – genau wie bei `lotse init`. Ohne
richtig abgetippten Code wird kein Konto beim Dienst angelegt. Das ist die eine Stelle, an
der die Oberfläche bewusst im Weg steht: ein verlorener Code heißt verlorene Daten, denn
der Dienst kann nicht helfen (`THREAT_MODEL.md`).

### 4e. Ein Browser ist ein Gerät

Gerätekennung je Browser in `localStorage`, Plattform `web`. Sonst wächst die Geräteliste
mit jedem Besuch. Die Sitzung hält im Web **12 Stunden** statt 30 Tage – das steht schon
im Protokoll und im Dienst.

### 4f. Kein Sanctora-Konto nötig

Das Ziel verlangt keins. Lotses Konto ist bereits E-Mail plus Passwort und geräteübergreifend.
Ein gemeinsamer Identitätsdienst würde hier nichts hinzufügen und müsste die
Schlüsselableitung mittragen (`BUSINESS.md`). Getrennt lassen; später gemeinsame
**Abrechnung**, nicht gemeinsame Identität.

## 5. Phasen mit Abschlusskriterium

Jede Phase endet an etwas Nachprüfbarem, nicht an »fertig«.

### Phase W1 · Der Kern spricht im Browser — **erledigt (0.10.x)**

`wasm-bindgen`-Anbindung für `crypto`, `model`, `sync`, `brief`, `vault` in
`crates/lotse-wasm`. Bau über `scripts/wasm-bauen.sh` (Fassung von `wasm-bindgen` wird aus
`Cargo.lock` gelesen, nicht geraten), Ergebnis nach `apps/web/src/lib/wasm/`. Eigener
CI-Job. Das Transport-Trait ist nach 4c in W2 gewandert.

**Abschluss erreicht:** `tests/vektoren/kdf.json` hält Passwort, Salt und beide
Parametersätze samt erwarteter Schlüssel fest. Dieselbe Datei lesen
`crates/lotse-core/tests/kdf_vektoren.rs` und `apps/web/scripts/wasm-browsertest.mjs`; der
Browsertest leitet in einem echten Chromium denselben `auth_key` ab — auch mit den
Produktionsparametern m = 64 MiB, t = 3. Gegengeprüft: ein verfälschter Vektor macht beide
Seiten rot.

Nebenbefund, der schwerer wog als die Anbindung selbst: `now_ms()` gab unter `wasm32`
schlicht **0** zurück. Aus dieser Uhr kommt der Zeitanteil jedes HLC-Werts, also hätte
jeder Schreibvorgang aus dem Browser jeden Konflikt verloren. Jetzt `Date.now()`.

### Phase W2 · Anmelden und den Hafen sehen

Transport-Trait nach 4c (asynchron, `ureq` nativ und `fetch` im Browser) samt Umstellung
der nativen Aufrufstellen. Darauf: Login über den eingebauten Dienst, Argon2id im Worker
mit Fortschritt, Pull aller Umschläge, Entsiegeln in den Speicher, `brief`-Funktionen
darüber, Hafen und Projektseite durch den vorhandenen `DataProvider`. Dazu das Anlegen
eines Kontos nach 4d, samt Abtippschritt für den Wiederherstellungscode.

**Abschluss:** in einem echten Browser gegen einen echten Dienst anmelden und die Vorhaben
sehen, mit gezählten Datensätzen – kein Mock, kein Screenshot als Beweis.

### Phase W3 · Wiederkommen ohne Wartezeit

Umschlag-Cache in IndexedDB nach 4a, inkrementeller Pull ab `last_server_seq`, Schalter
»fremder Rechner« (kein Cache), Abmelden räumt auf.

**Abschluss:** zweiter Besuch zeigt den Hafen, bevor der Netzaufruf zurück ist; nach
»Abmelden« ist in IndexedDB nichts mehr.

### Phase W4 · Schreiben, so weit es ohne Dateisystem geht

Notiz, Faden abhaken, Status, Kurs. Umschläge mit HLC der Browser-Gerätekennung, Push,
Konfliktregel aus dem Kern.

**Abschluss:** ein Faden im Browser abgehakt erscheint nach dem Abgleich auf dem Desktop,
und ein Konflikt landet als Notiz im Logbuch statt still zu verschwinden.

### Phase W5 · Ausliefern

Statisches Bündel auf **`app.lotse.sanctora.eu`** (entschieden, Cloudflare), eigene Kopfzeilen mit
`connect-src` nur auf den Sync-Dienst, keine Fremdressourcen – dieselbe Regel wie für
`site/`.

**Abschluss:** von einem fremden Rechner aus anmelden und Vorhaben sehen, und die
ausgelieferten Kopfzeilen sind nachgemessen, nicht behauptet.

## 6. Risiken, ehrlich benannt

| Risiko | Was dagegen steht |
|---|---|
| Argon2id mit 64 MiB im Browser ist zäh, besonders mobil | Worker und sichtbarer Fortschritt. Die Parameter zu senken wäre der falsche Ausweg – sie schützen genau das Passwort, das der Dienst nie sieht. |
| Der ganze Bestand im Arbeitsspeicher | Für einen persönlichen Bestand unkritisch; **vor** W3 an echten Daten messen, nicht schätzen. Wird es zu groß, holt der Pull seitenweise und der Index bleibt schlank. |
| IndexedDB ist kein Tresor | Deshalb liegen dort nur Umschläge, nie Schlüssel. Wer strenger will, nimmt »fremder Rechner«. |
| Zwei Oberflächenwege laufen auseinander | Es bleibt **eine** Schnittstelle (`provider.ts`). Was im Browser fehlt, wird ausgegraut mit Grund, nicht weggelassen. |
| Der Bau wird schwerfällig (WASM + Vite + CI) | In W1 einmal sauber gemacht: ein Skript, ein CI-Job, keine neue npm-Abhängigkeit. Der Browsertest fährt einen vorhandenen Chromium, statt Playwright mitzuschleppen. |
| Die Anbindung wächst zu einer zweiten Fassung der Logik | `crates/lotse-wasm` enthält ausschließlich Adapter. Das ist eine Regel in `CLAUDE.md`, keine Absicht – sie fällt beim Lesen auf, weil jede Funktion sonst länger als drei Zeilen wird. |

## 7. Was hier ausdrücklich nicht gebaut wird

- Keine Zweitfassung der Erkennung, des Git-Lesers oder des Exports in JavaScript.
- Kein Schema-Editor, keine Team-Funktionen, kein Mehrbenutzerbetrieb (`NON_GOALS.md`).
- Keine eigene Identitätsschicht (4f).
- Kein Konto-Anlegen **ohne** abgetippten Wiederherstellungscode – im Browser so wenig wie
  am Desktop (4d).
- Keine Absenkung der KDF-Kosten, um im Browser schneller zu sein.
