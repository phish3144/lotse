# Sync-Protokoll

Was genau zwischen Lotse und dem Abgleichdienst über die Leitung geht. Wer Lotse nur
benutzt, braucht **[[Abgleich]]**. Diese Seite ist für den, der den Dienst selbst
betreiben, mitlesen oder nachrechnen will, ob die Zusage »der Dienst sieht nichts«
stimmt.

Die maßgebliche Fassung steht in `docs/SYNC_PROTOCOL.md` im Repository. Hier steht
dasselbe in Prosa.

---

## Der Umschlag

Die einzige Einheit, die übertragen wird. Alles Inhaltliche darin ist Ciphertext.

| Feld | Sichtbar für den Dienst | Inhalt |
|---|---|---|
| `id` | ja | ULID des Datensatzes |
| `kind` | ja | `project`, `note`, `reference`, `vault_entry`, `device` |
| `hlc` | ja | Zeitstempel der Hybrid Logical Clock |
| `device_id` | ja | welches Gerät geschrieben hat |
| `deleted` | ja | Löschmarke |
| `format_version` | ja | Fassung der Krypto-Komposition |
| `nonce` | ja | 24 Byte, Base64 |
| `ciphertext` | **nein** | der eigentliche Datensatz |
| `server_seq` | ja | laufende Nummer, vom Dienst gesetzt |

Der Dienst sieht also: dass es einen Datensatz dieser Art gibt, wann er zuletzt geändert
wurde und von welchem Gerät. Er sieht **nicht** den Titel, den Text, den Kurs, keine
Zugangsdaten, keine Pfade, keine Adressen.

Was das an Metadaten preisgibt und warum das der bewusst gewählte Kompromiss ist, steht
in [[Sicherheit]].

### Die HLC

Format: `<unix_ms:013>-<counter:04>-<device_id:26>`, zum Beispiel
`1789131234567-0003-01M28KAPZG862XWZBTR8V68C2D`.

Sie ist lexikografisch sortierbar, und sie ist **nie kleiner als jede bereits gesehene**
HLC – auch wenn die Systemuhr zurückspringt, etwa nach einem NTP-Sprung oder weil jemand
die Zeitzone umstellt. Der Zähler in der Mitte trennt zwei Schreibvorgänge in derselben
Millisekunde.

---

## Die Konfliktregel

Eine Regel, überall dieselbe: **pro `id` gewinnt der Umschlag mit der lexikografisch
größten HLC.** Der Dienst speichert nur diesen; die Clients wenden dieselbe Regel lokal
an. Damit braucht es keine Sperren, keine Sitzungen, keine Reihenfolge.

Drei Feinheiten:

**Notizen sind append-only.** Das einzige veränderliche Feld ist `erledigt_am`. Wenn
zwei Geräte denselben Faden abhaken, wollten beide dasselbe – der Konflikt ist harmlos.

**Ein verlorener Push verschwindet nicht stumm.** Bei einem Vorhaben schreibt Lotse die
unterlegene Fassung als Notiz mit Quelle `sync` ins Logbuch: *»Konflikt: Gerät X hatte
Kurs = …«*. Die Alternative – die Fassung wegwerfen – wäre Datenverlust, den niemand
bemerkt.

**Push ist idempotent.** Ein Umschlag mit gleicher oder kleinerer HLC als die gespeicherte
wird als angenommen gemeldet, aber nicht geschrieben. Ein abgebrochener Abgleich darf
also einfach wiederholt werden.

---

## Endpunkte

Basis: `https://<dienst>/v1`. Alle Bodies JSON, Fehler als
`{"error": "<code>", "message": "<text>"}`.

### Konto

| Methode | Pfad | Wofür |
|---|---|---|
| POST | `/v1/auth/register` | Konto anlegen. Sendet Salt, KDF-Parameter, Auth-Schlüssel, Wiederherstellungs-Auth-Schlüssel und die **gewrappten** Kontoschlüssel. |
| GET | `/v1/auth/prelogin` | Salt und KDF-Parameter zu einer Adresse – nötig, um den Auth-Schlüssel überhaupt lokal berechnen zu können. |
| POST | `/v1/auth/login` | Anmelden, liefert Sitzungstoken und den gewrappten Kontoschlüssel. |
| POST | `/v1/auth/logout` | Abmelden. |
| POST | `/v1/auth/password` | Passwort wechseln. Widerruft **alle anderen** Sitzungen. |
| POST | `/v1/auth/recover` | Erster Schritt der Wiederherstellung: liefert den über den Wiederherstellungscode gewrappten Kontoschlüssel. |
| POST | `/v1/auth/recover/complete` | Zweiter Schritt: neues Passwort setzen. Widerruft **alle** Sitzungen, auch die eigene. |
| GET | `/v1/devices` | Geräte des Kontos. |
| DELETE | `/v1/devices/:id` | Gerät widerrufen; seine Sitzungen verfallen. |

