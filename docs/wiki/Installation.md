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

Neben jeder Datei liegt eine `.sha256`-Prüfsumme. Vergleichen lohnt sich:

```bash
# Linux/macOS
shasum -a 256 -c Lotse_0.8.0_amd64.deb.sha256

# Windows (PowerShell) – und mit der .sha256-Datei daneben vergleichen
Get-FileHash .\Lotse_0.8.0_x64-setup.exe -Algorithm SHA256
```

Nicht zu verwechseln mit den `.sig`-Dateien, die ebenfalls dabeiliegen: das ist die
**Update-Signatur**, die die App beim Selbsttausch prüft, keine Prüfsumme zum Vergleichen
von Hand. Und die Prüfsumme liegt auf demselben Server wie die Datei – sie findet einen
kaputten Download, keinen bösen Server. Gegen den hilft die Update-Signatur, deren
öffentliche Hälfte fest in der schon installierten App steckt.

> Bei **0.8.0** tragen nur die vier Kommandozeilen-Archive eine `.sha256`-Datei; für die
> Installer sind die Summen nachträglich gebildet worden und stehen im Release-Text. Ab
> 0.9.0 legt der Bau-Job sie für jeden Installer selbst mit an.

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

### Empfohlen: die Paketquelle

Damit aktualisiert `apt upgrade` Lotse zusammen mit allem anderen – kein Knopf, kein
Download, keine Handarbeit. Einmal einrichten:

**Debian, Ubuntu, Mint**

```bash
curl -fsSL https://lotse.sanctora.eu/lotse-archiv.gpg \
  | sudo tee /usr/share/keyrings/lotse-archiv.gpg > /dev/null
echo "deb [signed-by=/usr/share/keyrings/lotse-archiv.gpg] https://lotse.sanctora.eu/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/lotse.list
sudo apt update && sudo apt install lotse
```

**Fedora, openSUSE, RHEL**

```bash
sudo curl -fsSL -o /etc/yum.repos.d/lotse.repo https://lotse.sanctora.eu/rpm/lotse.repo
sudo dnf install lotse
```

Die Quelle ist signiert; eine veränderte Quelle lehnt `apt` mit `BADSIG` ab. In ihr steht
immer die **neueste** Fassung – zum Aktualisieren genügt das, eine ältere Fassung gezielt
zu installieren geht darüber nicht.

### Oder einzelne Dateien

| Datei | Aktualisiert sich | Wann |
|---|---|---|
| Paketquelle (oben) | **von selbst**, über `apt`/`dnf` | **Empfohlen** |
| `Lotse_<version>_amd64.AppImage` | **aus der App heraus** | Ohne Paketverwaltung, ohne Root, tragbar – auch vom Stick. |
| `Lotse_<version>_amd64.deb` | über `apt`, aber ohne Quelle von Hand | Wenn du die Quelle nicht willst. |
| `Lotse-<version>-1.x86_64.rpm` | dito über `dnf` | |

Aus `.deb` und `.rpm` heraus aktualisiert die **Paketverwaltung**, nicht Lotse. Die App
merkt, woher sie kommt, versucht es gar nicht erst und nennt stattdessen den Befehl, der
es wirklich tut. Sich dort selbst zu überschreiben würde die Buchführung der
Paketverwaltung zerreißen – und eine heruntergeladene Datei danebenzulegen ergäbe zwei
Installationen statt einer Aktualisierung.

AppImage startbar machen:

```bash
chmod +x Lotse_*_amd64.AppImage
./Lotse_*_amd64.AppImage
```

### Platz im System

Ein AppImage wird nicht installiert. Es liegt da, wo der Browser es hingelegt hat, und
daraus folgen drei Ärgernisse auf einmal:

- kein Eintrag im Anwendungsmenü, kein Icon
- das Selbst-Update arbeitet an genau dieser Datei – wird der Download-Ordner aufgeräumt,
  ist Lotse weg
- der Dateiname trägt die Version: nach dem ersten Selbst-Update heißt die Datei
  `Lotse_0.7.0_amd64.AppImage` und enthält 0.8.0

Beim ersten Start fragt Lotse deshalb **einmal**, ob es sich einrichten darf. Ein »nein«
wird gemerkt; der Knopf bleibt unter *Einstellungen → Version → Platz im System*.

Was dabei passiert – und nichts darüber hinaus:

| Datei | Was |
|---|---|
| `~/.local/share/lotse/Lotse.AppImage` | Die Fassung, die von jetzt an gilt. Fester Name ohne Version, damit das Selbst-Update verlässlich greift und der Name nicht lügt. |
| `~/.local/share/applications/lotse.desktop` | Der Eintrag im Menü. `Exec` zeigt auf die Datei oben. |
| `~/.local/share/icons/hicolor/128x128/apps/lotse.png` | Das Icon. |

Die heruntergeladene Datei bleibt liegen – Lotse löscht nichts im Download-Ordner; die
Anzeige nennt sie, damit du sie selbst wegräumen kannst. Nach dem Einrichten läuft noch
die alte Datei; *Von dort neu starten* wechselt (und sperrt vorher, weil der Schlüssel
einen Neustart nicht überlebt).

**Deinstallieren** heißt dann: die drei Dateien oben löschen, dazu den Datenordner
([[Datenablage]]). Der Eintrag im Menü lässt sich auch aus den Einstellungen wieder
wegnehmen, ohne dass das Programm verschwindet.

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


---

## Die Paketquelle betreiben

*Für den Betrieb, nicht für die Benutzung.* Die Quelle entsteht in
`.github/workflows/pages.yml` beim Ausliefern der Seite und liegt nur im Seiten-Bündel –
nicht im Repository. Ein `.deb` und ein `.rpm` sind zusammen rund 25 MB, und die blieben
sonst für immer in der Git-Geschichte.

Sie braucht **einen GPG-Signierschlüssel**, getrennt vom minisign-Schlüssel des Updaters.
Einmalig:

```bash
gpg --quick-generate-key "Lotse Paketquelle <paket@sanctora.eu>" rsa4096 sign never
gpg --armor --export-secret-keys paket@sanctora.eu
```

Die Ausgabe kommt als Repository-Geheimnis `APT_GPG_KEY` hinterlegt; hat der Schlüssel
eine Passphrase, zusätzlich `APT_GPG_PASSPHRASE`. **Ohne das Geheimnis wird die Quelle
übersprungen** und die Seite trotzdem ausgeliefert – eine fehlende Paketquelle ist ein
Mangel, eine fehlende Landing Page ein Ausfall.

Örtlich ausprobieren, ohne etwas zu veröffentlichen:

```bash
scripts/paketquelle-bauen.sh <ordner-mit-paketen> <ausgabe> https://lotse.sanctora.eu
```
