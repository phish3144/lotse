# Sicherheit

Die Kurzfassung. Vollständig und verbindlich steht es in
[`docs/THREAT_MODEL.md`](https://github.com/phish3144/lotse/blob/main/docs/THREAT_MODEL.md).

## Was geschützt ist

| Gegen | Wie |
|---|---|
| **Gestohlener Rechner** | Die Datenbank ist verschlüsselt (SQLCipher). Ohne Master-Passwort ist sie unlesbar. |
| **Neugieriger Sync-Betreiber** | Er sieht nur verschlüsselte Umschläge: keine Titel, keine Texte, keine Zugänge. |
| **Abgehörte Verbindung** | TLS mit dem Wurzelspeicher des Betriebssystems; die Inhalte sind ohnehin schon verschlüsselt. |
| **Untergeschobenes Update** | Signaturprüfung gegen den eingebauten öffentlichen Schlüssel. |
| **Blick über die Schulter** | Werte bleiben verdeckt, bis sie einzeln angefordert werden. Automatisches Sperren nach 15 Minuten. |
| **Fremdes Gerät am Konto** | Stufe *nur Desktop* braucht zusätzlich den Desktop-Schlüssel. |

## Was nicht geschützt ist

Ehrlichkeit gehört dazu:

- **Ein anderes Programm mit deinen Rechten** auf demselben Rechner. Ist Lotse entsperrt, kann es mitlesen. Dagegen hilft keine Anwendung, sondern nur das Betriebssystem.
- **Ein Schadprogramm mit Tastaturmitschnitt.** Es bekommt das Master-Passwort.
- **Die erste Installation.** Die Installer sind nicht code-signiert; prüfe die Prüfsumme.
- **Vergessenes Passwort ohne Wiederherstellungscode.** Dann sind die Daten weg. Es gibt keine Hintertür – das ist der Preis dafür, dass auch sonst niemand hineinkommt.

## Die Schlüssel

| Schlüssel | Woher | Wofür |
|---|---|---|
| **Master-Passwort** | von dir | Wurzel von allem |
| **Wiederherstellungscode** | einmalig angezeigt | öffnet das Konto ohne Passwort; meldet neue Geräte an |
| **Desktop-Schlüssel** | einmalig angezeigt | Tresor-Stufe *nur Desktop* |

Aus dem Master-Passwort wird mit **Argon2id** ein Schlüssel abgeleitet – absichtlich
langsam, damit Durchprobieren teuer ist. Alle Schlüssel im Speicher sind `Key32` und
werden beim Wegwerfen überschrieben.

**Keine eigenen Krypto-Primitive.** Nur RustCrypto, `age` und `zeroize`. Jede Änderung
an der Komposition erhöht die Formatversion und wird im Bedrohungsmodell protokolliert.

## Trennwände im Programm

Bestimmte Teile haben **keinen Zugriffsweg** zum Tresor. Das ist nicht Konvention,
sondern wird bei jedem Commit und in CI geprüft (`scripts/modulgrenzen.sh`):

`ai` · `detect` · `dokument` · `export::spiegel` · `forge` · `git` · `kalender` ·
`mcp` · `netz` · `update` · `watcher`

Nur `export::bundle`, die Oberfläche und die CLI dürfen Tresor-Werte lesen.

Die Prüfung folgt einem Modul auch dann, wenn es in mehrere Dateien aufgeteilt wird, und
schlägt an, wenn ein Name aus der Liste verschwindet – sonst verschwände die Regel
unbemerkt mit ihm.

## Was Lotse nach außen redet

Ohne Abgleich, ohne Verbindungen und ohne KI: nur die Update-Prüfung, und auch die lässt
sich abschalten. Dann redet Lotse mit niemandem.

Eingeschaltet, jeweils einzeln:

| Wohin | Wann | Was der Gegenüber sieht |
|---|---|---|
| Sync-Dienst | beim Abgleich | verschlüsselte Umschläge, Geräte-ID, E-Mail |
| GitHub / GitLab | auf Knopfdruck oder alle 30 min | dein Token, die Repo-Adresse |
| Kalender | beim Öffnen, höchstens alle 15 min | die Abo-Adresse |
| KI-Ziel | nur auf Klick | genau den Text, den du vorher gesehen hast |
| Veröffentlichungen | Update-Prüfung | die IP-Adresse |

## Einen Fund melden

Sicherheitslücken bitte **nicht** als öffentliches Issue, sondern über die private
Meldefunktion von GitHub: *Security* → *Report a vulnerability*.
