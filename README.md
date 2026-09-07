# Lotse

Das Logbuch für alle deine Vorhaben. Software, Werkstatt, Haus, Musik, Steuer, Reise:
für jedes Projekt an derselben Stelle *worum geht es, wo stehe ich, was war zuletzt, was ist
der nächste Schritt, wo liegt das Material, wie komme ich rein?*

Lotse löst vier Probleme: Wiedereinstieg in alte Projekte, den Faden nicht verlieren, das
Rad nicht neu erfinden, Geheimnisse sicher am Projekt aufbewahren.

## Status

Phase 0, Fundament. Siehe `docs/CONCEPT.md` Abschnitt 11 für die Roadmap.

| Teil | Stand |
|---|---|
| Konzept, Bedrohungsmodell, Nicht-Ziele, Sync-Protokoll | geschrieben (`docs/`) |
| Rust-Kern: Modell, Krypto, Tresor mit zwei Stufen, Sync-Umschläge, SQLCipher-Speicher mit Änderungsprotokoll und Volltextsuche, Projekterkennung, Git-Import, Spiegel- und age-Export | läuft, getestet |
| CLI `lotse` | läuft |
| Web-Oberfläche (Svelte) | Gerüst mit Mock-Daten |
| Sync-Dienst (Cloudflare Worker) + Sync-Client im Kern und in der CLI | läuft, End-to-End getestet (`scripts/sync-e2e.sh`) |
| Desktop-Hülle (Tauri 2) | läuft. Anlegen, erfassen, Fäden abhaken, Status mit Übergabe, Projektkopf bearbeiten, Referenzen anlegen/prüfen/öffnen, Tresor lesen/anlegen/löschen, Ordner scannen und laufend beobachten, Stand von GitHub/GitLab holen, Termine aus abonnierten Kalendern, KI-Verdichtung des Briefs, Update-Hinweis, Export (Spiegel und age-Bundle), Abgleich mit Geräteverwaltung, Passwortwechsel und Konto-Wiederherstellung. Offen: Tray, Auto-Lock, selbsttätiger signierter Updater, MCP aus der entsperrten Sitzung |
| Landing Page (`site/`) | fertig, Deploy per GitHub Pages |
| Ordner-Beobachter | läuft, in der CLI (`lotse beobachten`) und in der Desktop-App |
| MCP-Server (`lotse mcp`) | läuft |
| Remote-Git (GitHub und GitLab: offene PRs/MRs, Issue-Zahl, Prüflauf) | läuft im Kern, in der App und in der CLI (`lotse gegenseite`) |
| KI-Verdichtung des Briefs (Ollama, Gemini, jede OpenAI-kompatible Adresse) | läuft im Kern und in der App |
| Kalender lesend (iCalendar/.ics, auch `webcal://`) | läuft im Kern, in der App und in der CLI (`lotse termine`) |
| Update-Hinweis (neue Version erkennen, Datei fürs System nennen) | läuft in der App und in der CLI (`lotse update`); lädt bewusst nichts herunter |
| WebAssembly-Client für den Browser | offen |

## Bekannte Grenzen

- **Der Ordner-Beobachter hält beim Schreiben kurz die Sitzung.** Er teilt sich den
  Speicher mit der Oberfläche, und sein Schreibdurchlauf ruft `git log` je Repo auf.
  Bei vielen Repos stockt die Oberfläche dabei spürbar. Sauber lösen ließe sich das mit
  einer eigenen Datenbankverbindung für den Beobachter – das berührt aber die
  HLC-Vergabe und damit den Abgleich, deshalb steht es aus und wird nicht nebenbei
  gemacht.
- **Anhalten wartet nicht auf das Ende des Threads.** Nach „Anhalten" oder „Sperren"
  läuft der Beobachter-Thread bis zu 20 Sekunden weiter, bis sein Warteintervall
  abläuft; ist gerade eine Abfrage bei GitHub oder GitLab unterwegs, bis deren Zeit
  abgelaufen ist. Er schreibt in dieser Zeit nichts mehr: der Stop-Schalter wird vor
  jedem Schreiben geprüft, auch zwischen zwei Repos. Es bleibt ein untätiger Thread,
  keine Änderung an den Daten.
- **Selbst betriebene GitLab- und GitHub-Instanzen** werden nicht erkannt, nur `github.com`
  und `gitlab.com`. Der Hostname einer eigenen Instanz lässt sich nicht erraten; dafür
  braucht es eine Einstellung, die es noch nicht gibt.
