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
| Desktop-Hülle (Tauri 2) | läuft. Anlegen, erfassen, Fäden abhaken, Status mit Übergabe, Projektkopf bearbeiten, Referenzen anlegen/prüfen/öffnen, Tresor lesen/anlegen/löschen, Ordner scannen und laufend beobachten, Export (Spiegel und age-Bundle), Abgleich mit Geräteverwaltung, Passwortwechsel und Konto-Wiederherstellung. Offen: Tray, Auto-Lock, signierter Updater, MCP aus der entsperrten Sitzung |
| Landing Page (`site/`) | fertig, Deploy per GitHub Pages |
| Ordner-Beobachter | läuft, in der CLI (`lotse beobachten`) und in der Desktop-App |
| MCP-Server (`lotse mcp`) | läuft |
| Remote-Git (GitHub: offene PRs, Issue-Zahl, Prüflauf) | läuft im Kern und in der App; GitLab folgt |
| WebAssembly-Client für den Browser | offen |

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
```

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
Landing Page zeigt die neueste Veröffentlichung automatisch an. Builds sind unsigniert.

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
