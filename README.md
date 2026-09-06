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
| Sync-Dienst (Cloudflare Worker) | Gerüst, siehe `services/sync-worker/README.md` |
| Desktop-Hülle (Tauri 2) | Gerüst, noch nicht gebaut |
| Ordner-Beobachter, MCP-Server, Netzwerk-Sync im Client | offen |

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
(CLI: Umgebungsvariable `LOTSE_DESKTOP_KEY`; die Desktop-App nutzt den OS-Schlüsselbund).

Das age-Bundle lässt sich ohne Lotse entschlüsseln: `age -d -o bundle.json backup.json.age`.

## Aufbau

Siehe `CLAUDE.md` für die Verzeichnisstruktur und die Prüfbefehle.

## Lizenz

Noch nicht festgelegt. Alle Rechte vorbehalten, bis `docs/BUSINESS.md` Abschnitt 3
entschieden ist.
