# Erste Schritte

## 1. Das Master-Passwort

Beim ersten Start fragt Lotse nach einem Master-Passwort und einem Namen für diesen
Rechner.

Das Passwort **schützt alles**: die Datenbank auf der Platte und alles, was beim
Abgleich das Gerät verlässt. Es wird nirgends gespeichert und nirgends geprüft –
sondern zum Schlüssel gerechnet. Das hat eine Folge, die man verstanden haben sollte:

> **Ein falsches Passwort heißt nicht „Zugriff verweigert“, sondern: die Daten bleiben
> unlesbar.** Es gibt niemanden, der es zurücksetzen könnte – auch uns nicht.

Nimm ein langes Passwort aus deinem Passwortmanager. Vier zufällige Wörter sind besser
als `Hunde123!`.

## 2. Die zwei Schlüssel

Direkt danach zeigt Lotse zwei Zeichenfolgen – **genau einmal**:

| | Wofür |
|---|---|
| **Wiederherstellungscode** | Öffnet das Konto, wenn du das Master-Passwort vergisst. Auch nötig, um ein zweites Gerät anzumelden. |
| **Desktop-Schlüssel** | Zusätzlich nötig für Tresor-Einträge der Stufe *nur Desktop*. |

Beides gehört in den Passwortmanager. Nicht auf einen Zettel, nicht in eine Notiz-App,
nicht in eine unverschlüsselte Datei.

Lotse kann sie nicht noch einmal zeigen. Das ist kein Versäumnis: könnte es das, könnte
es auch jemand anders.

Im nächsten Schritt tippst du den Wiederherstellungscode einmal ab. Das wirkt lästig und
ist Absicht – so bleibt es nicht beim Glauben, ihn gesichert zu haben.

Wenn der Rechner einen Schlüsselbund hat (macOS Keychain, Windows Credential Manager,
GNOME Keyring), legt Lotse den Desktop-Schlüssel zusätzlich dort ab. Dann musst du ihn
auf *diesem* Rechner nicht mehr eingeben. Auf einem zweiten Gerät schon.

## 3. Das erste Vorhaben

Drei Wege hinein – alle gleichwertig:

### Ordner durchsuchen lassen

Einstellungen → *Ordner* → einen Wurzelordner wählen → *Einmal durchsuchen*.

Lotse sucht in den Unterordnern nach Erkennungsmarken (`.git`, `Cargo.toml`,
`platformio.ini`, eine README …) und schlägt vor, was es findet. Die Funde landen im
Hafen unter **Hafeneinfahrt** und warten dort. Übernommen wird nichts von allein, und
Dateiinhalte liest Lotse dabei nicht.

Details: **[[Beobachter und Erkennung|Beobachter-und-Erkennung]]**

### Von Hand anlegen

Hafen → *Neues Projekt*. Titel und ein Satz dazu, worum es geht.

Dieser Satz ist der **Kurs**. Nimm ihn ernst: er ist die eine Zeile, die dir in drei
Monaten sagt, was du eigentlich wolltest. Nicht „Gartenhaus“, sondern *„Fundament bis
Oktober, danach Winterpause, Aufbau im Frühjahr.“*

### Einfach schreiben

<kbd>Strg</kbd>+<kbd>K</kbd> von überall in der App. Ohne Zuordnung landet der Gedanke
im **Postkorb** – einem Vorhaben wie jedes andere, nur dass es das Ungeordnete sammelt.
Mit `@projekt` geht er direkt dorthin; Lotse schlägt beim Tippen passende vor.

Nichts geht verloren, während du überlegst, wohin es gehört.

## 4. Der Rhythmus

Lotse verlangt keine Pflege. Es lohnt sich nur, an einer Stelle diszipliniert zu sein:

**Wenn du ein Vorhaben verlässt, schreib eine Zeile.** Was du getan hast und was als
Nächstes dran ist. Das ist die ganze Methode.

Beim Wechsel auf *pausiert* oder *wartet* verlangt Lotse diese Zeile sogar – als
**Übergabenotiz**. Pflicht, aber billig: zwei Sätze reichen.

Beim nächsten Öffnen steht oben der **Wo-war-ich-Brief**: letzter Kontakt, offene Fäden,
was sich seither getan hat. Genau der Zustand, den du beim Weggehen hinterlassen hast.

## 5. Was als Nächstes lohnt

| Wenn du … | dann … |
|---|---|
| an mehreren Rechnern arbeitest | **[[Abgleich]]** einrichten |
| Zugangsdaten am Vorhaben brauchst | **[[Tresor]]** |
| Code schreibst | **[[Beobachter und Erkennung\|Beobachter-und-Erkennung]]** und **[[Verbindungen]]** |
| mit Claude Code o. ä. arbeitest | **[[Assistenten (MCP)\|Assistenten-MCP]]** |
| lieber tippst als klickst | **[[Kommandozeile]]** |

## Die Kommandozeile parallel

Die CLI arbeitet auf demselben Ordner. Einrichten geht auch dort:

```bash
lotse init
lotse log "Fundament ausgehoben. Nächster Schritt: Schalung."
lotse hafen
```

Steht dein Arbeitsverzeichnis in einem erkannten Projektordner, weiß `lotse log` von
selbst, wohin der Eintrag gehört.
