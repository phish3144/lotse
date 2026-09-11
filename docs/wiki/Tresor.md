# Tresor

Zugangsdaten, die **an einem Vorhaben hängen**: der Router im Gartenhaus, der
Admin-Login der eigenen App, die Kontonummer des Vereins, ein API-Schlüssel.

## Was hierher gehört – und was nicht

**Hierher:** was sonst in einer Textdatei neben dem Projekt landen würde. Wenn du beim
Wiederaufnehmen eines Vorhabens danach suchst, gehört es hierher.

**Nicht hierher:** die zweihundert Web-Logins des Alltags. Dafür gibt es
Passwortmanager, und Lotse will keiner werden. Es fehlt alles, was einen guten
Passwortmanager ausmacht: Browser-Integration, Autofill, Passwortgenerator,
Leak-Prüfung, Teilen.

## Zwei Stufen

| Stufe | Lesbar |
|---|---|
| **überall** | Auf jedem Gerät, das dein Master-Passwort kennt. |
| **nur Desktop** | Zusätzlich ist der **Desktop-Schlüssel** nötig. |

„nur Desktop“ ist für das, was auch dann nicht offenliegen soll, wenn jemand dein
Master-Passwort kennt: Bankzugänge, Notfall-PINs. Die Einträge werden mit abgeglichen –
ein fremdes Gerät kann sie aber nicht öffnen, weil ihm der zweite Schlüssel fehlt.

In der Oberfläche steht die Stufe als Etikett *und* als Kante links an der Karte.

## Wie es verschlüsselt ist

Jeder Eintrag hat einen **eigenen Datenschlüssel** (DEK), der wiederum mit dem Schlüssel
seiner Stufe eingepackt ist (KEK). Jedes Feld wird einzeln versiegelt, mit dem Eintrag
und dem Feldnamen als zusätzlichen Daten – ein Feld lässt sich also nicht in einen
anderen Eintrag umhängen.

Der Speicher sieht im Klartext nur: Titel, Zuordnung, Stufe. Die Werte sind innerhalb
eines JSON-Blocks versiegelt.

Der Datenprovider hält **nie** Klartext. Wenn du *Zeigen* drückst, wird genau ein Feld
entschlüsselt, angezeigt und wieder verworfen.

## Zwischenablage

Kopierte Werte räumt Lotse nach **30 Sekunden** wieder weg – aber nur, wenn seither
nichts anderes kopiert wurde. Was du inzwischen selbst kopiert hast, bleibt: deine
Zwischenablage gehört dir.

## In der Oberfläche

Tresor → *Neuer Zugang*. Titel, Vorhaben, Stufe, dann beliebig viele Felder als
Name/Wert.

Auf der Projektseite steht unter **Zugänge** nur, was zu diesem Vorhaben gehört.

## In der Kommandozeile

```bash
lotse tresor add "Fritzbox Gartenhaus" --projekt Gartenhaus benutzer=admin passwort=xyz
lotse tresor add "Vereinskonto" --stufe nur_desktop --projekt Verein iban=DE.. pin=1234
lotse tresor liste
lotse tresor zeige <id> passwort
lotse tresor loeschen <id>
```

> Werte auf der Kommandozeile landen im Shell-Verlauf und in der Prozessliste. Für
> Geheimnisse, die das nicht vertragen, nimm die Oberfläche.

## Wer den Tresor nicht sehen kann

Das ist im Programm festgelegt, nicht eingestellt. Diese Teile haben **keinen
Zugriffsweg** zum Tresor – geprüft bei jedem Commit durch `scripts/modulgrenzen.sh`:

`ai` · `detect` · `dokument` · `export::spiegel` · `forge` · `git` · `kalender` ·
`mcp` · `netz` · `update` · `watcher`

Konkret heißt das:

- **Kein KI-Ziel** sieht je einen Tresor-Wert.
- **Kein Assistent** über MCP kann Tresor-Einträge auch nur auflisten; die Volltextsuche
  wirft dort Treffer der Art *tresor* weg, bevor sie antwortet.
- **Der Klartext-Spiegel** beim Export enthält keine Zugänge.
- **Der Ordner-Beobachter** kann nichts damit tun.

Nur `export::bundle`, die Oberfläche und die CLI dürfen Tresor-Werte lesen.

## Verlust

| Weg | Folge |
|---|---|
| Master-Passwort vergessen | Mit dem **Wiederherstellungscode** neu setzen. |
| Wiederherstellungscode weg, Passwort bekannt | Kein Problem – aber sofort ein neues Paar erzeugen (Passwort ändern). |
| Beides weg | Die Daten sind unlesbar. Es gibt keine Hintertür. |
| Desktop-Schlüssel weg | Einträge der Stufe *nur Desktop* sind unlesbar; alles andere bleibt. |

Deshalb: beides in den Passwortmanager, und regelmäßig ein **[[Bundle|Export-und-Fluchtweg]]**.
