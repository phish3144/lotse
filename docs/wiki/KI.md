# KI

**Standardmäßig aus.** Lotse funktioniert vollständig ohne. Was die KI tut, ist
Verdichten und Deuten – nie Entscheiden, nie Automatisieren.

## Was sie tut

| Funktion | Was passiert |
|---|---|
| **Brief verdichten** | Aus dem Wo-war-ich-Brief wird ein Fließtext. Auf der Projektseite, pro Aufruf, nur auf Klick. |
| **Datei deuten** | Aus einem Dateiauszug wird eine Zusammenfassung. |

Das Ergebnis ist immer ein **Vorschlag**. Es landet nur im Logbuch, wenn du es
übernimmst.

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
