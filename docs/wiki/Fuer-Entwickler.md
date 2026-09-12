# Für Entwickler

## Aufbau

| Pfad | Inhalt |
|---|---|
| `crates/lotse-core` | Der Kern: Modell, Krypto, Tresor, Speicher (SQLCipher), Sync-Umschläge, Erkennung, Git-Reader, Gegenseite, Kalender, Dateiauszug, KI-Ziel, MCP, Export. |
| `crates/lotse-cli` | Binary `lotse`. |
| `apps/web` | Svelte 5 + Vite. Läuft in Tauri **und** im Browser. |
| `apps/desktop` | Tauri-2-Hülle. Braucht GTK/WebKit zum Bauen, wird nur in `release.yml` gebaut. |
| `services/sync-worker` | Cloudflare Worker (TypeScript) + D1 + R2. |
| `site` | Landing Page: eine HTML-Datei, inline CSS/JS, keine Fremdressourcen. |
| `docs/wiki` | **Dieses Wiki.** Wird von `wiki.yml` ins GitHub-Wiki gespiegelt. |

### Die drei Schichten

```
lotse-core        ← alles Fachliche, keine Oberfläche
   ↑         ↑
lotse-cli   apps/desktop (Tauri)
                  ↑
              apps/web (Svelte)
```

`apps/web/src/lib/data/provider.ts` ist die Schnittstelle zwischen Oberfläche und
Daten. Es gibt zwei Umsetzungen: `tauri.ts` spricht den Kern, `mock.ts` liefert
Beispieldaten im Browser. **Zeitstempel werden ausschließlich in `tauri.ts`** von
Millisekunden zu ISO umgerechnet – eine Stelle, nicht zwanzig.

Die Kommandos in `apps/desktop/src-tauri/src/lib.rs` spiegeln `provider.ts`. Wer dort
etwas hinzufügt, fügt es an beiden Stellen hinzu.

### Feature `native`

`lotse-core` baut mit dem Feature `native` (Standard) alles mit Dateisystem und Netz.
**Ohne** das Feature bleibt der Kern WebAssembly-tauglich – Modell, Krypto und
Sync-Umschläge. Das ist die Vorbereitung für einen Browser-Client.

Deshalb steht in der Prüfliste:

```bash
cargo check -p lotse-core --no-default-features --target wasm32-unknown-unknown
```

## Bauen

```bash
# Kern und CLI
cargo build --release -p lotse-cli

# Oberfläche
cd apps/web && npm ci && npm run dev     # läuft im Browser mit Beispieldaten

# Desktop (braucht GTK/WebKit unter Linux)
cd apps/desktop && npm ci && npm run tauri dev
```

