# Abgleich zwischen Geräten

Standardmäßig bleibt alles auf einem Rechner. Das ist ein gültiger Zustand, kein Mangel.

Wer mehrere Geräte hat, kann einen **Sync-Dienst** dazwischenschalten. Der Dienst sieht
dabei **nur verschlüsselte Umschläge** – keine Titel, keine Texte, keine Zugänge.

## Was der Dienst sieht

| Sichtbar | Nicht sichtbar |
|---|---|
| Ein Zähler je Datensatz (Reihenfolge) | Titel, Kurs, Logbuch-Texte |
| Die Größe der Umschläge | Referenzen, Zugänge, Tags |
| Zeitpunkte des Abgleichs | Wie viele Vorhaben, welcher Art |
| Geräte-Kennungen und E-Mail | irgendein Klartext |

Die Schlüssel werden aus deinem Master-Passwort abgeleitet und verlassen das Gerät nie.
Ein Betreiber, der die Datenbank kopiert, hat Umschläge – und sonst nichts.

## Einrichten

### Erstes Gerät

Einstellungen → *Abgleich* → Adresse des Dienstes und E-Mail. Lotse fragt den
**Wiederherstellungscode** ab: er beweist, dass du das Konto besitzt, ohne dass das
Master-Passwort das Gerät verlässt.

```bash
lotse sync register --url https://api.example --email ich@example
```

### Weiteres Gerät

Dort **kein** neues Konto anlegen, sondern anmelden:

```bash
lotse sync login --url https://api.example --email ich@example
```

Master-Passwort eingeben, alles wird heruntergeladen und entschlüsselt. Für Einträge der
Stufe *nur Desktop* zusätzlich den **Desktop-Schlüssel**.

## Wie oft

Auf Knopfdruck (*Jetzt abgleichen*) oder über `lotse sync jetzt`. Erst pushen, dann
pullen.

## Konflikte

Zwei Geräte ändern dasselbe Vorhaben, ohne zwischendurch abzugleichen. Lotse löst das
mit einer **Hybrid Logical Clock**: jeder Datensatz trägt einen Zeitstempel, der auch
bei ungenau gehenden Uhren eine eindeutige Reihenfolge ergibt. Die jüngere Änderung
gewinnt je Datensatz.

Beim Logbuch fällt das kaum auf: Einträge werden angehängt, nicht geändert. Spürbar wird
es nur, wenn du denselben Kurs an zwei Geräten gleichzeitig umschreibst.

Der Abgleich meldet nach jedem Lauf, was er getan hat: gepusht, übernommen, verworfen,
Konflikte.

## Geräte verwalten

```bash
lotse sync geraete
lotse sync widerrufen <geraet-id>
```

Ein widerrufenes Gerät kann nicht mehr abgleichen; seine Sitzungen verfallen. Die Daten
darauf bleiben, bis jemand sie löscht – Widerruf ist keine Fernlöschung.

Sitzungstoken: 32 Byte Zufall, serverseitig gehasht, 30 Tage auf dem Desktop.

## Eigener Dienst

Der Dienst liegt im Repository unter `services/sync-worker` – ein Cloudflare Worker mit
D1 und R2. Er läuft auf dem Gratisplan von Cloudflare; die Schnittstelle steht in
`docs/SYNC_PROTOCOL.md`.

Er loggt **nie** Bodies oder Tokens.

Ein gehosteter Dienst kommt später als bezahlte Bequemlichkeit – nie als Bedingung.

## Ohne Abgleich mehrere Geräte?

Geht, aber nicht über einen Cloud-Ordner: zwei Rechner, die gleichzeitig auf dieselbe
SQLite-Datei schreiben, beschädigen sie. Wer keinen Dienst will, nimmt den
**[[Bundle-Export|Export-und-Fluchtweg]]** und trägt ihn von Hand hinüber.