- **CalDAV spricht Lotse nicht.** Gelesen wird die iCalendar-Datei, die jeder Dienst als
  Abonnement-Adresse ausgibt (Google, Nextcloud, iCloud, Outlook). Das deckt den Fall
  „zeig mir, was ansteht" ab; ein echter CalDAV-Client wäre ein eigenes Stück Arbeit und
  bringt für reines Lesen nichts dazu.
- **Zeitzonen werden nicht umgerechnet.** Eine Uhrzeit wird so gezeigt, wie sie im
  Kalender steht; Zeiten in UTC werden als solche gekennzeichnet. Ohne
  Zeitzonendatenbank wäre jede Umrechnung geraten.
- **Komplizierte Wiederholungsregeln werden nicht ausgerechnet.** Täglich, wöchentlich,
  monatlich, jährlich samt Intervall, Anzahl, Stichtag und Wochentagen: ja. „Zweiter
  Montag im Monat“ und Verwandtes: dann steht nur der erste Termin da, mit dem Hinweis,
  dass er sich wiederholt.
- **Der selbsttätige Abruf der Gegenseite hängt am Ordner-Beobachter.** Läuft der
  nicht, fragt Lotse GitHub und GitLab nur auf Knopfdruck. Das ist Absicht: ein
  zweiter Hintergrundthread, der ohne sichtbaren Grund ins Netz geht, wäre schlechter
  zu durchschauen als einer.

## Wann Lotse von allein ins Netz geht

Vollständig, damit nichts überrascht. Alles andere passiert nur auf Knopfdruck.

| Wann | Wohin | Abschaltbar |
|---|---|---|
| Beim Entsperren der App, höchstens einmal am Tag | Veröffentlichungen dieses Projekts auf GitHub (Update-Hinweis) | ja, *Einstellungen → Version und Updates* |
| Beim Öffnen einer Projektseite, die eine Kalender-Referenz hat, höchstens alle 15 Minuten je Adresse | Die eingetragene Kalenderadresse | ja, indem die Referenz entfernt wird |
| Im Ordner-Beobachter, höchstens alle 30 Minuten | GitHub bzw. GitLab zu den erkannten Repos | ja, ausgeschaltet bis eingeschaltet, *Einstellungen → GitHub und GitLab* |
| Beim Abgleich | Der eingerichtete Sync-Dienst | ja, kein Abgleich eingerichtet = keine Verbindung |

Die KI-Verdichtung sendet nur, wenn jemand auf *Senden* drückt, und zeigt vorher den
vollständigen Text. Telemetrie gibt es nicht, auch keine anonyme.

## Ausprobieren (CLI)

```
cargo build -p lotse-cli
export LOTSE_HOME=$PWD/.lotse-daten
./target/debug/lotse init --geraet "Laptop"
./target/debug/lotse projekt neu "Gartenhaus" --vorlage haus --kurs "Fundament bis Oktober."
./target/debug/lotse log "@Gartenhaus Beton bestellt"
./target/debug/lotse log --projekt Gartenhaus --offen "Bewehrung nötig?"
./target/debug/lotse hafen
./target/debug/lotse projekt zeige Gartenhaus
./target/debug/lotse scan ~/Projekte
./target/debug/lotse tresor add "Fritzbox" --projekt Gartenhaus passwort=xyz
./target/debug/lotse tresor zeige Fritzbox passwort
./target/debug/lotse export bundle backup.json.age
./target/debug/lotse gegenseite --trocken
./target/debug/lotse termine --tage 30
./target/debug/lotse update
```

`gegenseite` sucht zu jedem Projekt ein Repo bei GitHub oder GitLab – entweder aus einer
eingetragenen Adresse oder aus dem Git-Remote eines Ordners, den das Projekt schon kennt –
und schreibt eine verdichtete Zeile ins Logbuch: offene Pull- bzw. Merge-Requests, Zahl
offener Issues, Zustand des Prüflaufs. Issues werden dabei **nicht** zu offenen Fäden.
Ein Token ist nur für private Repos und ein größeres Kontingent nötig: entweder in
`LOTSE_FORGE_TOKEN` oder als Tresor-Eintrag, den die Desktop-App unter *Einstellungen →
GitHub und GitLab* hinterlegt.