Bemerkenswert an `register` und `login`: der Dienst bekommt nie das Passwort, sondern
einen aus ihm abgeleiteten Auth-Schlüssel. Den hasht er noch einmal – mit PBKDF2-SHA256,
600 000 Runden, eigenem Salt – und speichert nur den Hash. Ein Dump der Datenbank taugt
damit nicht als Login.

Warum PBKDF2 auf dem Server und nicht Argon2id wie auf dem Client? Der Worker hat 10 ms
CPU pro Anfrage; Argon2id mit 64 MiB passt dort nicht hinein. Die eigentliche
Absicherung trägt ohnehin das clientseitige Argon2id – der Server-Hash verhindert nur den
direkten Missbrauch eines Dumps. Details in
[[Schlüssel und Krypto|Schluessel-und-Krypto]].

### Abgleich

| Methode | Pfad | Wofür |
|---|---|---|
| POST | `/v1/sync/push` | Umschläge hochschieben, höchstens 500 je Anfrage. Antwort nennt Angenommene, Abgelehnte mit Grund und die neue Server-Sequenz. |
| GET | `/v1/sync/pull` | Umschläge ab einer Sequenz holen. `since` ist exklusiv, `limit` standardmäßig 500, höchstens 1000. |
| GET | `/v1/sync/status` | Server-Sequenz, Zahl der Datensätze, belegte Bytes für Anhänge. |

### Anhänge

| Methode | Pfad | Wofür |
|---|---|---|
| PUT | `/v1/blobs/:id` | Roh-Ciphertext hochladen, höchstens 100 MiB. |
| GET | `/v1/blobs/:id` | Herunterladen. |
| DELETE | `/v1/blobs/:id` | Löschen. |

Der Dienst kennt zu einem Anhang nur Konto, ID und Größe. Zu welchem Vorhaben er gehört,
steht im verschlüsselten Referenz-Datensatz mit Typ `anhang`.

---

## Grenzen

| Grenze | Wert | Warum |
|---|---|---|
| Umschläge je Push | 500 | Antwortzeit des Workers |
| Ciphertext je Umschlag | 256 KiB | Ein einzelner Logbuch-Eintrag wird nie so groß; alles darüber wäre ein Anhang. |
| Umschläge je Pull | 500 Standard, 1000 Maximum | |
| Anhang | 100 MiB | |
| `last_seen` des Geräts | höchstens alle 60 s geschrieben | Sonst wäre jeder Abgleich ein Schreibvorgang mehr. |

---

## Der Ablauf im Client

Angestoßen wird zwei Sekunden nach der letzten lokalen Änderung, beim Start, wenn das
Fenster den Fokus bekommt, und alle fünf Minuten.

1. Lokale Umschläge ab der letzten gepushten Sequenz sammeln (höchstens 500).
2. `push`.
3. Gepushte Sequenz merken.
4. `pull` in einer Schleife, bis nichts mehr nachkommt. Jeder Umschlag mit höherer HLC
   als der lokalen wird entschlüsselt, lokal eingesetzt und bei Bedarf mit einer
   Konfliktnotiz versehen.
5. Volltextindex nachziehen.

**Offline** werden die Schritte 2 bis 4 übersprungen; die Änderungen sammeln sich im
lokalen Änderungsprotokoll. Es gibt keine Obergrenze für die Dauer – drei Monate ohne
Netz sind kein Sonderfall, sondern der Normalbetrieb eines Werkzeugs, das man nicht
jeden Tag anfasst.

---

## Was der Dienst nie tut

- **Keine Bodies protokollieren.** Keine Tokens, keine Ciphertexte, keine Adressen. Das
  ist Regel, nicht Einstellung.
- **Nicht entschlüsseln.** Er hat die Schlüssel nicht. Nicht »er benutzt sie nicht« – er
  hat sie nicht.
- **Nichts rechnen.** Kein Suchindex, keine Statistik, keine Verdichtung. Wäre auch
  nicht möglich.

Ein eigener Dienst ist deshalb überschaubar: `services/sync-worker` ist ein Cloudflare
Worker mit D1 und R2, und wer ihn selbst betreiben will, braucht nur dieses Protokoll
umzusetzen. Siehe [[Für Entwickler|Fuer-Entwickler]].
