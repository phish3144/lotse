# Fehlermeldungen

Jede Meldung, die der Kern ausgeben kann – was sie bedeutet und was zu tun ist. Wenn
etwas klemmt, ohne dass eine Meldung erscheint, ist **[[Fehlerbehebung]]** der richtige
Ort.

Die Meldungen sind bewusst deutsch und bewusst konkret. Eine Meldung, die nur »Fehler«
sagt, kostet den Leser eine halbe Stunde.

---

## Krypto und Schlüssel

### `Entschlüsselung fehlgeschlagen (falscher Schlüssel oder manipulierte Daten)`

Ein Ciphertext ließ sich nicht öffnen. Zwei Möglichkeiten, und Lotse kann sie nicht
unterscheiden – das ist bei authentifizierter Verschlüsselung so gewollt:

1. **Falscher Schlüssel.** Meist: falsches Master-Passwort. Bei einem
   `nur_desktop`-Tresor-Eintrag auch ein falscher oder fehlender Desktop-Schlüssel.
2. **Veränderte Daten.** Die Datei wurde beschädigt oder angefasst. Ein einziges
   gekipptes Bit reicht; das Authentifizierungsetikett schlägt dann fehl.

Was tun: Passwort prüfen. Wenn es sicher richtig ist, aus einer Sicherung zurückgehen
([[Datenablage]]). Ein »Reparieren« gibt es nicht und soll es nicht geben: Lotse soll
nicht erraten, was dort mal stand.

### `Der Desktop-Schlüssel fehlt; Einträge der Stufe »nur Desktop« sind hier nicht lesbar`

Der Desktop-Schlüssel ist nicht im Schlüsselbund und auch nicht in
`LOTSE_DESKTOP_KEY`. Alles außer diesen Einträgen funktioniert normal.

Tritt auf: auf einem neuen Rechner, nach dem Zurücksetzen des Schlüsselbunds, in
Containern und Server-Sitzungen, und auf jedem Gerät, das nicht der Desktop ist – dort
ist es kein Fehler, sondern der Sinn der Stufe ([[Tresor]]).

Was tun: den Schlüssel aus dem Passwortmanager in `LOTSE_DESKTOP_KEY` setzen oder in den
[[Einstellungen]] unter *Sicherheit* eintragen.

### `Schlüsselableitung fehlgeschlagen: …`

Argon2id konnte nicht rechnen. In der Praxis: zu wenig Arbeitsspeicher – die Ableitung
verlangt 64 MiB in einem Stück. Auf sehr kleinen Containern kommt das vor.

Was tun: dem Prozess mehr Speicher geben. Die Parameter herunterzusetzen ist keine
Lösung, sondern eine Schwächung ([[Schlüssel und Krypto|Schluessel-und-Krypto]]).

### `Kryptografie: …`

Ein Baustein hat etwas abgelehnt, etwa weil ein Schlüssel nicht 32 Byte lang ist. Sollte
nie im Alltag erscheinen – wenn doch, ist es einen Bericht wert ([[Sicherheit]]).

---

## Daten und Eingaben

### `Ungültige Eingabe: …`

Etwas passte nicht zur Regel. Der Text hinter dem Doppelpunkt nennt es. Häufig:

| Text | Bedeutung |
|---|---|
| `Kein Modell gewählt` | KI-Ziel ohne Modellnamen ([[KI]]). |
| `Für dieses Ziel wird ein Schlüssel gebraucht` | Ein entferntes KI-Ziel ohne API-Schlüssel. Lokale Ziele brauchen keinen. |
| `pausiert und wartet verlangen eine Notiz` | Statuswechsel ohne `--notiz`. Absicht, siehe [[Vorhaben]]. |
| Zu langer Text | Die KI-Anfrage ist auf 40 000 Zeichen begrenzt. |

### `Nicht gefunden: …`

Ein Vorhaben, eine Notiz oder eine Referenz mit dieser ID oder diesem Titel existiert
nicht. Bei Titeln: Lotse nimmt auch Titelanfänge, aber nur wenn sie eindeutig sind.

Was tun: `lotse projekt liste` beziehungsweise `lotse offen` zeigt die gültigen IDs.

### `Ungültige Base64-Kodierung`

