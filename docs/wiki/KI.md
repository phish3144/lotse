# KI

**Standardmäßig aus, aber nicht mehr versteckt.** Lotse funktioniert vollständig ohne. Was
die KI tut, ist Verdichten und Deuten – nie Entscheiden, nie Automatisieren.

## Sie fragt beim Start

Ist nichts eingerichtet, steht im Hafen eine Karte, die es anbietet und durch die
Einrichtung führt – Ollama lokal oder ein Schlüssel für einen gehosteten Dienst. Das war
bis 0.10 anders: die Einrichtung lag unter Einstellungen → KI, zeigte auf ein lokales
Ollama, das die wenigsten laufen haben, und wurde nie angeboten. Gefunden hat sie kaum
jemand, und benutzt niemand.

**»Nicht mehr fragen« ist eine Antwort.** Danach fragt Lotse nicht wieder; die Einrichtung
steht weiter unter Einstellungen → KI, und dort lässt sich das auch zurücknehmen.

## Was sie tut

| Funktion | Zweck | Was passiert |
|---|---|---|
| **Vorhaben deuten** | `vorhaben_deuten` | Beim Anlegen aus einem Ordner: Titel in Worten statt Ordnername, ein Satz zum Ziel, Themen und die offenen Fäden, die schon dastehen. |
| **Brief verdichten** | `brief_verdichten` | Aus dem Wo-war-ich-Brief wird ein Fließtext. Auf der Projektseite, pro Aufruf, nur auf Klick. |
| **Datei deuten** | `datei_deuten` | Aus einem Dateiauszug wird eine Zusammenfassung. |
| **Kurs vorschlagen** | `kurs_vorschlagen` | Aus dem Ordner eines Vorhabens wird der eine Satz, der sagt, worum es geht und was das Ziel ist. Unter *Bearbeiten*, neben dem Kursfeld. |

### Wo die Knöpfe sitzen

| Funktion | Wo |
|---|---|
| Vorhaben deuten | Beim Anlegen aus einem Ordner, unter *Von der KI deuten lassen*. |
| Kurs vorschlagen | Projektseite → *Bearbeiten*, neben dem Kursfeld. Nur mit angehängtem Ordner. |
| Brief verdichten | Projektseite, oben in der Karte *Wo war ich*. |
| Datei deuten | Projektseite, unten bei den Referenzen. |

Dieselbe Liste steht in der App unter Einstellungen → *KI*, samt Stand der Einrichtung und
einem Knopf **Ausprobieren**, der einen einzigen Probesatz sendet und die Antwort zeigt.
Ohne das war »tut die KI überhaupt etwas?« nirgends zu beantworten.

Ist nichts eingerichtet, steht an diesen Stellen keine Fehlermeldung, sondern der Weg zur
Einrichtung.

Dass die Karte *Wo war ich* nur nach längerer Stille erschien, hat den Knopf zum Verdichten
lange mitversteckt; sie zeigt sich jetzt auch, wenn etwas aus deiner Abwesenheit vorliegt
([[Wo-war-ich-Brief]]).

### Warum beim Anlegen

Das ist der Moment, in dem das Einlesen sonst wenig hergibt. Ohne Modell liefert es den
Ordnernamen als Titel (»heizungssteuerung-esp32«), eine aus Dateinamen geratene Vorlage
und einen offenen Faden, der dich auffordert, den Kurs selbst zu schreiben. Mit Modell
steht da ein beschriebenes Vorhaben.

Gesendet wird dasselbe wie beim Kurs-Vorschlag, und du siehst den vollständigen Text
vorher. Übernommen wird nur, was auch etwas sagt: ein leeres Feld überschreibt nichts.
Die vorgeschlagenen Fäden lassen sich einzeln wegnehmen, bevor du anlegst.

Das Ergebnis ist immer ein **Vorschlag**. Es landet nur im Logbuch, wenn du es
übernimmst; beim Kurs füllt »Übernehmen« das Feld, gespeichert wird erst mit *Speichern*.

