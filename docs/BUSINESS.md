# Lotse – Überlegungen zu einer späteren Monetarisierung

Stand: 2026-09-06. Lotse wird zuerst für eine Person gebaut und läuft kostenlos. Dieses
Dokument hält fest, welche Entscheidungen heute getroffen werden, damit eine spätere
Monetarisierung möglich bleibt, ohne dass heute Bezahl-Code entsteht.

## 1. Was heute schon dafür getan wird

| Entscheidung | Warum sie jetzt billig und später teuer ist |
|---|---|
| `account_id` auf jedem Datensatz im Sync-Dienst | Mandantenfähigkeit nachträglich einzubauen ist eine Migration aller Daten. |
| `plan` und `flags` am Konto | Feature-Gates werden ein `if`, keine Schema-Änderung. |
| Ende-zu-Ende-Verschlüsselung | Ist das Verkaufsargument gegenüber Notion & Co. und senkt Haftung und DSGVO-Aufwand (Auftragsverarbeitung nur für Ciphertext). |
| Sync-Protokoll gehört uns, Dienst ist ~300 Zeilen | Wechsel von Cloudflare auf eigene Server ist ein Nachmittag, kein Projekt. |
| Client kann gegen beliebige API-Basis-URL laufen | Selbsthosting bleibt möglich – wichtig für das Open-Core-Modell. |
| Keine stille Telemetrie | Vertrauen ist bei diesem Publikum das Produkt. |
| Lizenz noch offen, Repo privat | Eine einmal gewählte Open-Source-Lizenz lässt sich nicht zurücknehmen. |

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

Bis zur Entscheidung: keine `LICENSE`-Datei, Repo privat. Alle Rechte liegen beim Autor.

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
