# Einstellungen

Sieben Reiter, jeder Schalter. Erreichbar über *Einstellungen* in der Kopfzeile; jeder
Reiter hat eine eigene Adresse (`#/einstellungen/verbindungen`), sodass man sie
verlinken und mit Alt + ← zurücknavigieren kann.

Was gespeichert wird, steht in der lokalen Datenbank unter `meta` und wird **nicht**
abgeglichen – jedes Gerät hat seine eigenen Einstellungen ([[Datenablage]]).

---

## Darstellung

| Feld | Bedeutung |
|---|---|
| **Thema** | `Wie das System` (Standard), `Hell` oder `Dunkel`. |

Ab Werk folgt Lotse dem Betriebssystem. Eine ausdrückliche Wahl schlägt das System in
beide Richtungen – wer dunkel wählt, bleibt auch mittags dunkel.

Die Wahl gilt **für dieses Gerät** und wird nicht abgeglichen. Sie liegt auch nicht in
der Datenbank, sondern im Speicher der Oberfläche: der Sperrbildschirm erscheint, bevor
irgendetwas entschlüsselt ist, und soll trotzdem schon richtig aussehen.

## Ordner

Welche Wurzelordner Lotse durchsucht und beobachtet.

| Schalter | Was er tut |
|---|---|
| **Ordner wählen …** | Öffnet den Auswahldialog des Systems und merkt den Ordner als Wurzel. |
| **Durchsuchen** | Führt einen Scan aus und legt Funde in der Hafeneinfahrt ab. Legt nichts von selbst an. |
| **Laufend beobachten** | Startet oder beendet den Ordner-Beobachter. |

Läuft der Beobachter, steht darunter, welche Wurzeln er beobachtet. Er sammelt nur Pfade
und Zeitpunkte, **nie Inhalte**; `.env`, Schlüssel und Zertifikate stehen auf einer
festen Ausschlussliste ([[Erkennungsregeln]]).

> **Nur in der Desktop-App.** Im Browser hat Lotse keinen Zugriff auf deine Ordner. Der
> Reiter zeigt dort einen Hinweis statt der Schalter.

Sperren beendet den Beobachter mit. Gesperrt heißt gesperrt.

Mehr: [[Beobachter und Erkennung|Beobachter-und-Erkennung]].

---

## Verbindungen

### GitHub und GitLab

Je Hoster eine Zeile. Verbinden ist **ein Feld**: Token einfügen, fertig. *Token
erstellen …* öffnet die richtige Seite beim Hoster, damit man nicht suchen muss, welche
Rechte nötig sind.

| Schalter | Was er tut |
|---|---|
| **Token einfügen** | Legt das Token als Tresor-Eintrag ab und merkt in `meta` nur den Zeiger darauf. |
| **Trennen** | Löscht den Zeiger und den Tresor-Eintrag. Das Token beim Hoster bleibt gültig – dort musst du es selbst widerrufen, wenn du das willst. |
| **Mit dem Ordner-Beobachter mitlaufen lassen** | Fragt die Gegenseite selbsttätig ab, solange der Beobachter läuft – höchstens alle 30 Minuten. |
| **Mehr …** | Für den Fall, dass das Token schon woanders im Tresor liegt: Eintrag und Feldname von Hand angeben. |

GitHub und GitLab sind getrennt gespeichert. Ein Token für GitHub wird nie an GitLab
geschickt, auch nicht versehentlich. Siehe [[Gegenseite]].

### Kalender

Ein Kalender wird nicht hier *zugeordnet*, sondern als Referenz am Vorhaben – die
Abonnement-Adresse als `url`, die auf `.ics` endet oder mit `webcal://` beginnt.

Was hier steht, ist **Meine Kalender**: die Liste der Kalender, die dir gehören. Ein
Vorrat, keine Zuordnung – er erspart das Abtippen derselben langen Adresse für jedes
Vorhaben, und er ist die Voraussetzung dafür, dass eine Projektseite nachsehen kann, ob
dort etwas zum Vorhaben steht.

| Schalter | Was er tut |
|---|---|
| **Hinzufügen** | Nimmt eine Adresse in den Vorrat. Was keine Kalenderadresse ist, wird abgewiesen statt stillschweigend übernommen. |
| **Entfernen** | Nimmt sie wieder heraus. Referenzen an Vorhaben bleiben davon unberührt – die sind die Zuordnung. |

