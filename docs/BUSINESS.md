# Lotse – Überlegungen zu einer späteren Monetarisierung

Stand: 2026-09-06. Lotse wird zuerst für eine Person gebaut und läuft kostenlos. Dieses
Dokument hält fest, welche Entscheidungen heute getroffen werden, damit eine spätere
Monetarisierung möglich bleibt, ohne dass heute Bezahl-Code entsteht.

## 1. Was heute schon dafür getan wird

| Entscheidung | Warum sie jetzt billig und später teuer ist |
|---|---|
| `account_id` auf jedem Datensatz im Sync-Dienst | Mandantenfähigkeit nachträglich einzubauen ist eine Migration aller Daten. |
| `plan` und `flags` am Konto | Feature-Gates werden ein `if`, keine Schema-Änderung. Stand heute: Spalten da (`migrations/0001_init.sql`), aber noch tot – kein Endpunkt liefert sie aus. Das ist Absicht, solange es nichts zu gaten gibt. |
| Verbrauchszähler und Protokoll je Gerät (`ki_anfragen`, `ki_protokoll` in `meta`) | Ohne Zählung ab dem ersten Tag gibt es später keine Grundlage für eine faire Grenze – und keine Antwort auf „was habt ihr gesendet?". |
| Deckel je KI-Anfrage (`ai::MAX_EINGABE_ZEICHEN`) | Kostenbegrenzung ist beim Bauen kostenlos und nach dem ersten Kostenschock teuer. |
| Ende-zu-Ende-Verschlüsselung | Ist das Verkaufsargument gegenüber Notion & Co. und senkt Haftung und DSGVO-Aufwand (Auftragsverarbeitung nur für Ciphertext). |
| Sync-Protokoll gehört uns, Dienst ist ~300 Zeilen | Wechsel von Cloudflare auf eigene Server ist ein Nachmittag, kein Projekt. |
| Client kann gegen beliebige API-Basis-URL laufen | Selbsthosting bleibt möglich – wichtig für das Open-Core-Modell. |
| Keine stille Telemetrie | Vertrauen ist bei diesem Publikum das Produkt. |
| Lizenz **AGPL-3.0-only**, Repo öffentlich (seit 2026-09-12; davor stand hier »Lizenz offen, Repo privat«) | Eine einmal gewählte Open-Source-Lizenz lässt sich nicht zurücknehmen – deshalb war die Entscheidung aufgeschoben, bis die Architektur stand. Jetzt steht sie: verkauft werden die Dienste, nicht der Client. |

## 1a. KI als zweiter Weg zur Monetarisierung

Ergänzt Abschnitt 1: Neben bezahltem Sync kommt ein bezahlter KI-Zugang in Frage. Die
Entscheidung vom 2026-09-07 (siehe `CONCEPT.md` Abschnitt 12) hält beides offen und
getrennt:

| Weg | Wer zahlt | Was der Betreiber sieht |
|---|---|---|
| Lokales Modell | niemand | nichts, es verlässt kein Datum das Gerät |
| Eigener Schlüssel des Nutzers | der Nutzer, direkt beim Anbieter | nichts, Lotse ist nicht in der Kette |
| Zugang des Betreibers (verkäuflich) | der Betreiber, deshalb Abo | den einzeln freigegebenen Text, ohne ihn zu speichern |

Die ersten beiden bleiben kostenlos und vollwertig – „bezahlte Bequemlichkeit, nie als
Bedingung" gilt auch hier. Verkauft wird nicht „KI", sondern *schneller, robuster,
aktueller*: was ein kleines lokales Modell nachweislich nicht kann (lange oder unordentliche
Eingaben, komplexe Dokumente, mehrstufiger Werkzeuggebrauch, aktuelles Wissen) und was auf
schwacher Hardware lokal gar nicht erreichbar ist.

Was der dritte Weg zusätzlich verlangt, bevor Geld fließt: ein Auftragsverarbeitungs-
vertrag mit dem Modellanbieter samt „kein Training auf diesen Daten", die Nennung dieses
Anbieters als Unterauftragsverarbeiter, eine Grundlage für den Drittlandtransfer – und
die Bauweise „nichts speichern", die Auskunft, Löschung und Speicherfristen fast leer
laufen lässt. Datenschutzerklärung und Impressum sind davon unabhängig schon heute fällig,
weil der Sync-Dienst E-Mail-Konten führt.

Nicht geklärt und vor dem Verkaufsstart zu klären: ob der Betreiber gegenüber
Privatkunden Auftragsverarbeiter oder eigener Verantwortlicher ist. Das hängt an der
Vertragsgestaltung und gehört einmal fachlich geprüft.

## 2. Modelle, die zur Architektur passen