Ein Feld, das Base64 enthalten sollte, tut es nicht. Tritt auf bei von Hand
bearbeiteten `konto.json`-Dateien oder einem kaputten Download eines Sync-Umschlags.

### `Serialisierung: …`

JSON ließ sich nicht lesen oder schreiben. Fast immer eine beschädigte Datei.

### `Formatversion … wird von diesem Kern (Version …) nicht unterstützt`

Die Daten wurden von einer Fassung mit anderer Krypto-Komposition geschrieben. Die
erste Zahl ist die der Daten, die zweite die dieses Programms.

Ist die Zahl der Daten **höher**: dieses Lotse ist zu alt. Aktualisieren ([[Updates]]).

Ist sie **niedriger**: die Daten stammen aus einer älteren Komposition. Lotse liest sie
nicht blind weiter, weil »irgendwie entschlüsseln« genau die Nachlässigkeit ist, aus der
Sicherheitslücken werden. Es gibt dann einen ausdrücklichen Migrationsweg, der im
Änderungstext der betreffenden Fassung steht ([[Änderungen|Aenderungen]]).

---

## Netz und Abgleich

### `Sync-Dienst antwortet …`

Der Dienst hat abgelehnt. Die Meldung nennt HTTP-Status, einen Code und einen Text.

| Status | Bedeutung | Was tun |
|---|---|---|
| `401` | Sitzung abgelaufen oder Gerät widerrufen. | Neu anmelden. Wurde das Gerät absichtlich widerrufen, ist das die richtige Antwort. |
| `403` | Keine Berechtigung. | Konto prüfen. |
| `404` | Konto oder Datensatz unbekannt. | Adresse prüfen – `--url` zeigt vielleicht auf den falschen Dienst. |
| `409` | Konflikt beim Push. | Lotse löst das selbst auf ([[Sync-Protokoll]]); bleibt es stehen, einmal `lotse sync jetzt` wiederholen. |
| `413` | Zu groß. | Ein einzelner Anhang sprengt die Grenze des Dienstes. |
| `429` | Zu viele Anfragen. | Warten. Der Dienst begrenzt absichtlich. |
| `5xx` | Der Dienst hat ein Problem. | Später erneut. Lokal arbeitet Lotse ohne Abgleich weiter. |

### `Netzwerk: …`

Die Verbindung kam nicht zustande: keine Namensauflösung, abgewiesen, Zeitüberschreitung,
TLS-Prüfung fehlgeschlagen. Der Text nennt, was genau.

Bei TLS-Fehlern in Firmennetzen: Lotse prüft gegen den Wurzelspeicher des Systems. Wenn
`curl` auf demselben Rechner geht und Lotse nicht, ist es ein Fehler und kein Zustand –
bitte melden.

---

## Dateisystem und Speicher

### `Dateisystem: …`

Ein Lese- oder Schreibfehler. Meist: Ordner existiert nicht, keine Rechte, Platte voll.

### `Speicher: …`

SQLite hat etwas abgelehnt. Mit einer SQLCipher-Datenbank bedeutet »file is not a
database« fast immer: **falsches Passwort**, nicht kaputte Datei – die Seiten sind
verschlüsselt, und ohne Schlüssel sieht auch eine intakte Datei nach Unsinn aus.

Bei `database is locked`: eine zweite Lotse-Instanz läuft noch. App und Kommandozeile
gleichzeitig auf demselben Datenordner ist eine schlechte Idee.

---

## Was keine Fehlermeldung ist

| Text | Warum kein Fehler |
|---|---|
| `Version … ist die neueste.` | `lotse update` hat nachgesehen, es gibt nichts Neues. |
| `Kein Sync eingerichtet` | Hinweis von `lotse sync status`. Lotse funktioniert ohne Abgleich vollständig. |
| `nicht_pruefbar` | Prüfstatus einer Referenz, bei der kein Programm nachsehen kann – ein Regalfach etwa ([[Referenzen]]). |
| `Kein Projekt erkannt – landet im Postkorb.` | `lotse log` ohne Zuordnung. Der Eintrag ist gespeichert, nur eben im Postkorb. |

---

## Eine Meldung melden

Wenn eine Meldung nicht erklärt, was zu tun ist, ist das ein Fehler in der Meldung.
Bitte als Issue melden, mit dem genauen Wortlaut und dem Kommando –
[[Fehlerbehebung]] nennt den Weg.
