# Entwurf der Oberfläche

Arbeitsdateien für den Neuentwurf (Artefakt: „Lotse – Oberfläche").
Vier Artboards: Projektseite als Arbeitsfläche, Hafen, Einstellungen, Palette.

| Datei | Inhalt |
|---|---|
| `_basis.css` | Palette und Bausteine, aus `apps/desktop/src-tauri/icons/icon.svg` abgeleitet |
| `_<Name>.body.html`, `_<Name>.css` | Quellen je Artboard |
| `_leiste.html` | Kopfleiste, aus `_Main.body.html` gezogen und geteilt |
| `_bau.sh` | setzt Quellen zu `<Name>.dc.html` zusammen |
| `canvas.json` | Anordnung und Notizen auf der Leinwand |

Ändern heißt: `_*`-Quelle bearbeiten, `./_bau.sh <Name>` je Artboard, neu zusammensetzen.
Die zusammengesetzte Datei steht in `.gitignore` – sie ist erzeugt, nicht gepflegt.

Die Palette ist seit dem Umbau in `apps/web/src/app.css` umgesetzt; die Artboards sind
damit Vorlage und Gedächtnis, nicht mehr Vorschlag. Weicht der Entwurf von der App ab,
gilt die App.
