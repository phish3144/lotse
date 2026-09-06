# Lotse Desktop (Tauri 2)

Gerüst. Die Oberfläche kommt aus `apps/web`, der Kern aus `crates/lotse-core`.

## Bauen

Voraussetzungen siehe https://v2.tauri.app/start/prerequisites/ (Linux: `libwebkit2gtk-4.1-dev`,
`libgtk-3-dev`, `libayatana-appindicator3-dev`; macOS: Xcode CLT; Windows: WebView2).

```
cd apps/web && npm install
cargo install tauri-cli --version "^2"
cd ../desktop/src-tauri && cargo tauri dev
```

## Stand

- Kommandos für Hafen, Projekt, Notizen, Statuswechsel, offene Fäden, Suche sind angelegt
  und rufen direkt `lotse-core` auf.
- Offen: Entsperren (Master-Passwort, OS-Schlüsselbund), Tresor-Kommandos, Ordner-Beobachter,
  Sync-Client, Tray, Auto-Lock, signierter Updater, Icons.
- Das Crate ist bewusst kein Workspace-Mitglied, bis die CI die GTK/WebKit-Abhängigkeiten hat.
