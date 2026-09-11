# Fehlerbehebung

## Beim Start

**„Der Computer wurde durch Windows geschützt“ / „Entwickler nicht verifiziert“**

Die Installer sind nicht code-signiert. Windows: *Weitere Informationen* → *Trotzdem
ausführen*. macOS: Rechtsklick auf die App → *Öffnen* → *Öffnen*. Siehe
**[[Installation]]**.

**Lotse startet nicht, keine Meldung (Linux)**

Meist fehlt WebKit. Auf Debian/Ubuntu:

```bash
sudo apt-get install libwebkit2gtk-4.1-0
```

Beim AppImage hilft ein Start aus dem Terminal – dann steht der Grund da.

## Passwort und Schlüssel

**Master-Passwort vergessen**

Auf dem Sperrbildschirm *Passwort vergessen?* → Wiederherstellungscode eingeben → neues
Passwort setzen. Ohne den Code sind die Daten unlesbar; es gibt keine Hintertür.

**Wiederherstellungscode verloren, Passwort bekannt**

Sofort handeln: Einstellungen → *Sicherheit* → Passwort ändern. Dabei entsteht ein neuer
Code, der einmal angezeigt wird. Diesmal in den Passwortmanager.

**„nur Desktop“-Einträge sind leer**

Auf diesem Gerät fehlt der Desktop-Schlüssel. Beim Entsperren eingeben, oder – wenn der
Rechner einen Schlüsselbund hat – einmalig dort ablegen lassen.

**Lotse sperrt ständig**

Standard sind 15 Minuten Untätigkeit. Einstellungen → *Sicherheit* → *Automatisches
Sperren*; `0` schaltet es ab.

## Referenzen und Erkennung

**Eine Referenz sagt „nicht erreichbar“, obwohl es sie gibt**

- **Pfad:** Pfade gelten nur auf dem Gerät, auf dem sie angelegt wurden. Auf einem anderen Rechner steht dort *nicht prüfbar* – das ist richtig, nicht kaputt.
- **GitHub/GitLab-Adresse, privates Repo:** ohne hinterlegtes Token kommt Lotse nicht heran. Siehe **[[Verbindungen]]**.
- **Andere Adresse:** Lotse wertet 401 und 403 als *erreichbar* („da, aber nicht für dich“). Nur 404/410 und ein Verbindungsfehler heißen *nicht da*.

**Der Scan findet meine Ordner nicht**

- Höchstens **vier Ebenen** tief unter dem Wurzelordner.
- Ordner mit `.` am Anfang und die Ausschlussliste (`node_modules`, `target`, `build`, `dist`, `.venv` …) werden übersprungen.
- Der Ordner braucht eine **Marke**: `.git`, `Cargo.toml`, `package.json`, eine README … Die vollständige Liste steht unter **[[Beobachter und Erkennung|Beobachter-und-Erkennung]]**.
- Ein bereits erkannter Ordner wird nicht weiter durchsucht – Unterordner eines Vorhabens sind keine eigenen Vorhaben.

**Der Beobachter schreibt nichts**

Er schreibt **verdichtet**: eine Zeile je Vorhaben und Tag, nicht je Ereignis. Und er
endet, sobald Lotse sperrt. Nach dem Entsperren neu starten.

## Abgleich

**„Formatversion wird nicht unterstützt“**

Ein Gerät läuft mit einer älteren Fassung. Beide auf denselben Stand bringen; siehe
**[[Updates]]**.

**Ein Gerät bekommt nichts**

`lotse sync status` auf beiden Geräten. Prüfen: dieselbe Dienst-Adresse, dieselbe
E-Mail, Gerät nicht widerrufen (`lotse sync geraete`).

**Beide Geräte haben unterschiedliche Stände**

Auf beiden *Jetzt abgleichen*. Erst pushen, dann pullen – ein Lauf je Gerät genügt, zwei
sind sicherer.

## Updates

**„Die Liste der Fassungen war nicht erreichbar“**

Netz prüfen. Der Download von Hand geht weiterhin: *Was ist neu* führt zur Seite mit
allen Dateien.

**„Austausch fehlgeschlagen“**

- **macOS:** Liegt Lotse in `/Programme`? Direkt aus dem DMG heraus geht es nicht.
- **Windows:** Rechte im Installationsordner. Notfalls den Installer von Hand ausführen.
- **Linux:** Aus `.deb`/`.rpm` heraus geht es nicht – dort aktualisiert die Paketverwaltung. Für Updates aus der App das AppImage nehmen.

Ein misslungener Austausch lässt die alte Fassung stehen.

## Assistenten (MCP)

**Der Assistent findet den Zugang nicht**

- Ist Lotse entsperrt? Der Zugang endet mit dem Sperren.
- Steht der Port in der eingetragenen Adresse? Standard ist 7457.
- Stimmt das Token? Nach *Token erneuern* muss der Eintrag im Assistenten neu gesetzt werden.

**„Nur POST“ oder 405**

Der Zugang hält keinen SSE-Strom offen. Der Client muss den HTTP-Transport mit POST
benutzen.

**Der Assistent sieht meine Zugangsdaten nicht**

Richtig so. Es gibt kein Werkzeug für den Tresor, und die Suche wirft Tresor-Treffer weg.
Siehe **[[Assistenten (MCP)|Assistenten-MCP]]**.

## Daten

**Habe ich aus Versehen etwas gelöscht?**

Gelöschte Datensätze werden als gelöscht markiert und mit abgeglichen, nicht sofort
entfernt. Ein **[[Bundle|Export-und-Fluchtweg]]** von vorher ist trotzdem der
zuverlässigere Weg zurück.

**Die Datenbank ist beschädigt**

Fast immer die Folge davon, dass der Datenordner in einem synchronisierten Cloud-Ordner
lag und zwei Rechner gleichzeitig geschrieben haben. Aus dem letzten Bundle
wiederherstellen und für mehrere Geräte den **[[Abgleich]]** benutzen.

**Wo liegen die Daten?**

| System | Pfad |
|---|---|
| Linux | `~/.local/share/lotse/` |
| macOS | `~/Library/Application Support/lotse/` |
| Windows | `%APPDATA%\lotse\` |

Ein anderer Ort über `--home` oder `LOTSE_HOME`.

---

## Etwas melden

**Fehler und Wünsche:** [Issues](https://github.com/phish3144/lotse/issues). Hilfreich
sind: Fassung (`lotse --version`), Betriebssystem, was du getan hast, was passiert ist.

**Sicherheitslücken bitte nicht öffentlich**, sondern über *Security* → *Report a
vulnerability* im Repository.