Unter Ubuntu für die Hülle:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev patchelf libssl-dev pkg-config
```

## Prüfen vor jedem Commit

Das ist die verbindliche Liste aus `CLAUDE.md`:

```bash
scripts/modulgrenzen.sh
scripts/wiki_pruefen.sh
cargo deny check
cargo deny --manifest-path apps/desktop/src-tauri/Cargo.toml check
cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
scripts/sync-e2e.sh
cargo check -p lotse-core --no-default-features --target wasm32-unknown-unknown
(cd apps/web && npm run check && npm test && npm run build)
(cd services/sync-worker && npm run typecheck && npm test)
```

### Warum `modulgrenzen.sh`

Es prüft, dass bestimmte Module **keinen Zugriffsweg** zum Tresor haben. Das ist die
wichtigste Regel im Projekt, und sie ist mechanisch geprüft statt nur verabredet:

`ai` · `detect` · `dokument` · `export::spiegel` · `forge` · `git` · `kalender` ·
`mcp` · `netz` · `update` · `watcher`

Das Skript folgt einem Modul auch in einen Unterordner und liest eingebettete Module per
Klammerzählung (`export::bundle` in derselben Datei *darf* den Tresor sehen,
`export::spiegel` nicht). Verschwindet ein Name aus der Liste, schlägt es an – sonst
verschwände die Regel unbemerkt mit ihm.

## Regeln, die aus dem Bedrohungsmodell folgen

- **Keine eigenen Krypto-Primitive.** Nur RustCrypto, `age`, `zeroize`. Änderungen an der Komposition erhöhen `FORMAT_VERSION` und werden in `THREAT_MODEL.md` protokolliert.
- **Schlüssel sind `Key32`** (überschreiben beim Wegwerfen). Kein Schlüssel als `Vec<u8>` oder `String`.
- **Alles Netz geht über `netz.rs`** – ein Agent, eine Fehlerübersetzung.
- **Keine Fremdskripte, CDNs, Fonts oder Analytics** in `apps/web` und `site`. Kein `{@html}` für Nutzertext.
- **Der Sync-Dienst loggt nie Bodies oder Tokens.**
- `site/index.html` behauptet nur, was im Kern läuft und getestet ist.

## Konventionen

- **Feldnamen im Modell sind deutsch** und entsprechen `CONCEPT.md` Abschnitt 3. Sync-Umschläge und API-Feldnamen sind **englisch** (`SYNC_PROTOCOL.md`).
- Zeitstempel sind Unix-Millisekunden (`i64`), Datumsangaben `JJJJ-MM-TT`.
- Nutzertexte auf Deutsch; nur die fünf Metaphern **Hafen, Kurs, Logbuch, vor Anker, Hafeneinfahrt**.
- Neue Features müssen den Aufnahmetest in `NON_GOALS.md` bestehen.
- Die Lizenz ist **AGPL-3.0-only** (`LICENSE`) – **keine Lizenz-Header in einzelne Dateien einfügen**.
  Sie steht einmal im Repo und in den Cargo-Manifesten.

## Ein Release bauen

1. Version an **fünf** Stellen erhöhen: `Cargo.toml`, `apps/desktop/src-tauri/Cargo.toml`, `apps/desktop/src-tauri/tauri.conf.json`, `apps/web/package.json`, `apps/desktop/package.json`.
2. Lockfiles nachziehen: `cargo update -w --offline` (auch in `apps/desktop/src-tauri`).
3. Commit „Version X.Y.Z“, nach `main`.
4. Actions → *Release* → *Run workflow* mit dem Tag `vX.Y.Z`. Der Workflow legt den Tag selbst an.

Der Workflow prüft zuerst, ob der **Update-Schlüssel** hinterlegt ist und ob in
`tauri.conf.json` ein echter öffentlicher Schlüssel steht. Fehlt eines, bricht er ab –
ein Release ohne Signatur wäre eine Sackgasse, weil der Updater es abweisen würde.

Danach spiegelt ein Job die fertige `latest.json` nach `site/`, von wo GitHub Pages sie
ausliefert. Siehe **[[Updates]]**.

## Das Wiki ändern

Der Text liegt in `docs/wiki/` im Hauptrepository, eine Markdown-Datei je Seite.
`.github/workflows/wiki.yml` schreibt sie ins GitHub-Wiki, sobald auf `main` etwas darunter
geändert wird. Änderungen im Browser werden dabei überschrieben.

```
scripts/wiki_pruefen.sh
```

liest die tatsächliche Oberfläche aus dem Quelltext – Kommandobaum, Tauri-Kommandos,
MCP-Werkzeuge, Modellfelder, Fehlertexte, Einstellungsschlüssel, `LOTSE_*`-Variablen,
Sync-Endpunkte, Erkennungsmarken – und verlangt, dass jedes Stück davon im Wiki vorkommt.
Sie läuft in CI. Wer ein Feld hinzufügt und das Wiki nicht anfasst, bekommt eine rote CI.

Vollständig: [[Wiki pflegen|Wiki-pflegen]].

## Beitragen

Issues und Pull Requests sind willkommen. Zwei Bitten:

1. **Lies `NON_GOALS.md` zuerst.** Ein Feature, das den Aufnahmetest nicht besteht, wird abgelehnt – auch wenn es gut gebaut ist.
2. **Die Prüfliste muss grün sein.** CI prüft dasselbe, aber lokal geht es schneller.

Die Lizenz ist **AGPL-3.0-only**. Beiträge sind willkommen, brauchen aber ein **DCO**
(`Signed-off-by:` in der Commit-Nachricht) – ohne das wäre der kommerzielle Doppelvertrieb
dahin, auf dem das Geschäftsmodell beruht (`BUSINESS.md` 3). Bei größeren Änderungen vorher
ein Issue, damit niemand umsonst arbeitet.
