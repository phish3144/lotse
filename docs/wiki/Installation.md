# Installation

Lotse gibt es als **Desktop-App** (Windows, macOS, Linux) und als **Kommandozeile**.
Beide arbeiten auf demselben Datenbestand und können parallel benutzt werden.

Alle Dateien liegen bei den [Veröffentlichungen](https://github.com/phish3144/lotse/releases).

---

## Vorher: die Warnung beim ersten Start

Die Installer sind **nicht code-signiert**. Code-Signaturzertifikate für Windows und
macOS kosten Geld und laufende Arbeit; das Geld steckt im Moment in der Anwendung.
Deshalb warnen beide Systeme einmal:

- **Windows**: „Der Computer wurde durch Windows geschützt.“ → *Weitere Informationen* → *Trotzdem ausführen*
- **macOS**: „…kann nicht geöffnet werden, da der Entwickler nicht verifiziert werden kann.“ → Rechtsklick auf die App → *Öffnen* → *Öffnen*

Das ist **etwas anderes** als die Update-Signatur: jede Fassung wird mit einem
Schlüssel unterschrieben, dessen öffentliche Hälfte in der App steckt, und nur was dazu
passt, installiert Lotse als Update. Siehe **[[Updates]]** und **[[Sicherheit]]**.

Neben jedem Archiv liegt eine `.sha256`-Prüfsumme. Vergleichen lohnt sich:

```bash
# Linux/macOS
shasum -a 256 -c lotse-cli-v0.6.0-x86_64-unknown-linux-gnu.tar.gz.sha256

# Windows (PowerShell)
Get-FileHash .\lotse-cli-v0.6.0-x86_64-pc-windows-msvc.zip -Algorithm SHA256
```

---

## Windows

| Datei | Wann |
|---|---|
| `Lotse_<version>_x64-setup.exe` | **Empfohlen.** Normaler Installer, kann sich selbst aktualisieren. |
| `Lotse_<version>_x64_en-US.msi` | Für zentrale Verteilung (Gruppenrichtlinie, Intune). |

Voraussetzung: Windows 10 oder 11, 64 Bit. WebView2 ist auf aktuellen Systemen
vorhanden; falls nicht, holt der Installer es nach.

## macOS

| Datei | Wann |
|---|---|
| `Lotse_<version>_aarch64.dmg` | Apple Silicon (M1 und neuer) |
| `Lotse_<version>_x64.dmg` | Intel-Macs |

Im Zweifel:  → *Über diesen Mac* → steht dort „Apple M…“, nimm `aarch64`.

> In 0.6.0 fehlt `x64.dmg`: der Release-Workflow hat beide Mac-Bauten auf Apple Silicon
> gestellt. Ab 0.6.1 liegen wieder beide bei.

Nach dem Öffnen des DMG die App nach *Programme* ziehen. **Wichtig für Updates:** Lotse
muss in einem beschreibbaren Ordner liegen – von `/Programme` aus geht der Austausch,
direkt aus dem DMG heraus nicht.

## Linux

Hier lohnt sich die Wahl, weil sie darüber entscheidet, ob Lotse sich selbst
aktualisieren kann:

| Datei | Selbst-Update | Wann |
|---|---|---|
| `Lotse_<version>_amd64.AppImage` | **ja** | **Empfohlen**, wenn du Updates aus der App willst. Keine Installation nötig. |
| `Lotse_<version>_amd64.deb` | nein | Debian, Ubuntu, Mint – wenn dir Paketverwaltung wichtiger ist. |
| `Lotse-<version>-1.x86_64.rpm` | nein | Fedora, openSUSE, RHEL. |

Aus `.deb` und `.rpm` heraus aktualisiert die **Paketverwaltung**, nicht Lotse. Die App
merkt das (am fehlenden `APPIMAGE` in der Umgebung), versucht es gar nicht erst und
zeigt stattdessen den Download-Weg. Sich dort selbst zu überschreiben würde die
Buchführung der Paketverwaltung zerreißen.

AppImage startbar machen:

```bash
chmod +x Lotse_*_amd64.AppImage
./Lotse_*_amd64.AppImage
```

## Kommandozeile

Vier Ziele, jeweils als Archiv mit Prüfsumme:

```bash
# Beispiel Linux
tar xzf lotse-cli-v0.6.0-x86_64-unknown-linux-gnu.tar.gz
sudo install -m 755 lotse-cli-*/lotse /usr/local/bin/lotse
lotse --version
```

Auf macOS dasselbe; unter Windows die `.exe` aus dem ZIP an einen Ort im `PATH` legen.

Im Archiv liegen außerdem `README.md`, `CONCEPT.md` und `THREAT_MODEL.md` – damit die
Unterlagen auch dann da sind, wenn das Netz nicht da ist.

---

## Wo die Daten liegen

Ein Ordner pro Rechner, standardmäßig im Nutzer-Datenverzeichnis:

| System | Pfad |
|---|---|
| Linux | `~/.local/share/lotse/` |
| macOS | `~/Library/Application Support/lotse/` |
| Windows | `%APPDATA%\lotse\` |

Darin: die verschlüsselte Datenbank (SQLCipher) und `konto.json` mit den
Schlüsselableitungs-Parametern – **keine** Schlüssel im Klartext.

Ein anderer Ort geht über `--home` oder die Umgebungsvariable `LOTSE_HOME`:

```bash
LOTSE_HOME=~/Cloud/lotse lotse hafen
```

> **Achtung:** Lege den Datenordner nicht in einen Cloud-Ordner, der zwischen Geräten
> synchronisiert wird. Zwei Rechner, die gleichzeitig auf dieselbe SQLite-Datei
> schreiben, beschädigen sie. Für mehrere Geräte gibt es den **[[Abgleich]]**.

## Deinstallieren

Die App wie jedes andere Programm entfernen. Der Datenordner bleibt – er gehört dir.
Wenn er auch weg soll, lösche ihn von Hand. Vorher lohnt ein **[[Export und
Fluchtweg|Export-und-Fluchtweg]]**.