`termine` liest die Kalender, die als Referenz am Projekt hängen (Typ `url` oder `datei`,
Adresse auf `.ics` oder `webcal://`), und zeigt, was ansteht. Geschrieben wird dabei
nichts: Termine bleiben im Kalender.

`update` sieht bei den Veröffentlichungen dieses Projekts nach, ob es eine neuere
Version gibt, und nennt die Datei für dieses System. Heruntergeladen und installiert wird
nichts von allein – die Installer sind unsigniert, deshalb bleibt der letzte Schritt
bewusst beim Menschen. Die Desktop-App zeigt denselben Hinweis beim Entsperren, höchstens
einmal am Tag, abschaltbar unter *Einstellungen → Version und Updates*.

`init` zeigt einmalig den Wiederherstellungscode und den Desktop-Schlüssel. Beides gehört
in den Passwortmanager. Einträge der Stufe »nur Desktop« brauchen den Desktop-Schlüssel
(aus dem OS-Schlüsselbund, sonst Umgebungsvariable `LOTSE_DESKTOP_KEY`).

Das age-Bundle lässt sich ohne Lotse entschlüsseln: `age -d -o bundle.json backup.json.age`.

## Abgleich zwischen Geräten

```
# Erstes Gerät: Konto beim Dienst registrieren (fragt den Wiederherstellungscode ab)
lotse sync register --url https://api.example.invalid --email du@example.invalid
# Weiteres Gerät: anmelden, alles herunterladen
lotse sync login --url https://api.example.invalid --email du@example.invalid --geraet "Laptop"
lotse sync jetzt      # pushen, dann pullen
lotse sync status     # lokal und entfernt
lotse sync geraete    # Geräte des Kontos, `lotse sync widerrufen <id>` entzieht eines
```

Lokal testen: `scripts/sync-e2e.sh` startet den Dienst mit `wrangler dev` und lässt den
End-to-End-Test des Kerns dagegen laufen. Der Dienst sieht dabei nur Umschläge.

## KI-Assistenten anbinden (MCP)

`lotse mcp` spricht das Model Context Protocol über stdin/stdout. Werkzeuge: `list_projects`,
`get_project_context`, `log_activity`, `list_open_threads`, `search`. Der Tresor ist über
diese Schnittstelle nicht erreichbar. Für Claude Code:

```
claude mcp add lotse --env LOTSE_HOME=$HOME/.lotse --env LOTSE_PASSWORD=… -- lotse mcp
```

Das Passwort in der Umgebung ist ein Kompromiss für die Kommandozeile; die Desktop-App wird
den Server aus der entsperrten Sitzung heraus starten.

## Landing Page und Veröffentlichungen

`site/` ist die Landing Page (eine HTML-Datei ohne Fremdressourcen). Der Workflow
`.github/workflows/pages.yml` veröffentlicht sie auf GitHub Pages, sobald in den
Repo-Einstellungen unter *Pages* die Quelle *GitHub Actions* gewählt ist. Auf einem
kostenlosen GitHub-Konto muss das Repository dafür öffentlich sein; alternativ lässt sich
`site/` unverändert auf Cloudflare Pages legen.

`.github/workflows/release.yml` baut bei einem Tag `v*` die Kommandozeile und die
Desktop-Installer für Windows, macOS und Linux und hängt sie an ein GitHub-Release. Die
Landing Page zeigt die neueste Veröffentlichung automatisch an. Builds sind unsigniert;
solange das so ist, sagt die App nur Bescheid, statt sich selbst zu ersetzen. Für einen
selbsttätigen Updater fehlt genau ein Stück: ein Signaturschlüsselpaar
(`npm exec tauri signer generate`), der öffentliche Teil in `tauri.conf.json`, der private
als Repository-Secret `TAURI_SIGNING_PRIVATE_KEY` für `release.yml`.

```
git tag v0.1.0 && git push origin v0.1.0
```

App-Icons entstehen aus `apps/desktop/src-tauri/icons/icon.svg` mit `scripts/make-icons.sh`
(braucht ein headless Chromium und Python 3).

## Aufbau

Siehe `CLAUDE.md` für die Verzeichnisstruktur und die Prüfbefehle.

## Lizenz

Noch nicht festgelegt. Alle Rechte vorbehalten, bis `docs/BUSINESS.md` Abschnitt 3
entschieden ist.