### Warum gerade der Kurs

Das ist die Aufgabe, für die es ein Modell braucht. Titel und Vorlage liest die Erkennung
aus Marken – `Cargo.toml`, `platformio.ini`, eine README ([[Erkennungsregeln]]); dafür
wäre ein Modell Verschwendung. Aber die erste Zeile einer README ist als Kurs meistens
eine Überschrift und kein Ziel, und den Satz, der einer Person in drei Monaten sagt, was
sie eigentlich wollte, kann keine Regel schreiben.

Gesendet wird dabei: Name des Vorhabens, die Erkennungsmarken, die ersten 3000 Zeichen der
README und bis zu 40 Dateinamen. **Keine Dateiinhalte außer der README** – wer mehr senden
will, deutet die Datei ausdrücklich ([[Datei deuten|Datei-deuten]]), und das ist dann eine
eigene Entscheidung. Hängt am Vorhaben kein Ordner, gibt es den Knopf nicht: ein Kurs aus
dem Titel allein wäre geraten.

Empfohlenes lokales Modell: `qwen3:8b` – reicht für einen Satz, läuft auf 8 GB.

## Die sieben Zusagen

1. **Aus, bis du es einschaltest.** Kein Ziel, kein Schlüssel, keine Anfrage.
2. **Du siehst vorher, was gesendet würde** – den vollständigen Text, nicht eine Zusammenfassung davon.
3. **Nur auf Klick.** Nichts läuft im Hintergrund.
4. **Der Tresor ist unerreichbar.** Das Modul `ai` hat keinen Zugriffsweg dorthin, geprüft bei jedem Commit.
5. **Das Ziel bestimmst du.** Lokal (Ollama) oder eine beliebige OpenAI-kompatible Adresse.
6. **Der Verbrauch ist sichtbar.** Ein Zähler und ein Protokoll, beides löschbar.
7. **Kein Ergebnis wird ungefragt übernommen.**

## Ziele

| Ziel | Schlüssel nötig | Daten verlassen den Rechner |
|---|---|---|
| **Ollama** (lokal) | nein | **nein** |
| Gemini, Groq, Mistral, OpenRouter … | ja | ja |
| Jede andere OpenAI-kompatible Adresse | je nachdem | je nachdem |

Läuft auf dem Rechner ein Ollama, erkennt Lotse das und sagt es. Dann bleibt alles
lokal – kein Schlüssel, kein Konto, keine Verbindung nach außen.

Für ein gehostetes Ziel legst du den Schlüssel als **Tresor-Eintrag** an und zeigst in
den Einstellungen darauf. Lotse liest ihn nur beim Senden.

## Was protokolliert wird

Je Anfrage: Zeitpunkt, Zweck, **nur der Host** (nicht die vollständige Adresse), Modell,
Anzahl gesendeter Zeichen und, falls das Ziel es meldet, Ein- und Ausgabe-Token.

**Der gesendete Text selbst wird nicht aufbewahrt.** Der Zähler soll dir sagen, wie viel
Lotse verbraucht – nicht ein zweites Archiv deiner Inhalte anlegen.

Höchstens 40.000 Zeichen je Anfrage. Längeres wird gekürzt, und du siehst die Kürzung.

## Datenschutz

Mit **Ollama lokal** verlässt nichts den Rechner. Das ist der Weg, den Lotse empfiehlt
und der ohne weitere Erklärung auskommt.

Bei einem gehosteten Ziel gilt, was dessen Anbieter zusagt – Lotse kann das nicht
garantieren und behauptet es auch nicht. Deshalb die Vorschau vor jedem Senden: was du
dort siehst, ist genau das, was geht.

Details: `docs/THREAT_MODEL.md`, Abschnitt 6b.

## Abschalten

Einstellungen → *KI* → Ziel leeren. Danach ist es, als wäre es nie da gewesen. Der
Zähler und das Protokoll lassen sich getrennt löschen.