**Open Core mit bezahltem Sync (Empfehlung).** Vorbild Obsidian, Bitwarden, Standard Notes.
Desktop-App und Web-Client sind quelloffen und für Selbsthoster kostenlos; "Lotse Sync" als
gehosteter Dienst kostet eine kleine Monatsgebühr. Die Architektur trennt Client und Dienst
bereits sauber. Kosten pro Nutzer auf Cloudflare liegen im Cent-Bereich.

**Einmalkauf der Desktop-App plus Gratis-Sync mit Grenzen.** Vorbild Sublime, Things.
Einfacher zu kommunizieren, aber ohne wiederkehrende Einnahmen und mit Lizenz-Schlüssel-
Verwaltung, die Lotse selbst bauen müsste.

**Freemium.** Ein Gerät kostenlos, mehrere Geräte, Web-Zugriff und Anhänge kosten. Passt zu
`flags`, ist aber erfahrungsgemäß das Modell mit dem meisten Support-Aufwand.

Nicht passend: Werbung, Datenverkauf, Team-Pläne (Multi-User ist Nicht-Ziel).

## 3. Lizenz

Vor der ersten Veröffentlichung entscheiden:

| Option | Wirkung |
|---|---|
| **AGPL-3.0** für Client und Dienst | Verhindert, dass Dritte den Dienst kommerziell hosten, ohne Änderungen offenzulegen. Klassisches Open-Core-Fundament. |
| **Functional Source License (FSL)** oder BSL | Quelloffen, aber kein kommerzielles Konkurrenz-Hosting für zwei Jahre, danach automatisch Apache/MIT. Weniger Akzeptanz in der Community. |
| **Proprietär, Quelle einsehbar** | Maximale Kontrolle, minimales Vertrauen. |

**Entschieden am 2026-09-12: AGPL-3.0-only.** `LICENSE` liegt im Repo, die Cargo-Manifeste
tragen `license = "AGPL-3.0-only"`. Der Zustand davor – öffentlich einsehbar ohne Lizenz,
also „alle Rechte vorbehalten" – ist damit beendet.

Gewählt wurde `-only` statt `-or-later`: eine künftige AGPLv4 gilt damit nicht automatisch,
was die Kontrolle über den Doppelvertrieb erhält. Umstellen wäre eine Zeile.

Warum AGPL und nicht FSL/BSL oder proprietär: Die realistische Bedrohung ist nicht, dass
jemand Lotse selbst baut – das darf er bei Open Core ohnehin und kostet nichts. Sie ist,
dass jemand eine **geschlossene** Fassung als Dienst anbietet. Genau das verhindert §13.
Dazu kommt das Argument aus Abschnitt 1: Vertrauen ist bei diesem Publikum das Produkt, und
ein Krypto-Client, dessen Code niemand prüfen kann, ist in dieser Nische schwer zu verkaufen.
Die drei Vorbilder aus Abschnitt 2 – Obsidian, Bitwarden, Standard Notes – sind alle
quelloffen oder quell-einsehbar.

Zur Sorge, eine offene Lizenz verbaue den späteren bezahlten KI-Zugang: Sie tut es nicht.
Als alleiniger Urheber darf derselbe Code zusätzlich anders lizenziert werden. Eng wird es
erst, wenn Fremdbeiträge ohne Beitragsvereinbarung dazukommen – wer sich die Möglichkeit
offenhalten will, braucht ab dem ersten fremden Pull Request ein DCO oder eine CLA.
Unwiderruflich ist nur, was einmal veröffentlicht wurde: diese Fassung bleibt unter der
gewählten Lizenz, künftige müssen es nicht.

## 4. Was vor einer Veröffentlichung nötig wäre

- Markenrecherche "Lotse" (DPMA, EUIPO) und Domain.
- Impressum, Datenschutzerklärung, Auftragsverarbeitungsvertrag mit Cloudflare.
- Zahlungsabwicklung über einen Merchant of Record (Paddle, Lemon Squeezy), damit
  EU-Umsatzsteuer nicht selbst abgeführt werden muss.
- Code-Signierung und Notarisierung für macOS und Windows.
- Externe Prüfung der Kryptografie-Komposition (nicht der Primitive), bevor fremde
  Geheimnisse damit verwaltet werden.
- Cloudflare Workers Paid Plan (5 USD/Monat) ab dem ersten zahlenden Nutzer, weil die
  Gratis-Limits dann verbindlich überschritten werden dürfen.

## 5. Was nicht getan wird

Kein Bezahl-Code, keine Lizenzschlüssel, keine Konten-Pläne mit Wirkung, keine
Marketing-Seite, solange das Abbruchkriterium aus `NON_GOALS.md` nicht bestanden ist.
Zuerst muss Lotse für eine Person unverzichtbar sein.