Diese Adressen sind **Geheimnisse**: wer sie hat, liest deinen Kalender. Sie stehen im
Klartext in der verschlüsselten Datenbank – derselbe Schutz wie für alles andere dort, aber
ohne die zusätzliche Hülle des [[Tresor]]s.

Geholt wird der Kalender beim Öffnen der Projektseite, danach höchstens alle 15 Minuten
neu. Siehe [[Kalender]].

---

## Abgleich

### Konto beim Dienst löschen

Löscht Konto, alle Umschläge, alle Anhänge, alle Geräte und alle Sitzungen beim Dienst –
**unwiderruflich**. Die E-Mail-Adresse ist danach wieder frei. Bestätigt wird mit dem
getippten Wort `LÖSCHEN`, nicht mit einem zweiten Klick.

**Die Daten auf diesem Gerät bleiben.** Das sind zwei Entscheidungen, und sie haben zwei
Knöpfe: *Konto löschen* hier, *Gerät zurücksetzen* unter [[#Sicherheit|Sicherheit]]. Wer
beides will, tut beides – ein Knopf, der still beides täte, wäre eine Falle.

Der Dienst verlangt dafür das Master-Passwort und nicht nur die offene Sitzung. Den
Wiederherstellungscode verlangt er nicht: wer nicht mehr hineinkommt, hat trotzdem das
Recht, seine Daten loszuwerden. Auf der Kommandozeile: `lotse sync konto-loeschen`.


| Schalter | Was er tut |
|---|---|
| **Abgleich einrichten** | Fragt Dienst-Adresse, E-Mail-Adresse und Wiederherstellungscode ab und registriert dieses Konto. |
| **Jetzt abgleichen** | Pusht, dann pullt. |
| **Widerrufen** (je Gerät) | Beendet die Sitzungen dieses Geräts. Mit Rückfrage – *Ja* / *Nein* statt eines Dialogs, damit man nicht wegklickt, was man nicht wollte. |

Darunter die Liste der Geräte mit Name, Plattform und letztem Abgleich. Das eigene Gerät
ist markiert.

Widerrufen ist **kein Fernlöschen**: die Daten auf dem Gerät bleiben verschlüsselt
liegen, es kann nur nicht mehr abgleichen. Mehr: [[Abgleich]], [[Sync-Protokoll]].

---

## KI

Standardmäßig **aus**. Ohne eingetragenes Ziel passiert hier nichts, und ohne dein
ausdrückliches Zutun verlässt nie etwas das Gerät.

| Feld | Bedeutung |
|---|---|
| **Adresse** | Basis-Adresse einer OpenAI-kompatiblen Schnittstelle. Für ein lokales Ollama: `http://localhost:11434/v1`. |
| **Modell** | Modellname, z. B. `llama3.2`. Bei erreichbarem Ziel füllt Lotse eine Auswahlliste. |
| **Schlüssel** | API-Schlüssel. Landet im Tresor, nicht in den Einstellungen. Lokale Ziele brauchen keinen. |
| **Merken** | Speichert Adresse, Modell und Schlüssel. |

### Was bisher gesendet wurde

Ein Protokoll: wann, wozu, wie viele Zeichen, wie viele Token hin und zurück. Dazu
**Zähler und Protokoll löschen**.

Der Abschnitt existiert, damit »was kostet das eigentlich« beantwortbar ist, **bevor** die
Rechnung kommt. Mehr: [[KI]].

---

## Export

| Schalter | Was er tut |
|---|---|
| **Klartext-Spiegel schreiben …** | Markdown je Vorhaben und Monat. **Ohne Tresor.** |
| **Verschlüsseltes Bundle …** | Alles als JSON, mit `age` verschlüsselt, **mit Tresor**. Verlangt eine Passphrase von mindestens 8 Zeichen. |

Der Fluchtweg, und er ist Absicht: **[[Export und Fluchtweg|Export-und-Fluchtweg]]**.

---

## Sicherheit

### Zugang für Assistenten (MCP)

| Schalter | Was er tut |
|---|---|
| **Starten / Beenden** | Bindet den Zugang auf `127.0.0.1`. |
| **Auge** | Zeigt das Token. Standardmäßig verdeckt. |
| **Kopieren** | Legt die Zeile für den Assistenten in die Zwischenablage. |
| **Token erneuern** | Neues Token, alte Verbindungen fallen weg. |

Der Zugang endet mit dem Sperren. Tresor-Inhalte sind über ihn **nie** erreichbar – nicht
gefiltert, sondern nicht vorhanden. Mehr: [[Assistenten (MCP)|Assistenten-MCP]].

### Master-Passwort ändern

Fragt das alte und zweimal das neue. Widerruft anschließend **alle anderen** Sitzungen.
Der Kontoschlüssel selbst bleibt derselbe und wird nur neu gewrappt – deshalb muss nichts
neu verschlüsselt und nichts neu hochgeladen werden.

Danach wird ein **neuer Wiederherstellungscode** angezeigt, einmalig, mit *Habe ich
gesichert* zum Bestätigen. Der alte gilt nicht mehr.

---

### Gerät zurücksetzen

Entfernt Lotse von diesem Rechner: Kontodatei, Datenbank und den Desktop-Schlüssel aus
dem Schlüsselbund ([[Datenablage]]). Was auf anderen Geräten und beim Abgleich liegt,
bleibt unberührt.

Die Bestätigung ist getippt, nicht geklickt: erst das Wort `LÖSCHEN` gibt den Knopf frei.
Ein Klick aus Versehen ist möglich, ein getipptes Wort nicht. Danach startet Lotse neu und
beginnt wieder mit der Einrichtung.

> **Ohne Wiederherstellungscode ist das Konto danach zu.** Beim Abgleich bleibt der
> Ciphertext liegen, aber niemand kann ihn mehr öffnen. Auf der Kommandozeile macht
> `lotse zuruecksetzen` dasselbe ([[Kommandozeile-Referenz]]).

## Version

| Schalter | Was er tut |
|---|---|
| **Jetzt aktualisieren** | Lädt die neue Fassung, prüft die Signatur, tauscht aus, startet neu. Mit Fortschrittsbalken. |
| **Was ist neu** | Öffnet den Änderungstext im Browser. |
| **Automatisch nachsehen** | Ob Lotse selbst nach Updates sucht. |

Aus einem `.deb` oder `.rpm` heraus geht der Austausch nicht – dort aktualisiert die
Paketverwaltung, und Lotse zeigt das statt eines Knopfes, der nichts täte. Mehr:
[[Updates]].

### Platz im System

Nur unter Linux und nur, wenn Lotse als AppImage läuft. Ein AppImage wird nicht
installiert; dieser Abschnitt holt das nach.

| Schalter | Was er tut |
|---|---|
| **Platz geben und ins Menü** | Legt die Datei nach `~/.local/share/lotse/Lotse.AppImage` – feste Stelle, fester Name – und schreibt Desktop-Datei und Icon nach `~/.local/share`. |
| **Von dort neu starten** | Erscheint danach: es läuft noch die heruntergeladene Datei. Sperrt vorher, weil der Schlüssel einen Neustart nicht überlebt. |
| **Eintrag entfernen** | Nimmt Desktop-Datei und Icon wieder weg. Die AppImage-Datei bleibt – das ist das laufende Programm. |

Angezeigt wird, woraus Lotse gerade läuft, wo sein Platz wäre und ob es den Eintrag schon
gibt. Die heruntergeladene Datei bleibt liegen; die Anzeige nennt sie, damit du sie selbst
wegräumen kannst – Lotse löscht nichts in deinem Download-Ordner. Mehr: [[Installation]].

### Dieses Gerät

Version, Geräte-ID, Plattform und der Pfad des Datenordners – die Angaben, die in einen
Fehlerbericht gehören.

| Schalter | Optionen |
|---|---|
| **Von selbst sperren nach** | 5, 15, 30, 60 Minuten oder **nie**. Standard: 15. |
| **Jetzt sperren** | Sofort. |

Dreißig Sekunden vor dem Sperren meldet sich Lotse; jede Tasten- oder Mausbewegung
stellt den Zähler zurück. Sperren beendet auch den Ordner-Beobachter und den
MCP-Zugang: der Schlüssel verlässt den Speicher, und ohne Schlüssel gibt es nichts zu
bedienen.

---

## Was es **nicht** gibt

| Fehlt | Warum |
|---|---|
| Konfigurationsdatei | Ein zweiter Ort für Einstellungen ist ein zweiter Ort zum Nachsehen, wenn etwas unerwartet ist. Alles steht in der Datenbank. |
| Theme-Umschalter | Lotse folgt der Systemeinstellung für hell und dunkel. |
| Telemetrie-Schalter | Es gibt keine Telemetrie, also nichts abzuschalten ([[Nicht-Ziele]]). |
| Konto löschen | Noch nicht gebaut. Bis dahin: Datenordner löschen und beim Dienst alle Geräte widerrufen. |
