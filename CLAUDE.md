# Lotse – Hinweise für die Arbeit im Repo

Lotse ist das Logbuch für alle Vorhaben einer Person. Das verbindliche Konzept steht in
`docs/CONCEPT.md`; Sicherheit in `docs/THREAT_MODEL.md`; was bewusst nicht gebaut wird in
`docs/NON_GOALS.md`; die Sync-Schnittstelle in `docs/SYNC_PROTOCOL.md`; der geplante
Web-Client in `docs/WEB_CLIENT.md`.

## Aufbau

| Pfad | Inhalt |
|---|---|
| `crates/lotse-core` | Rust-Kern: Modell, Krypto, Tresor, Sync-Umschläge, SQLCipher-Speicher, Erkennung, Git-Reader, Gegenseite (GitHub/GitLab), Kalender, Dateiauszug, KI-Ziel, Export. Alles Netz geht über `netz.rs` (ein Agent, eine Fehlerübersetzung). Feature `native` (Default) für alles mit Dateisystem; ohne Feature WASM-tauglich. |
| `crates/lotse-wasm` | WebAssembly-Anbindung: Krypto, Umschläge, Logikuhr, Brief, Tresor Stufe `ueberall`. **Nur Adapter, keine Logik** – wer hier entscheidet, baut die zweite Wahrheit. Schlüssel sind undurchsichtige Griffe, nur der `auth_key` darf heraus. Gebaut mit `scripts/wasm-bauen.sh`. |
| `crates/lotse-cli` | Binary `lotse`: Schnellerfassung, Projekte, Tresor, Scan, Export. |
| `apps/web` | Svelte 5 + Vite Oberfläche, läuft in Tauri und im Browser. `src/lib/data/provider.ts` ist die Schnittstelle; `tauri.ts` spricht den Kern, `mock.ts` liefert Beispieldaten im Browser. Zeitstempel werden nur in `tauri.ts` von Millisekunden zu ISO umgerechnet. |
| `apps/desktop` | Tauri-2-Hülle: Kommandos in `src-tauri/src/lib.rs` spiegeln `apps/web/src/lib/data/provider.ts`. Braucht GTK/WebKit zum Bauen, wird nur in `release.yml` gebaut. |
| `services/sync-worker` | Cloudflare Worker (TypeScript) + D1 + R2, implementiert `SYNC_PROTOCOL.md`. |
| `site` | Landing Page: eine HTML-Datei, inline CSS/JS, keine Fremdressourcen. Deploy über `.github/workflows/pages.yml`. |
| `scripts` | `make-icons.sh` und `pack-icons.py` erzeugen die Tauri-Icons aus `icon.svg`. |

## Prüfen vor jedem Commit

```
scripts/modulgrenzen.sh   # kein Tresor-Zugriff aus den Modulen, die ihn nicht haben dürfen
scripts/wiki_pruefen.sh   # docs/wiki/ deckt noch jedes Kommando, Feld und Schalter ab
cargo deny check          # Sicherheitsmeldungen und Lizenzen der Abhängigkeiten
                          # npm-Seite: siehe docs/ABHAENGIGKEITEN.md (CI prüft --omit=dev)
cargo deny --manifest-path apps/desktop/src-tauri/Cargo.toml check   # dasselbe für die Hülle
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
scripts/sync-e2e.sh   # Sync-Client gegen wrangler dev
cargo check -p lotse-core --no-default-features --target wasm32-unknown-unknown
cargo clippy -p lotse-wasm --target wasm32-unknown-unknown -- -D warnings
scripts/wasm-bauen.sh && (cd apps/web && node scripts/wasm-browsertest.mjs)   # braucht wasm-bindgen
                          # und einen Chromium/Chrome; derselbe Vektor wie tests/vektoren/kdf.json
(cd apps/web && node scripts/wasm-mengentest.mjs 2000)   # Mengen-Sicherung; große Läufe auf Abruf:
                          # node scripts/wasm-mengentest.mjs 500000 (siehe WEB_CLIENT.md 4a)
(cd apps/web && npm run check && npm test && npm run build)
(cd services/sync-worker && npm run typecheck && npm test)
```

## Regeln, die aus dem Bedrohungsmodell folgen

- `ai`, `detect`, `dokument`, `export::spiegel`, `forge`, `git`, `kalender`, `mcp` (samt
  `mcp::dienst`), `netz`, `systemeintrag`, `update` und `watcher` importieren **nie** aus `vault`. Nur `export::bundle`, die Oberfläche und die CLI dürfen Tresor-Werte lesen.
  `scripts/modulgrenzen.sh` prüft das und läuft in CI.
- Keine eigenen Krypto-Primitive. Nur RustCrypto, `age`, `zeroize`. Änderungen an der
  Komposition erhöhen `FORMAT_VERSION` und werden in `THREAT_MODEL.md` protokolliert.
  `tests/vektoren/kdf.json` hält die Ableitung mit festen Werten fest – gelesen vom nativen
  Test *und* vom Browsertest. Ändert sich einer der Werte unabsichtlich, ist jedes
  bestehende Konto ausgesperrt; deshalb wird es dort rot und nicht erst beim Nutzer.
- Schlüssel sind `Key32` (zeroize on drop). Kein Schlüssel als `Vec<u8>` oder `String`
  herumreichen.
- Keine Fremdskripte, CDNs, Fonts oder Analytics in `apps/web` und `site`. Kein `{@html}` für Nutzertext.
- `site/index.html` behauptet nur, was in `crates/` läuft und getestet ist; Feature-Stand-Badges
  (läuft / in Arbeit / geplant) bei jeder Änderung am Kern nachziehen.
- Der Sync-Dienst loggt nie Bodies oder Tokens.

## Konventionen

- Feldnamen im Modell sind deutsch und entsprechen `CONCEPT.md` Abschnitt 3. Sync-Umschläge
  und API-Feldnamen sind englisch (`SYNC_PROTOCOL.md`).
- Zeitstempel sind Unix-Millisekunden (`i64`), Datumsangaben `JJJJ-MM-TT`.
- Nutzertexte in der Oberfläche und CLI auf Deutsch; nur die fünf Metaphern Hafen, Kurs,
  Logbuch, vor Anker, Hafeneinfahrt.
- Neue Features müssen den Aufnahmetest in `NON_GOALS.md` bestehen.
- Lizenz ist **AGPL-3.0-only** (`LICENSE`). Keine Lizenz-Header in einzelne Dateien einfügen –
  die Lizenz steht einmal im Repo und in den Cargo-Manifesten. Beiträge Dritter brauchen ein
  DCO oder eine CLA, sonst ist der kommerzielle Doppelvertrieb dahin (`BUSINESS.md` 3).
