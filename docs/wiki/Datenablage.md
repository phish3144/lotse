# Datenablage

Wo Lotse etwas hinschreibt, was in jeder Datei steht, und was davon mitgesichert werden
muss. Nützlich beim Umziehen, beim Sichern und wenn man sicher sein will, dass nichts an
einer unerwarteten Stelle liegt.

---

## Der Datenordner

| System | Pfad |
|---|---|
| Linux | `~/.local/share/lotse` (bzw. `$XDG_DATA_HOME/lotse`) |
| macOS | `~/Library/Application Support/lotse` |
| Windows | `%APPDATA%\lotse` |

Überschreibbar mit `--home <PFAD>` oder `LOTSE_HOME` ([[Umgebungsvariablen]]). App und
Kommandozeile benutzen **denselben** Ordner – zwei Oberflächen, ein Bestand.

Darin liegen genau zwei Dateien:

```
konto.json      582 Byte
lotse.db        die Datenbank
```

### `konto.json`

Der Konto-Kopf. Enthält den Salt, die KDF-Parameter, die **gewrappten** Schlüssel und
die Geräte-ID. Keine Geheimnisse im Klartext: ohne Master-Passwort ist die Datei
nutzlos. Sie darf in eine Sicherung, und sie muss dort hinein – ohne sie ist die
Datenbank nicht aufzuschließen.

### `lotse.db`

SQLCipher: eine SQLite-Datenbank, deren Seiten verschlüsselt sind. Der Schlüssel wird
aus dem Konto-Schlüssel abgeleitet (HKDF, `lotse/local-db`) und existiert nur im
Speicher der laufenden Anwendung. Ein `sqlite3 lotse.db` liefert *file is not a
database* – das ist richtig so.

---

## Was in der Datenbank steht

| Tabelle | Inhalt | Wird abgeglichen |
|---|---|---|
| `projekte` | Vorhaben | ja |
| `notizen` | Logbuch-Einträge | ja |
| `referenzen` | Zeiger nach draußen | ja |
| `tresor` | verschlüsselte Zugangsdaten | ja (als Ciphertext) |
| `geraete` | Geräte am Konto | ja |
| `aenderungen` | Änderungsprotokoll, Quelle der Sync-Umschläge | nein (ist der Absender) |
| `kandidaten` | Hafeneinfahrt | **nein** – Pfade gelten nur auf diesem Gerät |
| `suche` | FTS5-Volltextindex | nein (wird lokal aufgebaut) |
| `meta` | Einstellungen dieses Geräts | **nein** |

Dazu drei Indizes: `notizen_projekt` (Projekt und Zeit), `notizen_offen` (nur offene
Fäden – ein Teilindex, damit [[Offene Punkte|Offene-Punkte]] auch bei zehntausend
Notizen sofort antwortet) und `referenzen_projekt`.

Der Volltextindex enthält **keine** Tresor-Inhalte, weder Werte noch Titel. Die Suche
kann also gar nichts Vertrauliches finden, weil nichts Vertrauliches darin steht.

### Warum `meta` nicht abgeglichen wird

Dort stehen Dinge, die auf einem anderen Gerät falsch wären: welche Ordner beobachtet
werden, welcher Port der MCP-Zugang hat, welches KI-Ziel eingestellt ist. Ein
Handy hat diese Ordner nicht und keinen Port zu vergeben.

---

## Die Einstellungsschlüssel in `meta`

Alles, was die App unter [[Einstellungen]] speichert. Die Werte sind Text.

### Ordner und Beobachter

| Schlüssel | Inhalt |
|---|---|
| `watch_wurzeln` | Die gemerkten Wurzelordner, die beobachtet und durchsucht werden. |
| `watch_git_stand` | Je Repo der letzte gesehene Commit, damit der Beobachter nur Neues schreibt. |

### Verbindungen zur Gegenseite

| Schlüssel | Inhalt |
|---|---|
| `forge_token_eintrag` | ID des Tresor-Eintrags mit dem GitHub-Token. |
| `forge_token_feld` | Welches Feld darin das Token ist. |
| `forge_token_gitlab_eintrag` | Dasselbe für GitLab – getrennt, damit ein GitHub-Token nie an GitLab geht. |
| `forge_token_gitlab_feld` | |
| `forge_auto` | Ob beim Öffnen eines Vorhabens von allein abgefragt wird. |
| `forge_zuletzt` | Wann zuletzt abgefragt wurde. Verhindert, dass jedes Öffnen eine Anfrage auslöst. |

Der Token selbst steht **nicht** hier, sondern im Tresor. In `meta` liegt nur ein
Zeiger. Siehe [[Gegenseite]].

### KI

| Schlüssel | Inhalt |
|---|---|
| `ki_basis_url` | Adresse des Ziels, z. B. `http://localhost:11434/v1`. |
| `ki_modell` | Modellname. |
| `ki_schluessel_eintrag` | ID des Tresor-Eintrags mit dem API-Schlüssel. |
| `ki_schluessel_feld` | Welches Feld darin. |
| `ki_protokoll` | Protokoll der Anfragen: wann, wozu, wie viele Zeichen. |
| `ki_anfragen` | Zähler der Anfragen. |
| `ki_eingabe_token` | Summe der gesendeten Token. |
| `ki_ausgabe_token` | Summe der empfangenen Token. |

Die drei Zähler sind da, damit »was kostet das eigentlich« beantwortbar ist, bevor die
Rechnung kommt. Siehe [[KI]].

### Assistenten

| Schlüssel | Inhalt |
|---|---|
| `mcp_token` | Das Bearer-Token für den HTTP-Zugang. Wird bei »Token erneuern« ersetzt. |
| `mcp_port` | Der gebundene Port, damit der Assistent ihn nach einem Neustart wiederfindet. |

### Sicherheit und Updates

| Schlüssel | Inhalt |
|---|---|
| `auto_lock_minuten` | Nach wie vielen Minuten ohne Eingabe die App sperrt. `0` = nie. |
| `update_aus` | Ob die Suche nach Updates abgeschaltet ist. |
| `update_zuletzt` | Wann zuletzt nachgesehen wurde. |

---

## Was **nicht** im Datenordner liegt

| Was | Wo |
|---|---|
| Desktop-Schlüssel | Schlüsselbund des Betriebssystems (GNOME Keyring / Secret Service, macOS Keychain, Windows Credential Manager). Ausweichweg: `LOTSE_DESKTOP_KEY`. |
| Master-Passwort | Nirgends. Es wird bei jedem Entsperren neu eingegeben. |
| Wiederherstellungscode | Nirgends. Einmal angezeigt, danach nur bei dir. |
| Markerdateien | `.lotse-projekt` in den Projektordnern selbst ([[Erkennungsregeln]]). |
| Fensterposition, Zoom | Wo Tauri sie ablegt; für die Daten unerheblich. |

Es gibt **keine** Protokolldatei. Was Lotse zu sagen hat, steht im Logbuch oder auf der
Konsole.

---

## Sichern

```
# Anwendung schließen, dann:
tar czf lotse-sicherung-$(date +%F).tar.gz -C ~/.local/share lotse
```

Beide Dateien gehören zusammen. Die Sicherung ist verschlüsselt und ohne dein Passwort
wertlos – sie darf also in eine Cloud, auf eine externe Platte, in ein Git-Repo.

Was sie **nicht** ersetzt: den Wiederherstellungscode und den Desktop-Schlüssel. Ohne
Passwort ist die Sicherung nicht aufzuschließen, und `nur_desktop`-Einträge brauchen
zusätzlich den Desktop-Schlüssel aus dem Schlüsselbund – der in der Sicherung nicht
enthalten ist.

Für einen Bestand, der auch ohne Lotse lesbar bleibt, ist der Export der richtige Weg:
**[[Export und Fluchtweg|Export-und-Fluchtweg]]**.

## Umziehen

Datenordner kopieren, fertig. Auf dem neuen Rechner wird beim ersten Entsperren nach dem
Passwort gefragt. Der Desktop-Schlüssel zieht **nicht** mit – er liegt im Schlüsselbund
des alten Rechners. Auf dem neuen musst du ihn aus deinem Passwortmanager eintragen,
sonst sind `nur_desktop`-Einträge dort nicht lesbar.

Mit eingerichtetem Abgleich ist `lotse sync login` der bessere Weg: das neue Gerät holt
alles selbst und bekommt eine eigene Geräte-ID, die du einzeln widerrufen kannst
([[Abgleich]]).
