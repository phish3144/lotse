# Lotse – Sync-Protokoll v1

Stand: 2026-09-06. Verbindliche Schnittstelle zwischen Clients (`lotse-core`) und dem
Sync-Dienst (`services/sync-worker`). Der Dienst ist bewusst dumm: Er speichert Ciphertext,
vergibt Sequenznummern und liefert Änderungen seit einer Sequenz aus. Alle Semantik liegt
im Client.

## 1. Begriffe

- **Konto** – ein Nutzer, identifiziert durch `account_id` (ULID). Jede Tabelle im Dienst
  trägt `account_id`.
- **Gerät** – ein installierter Client (Desktop, Web-Browser, CLI teilt sich das Gerät mit
  der Desktop-App). `device_id` (ULID), vom Client erzeugt.
- **Datensatz** – die Sync-Einheit. Entspricht einer Entität (`project`, `note`,
  `reference`, `vault_entry`, `device`, `settings`). Inhalt ist Ciphertext.
- **HLC** – Hybrid Logical Clock, String der Form `<unix_ms:013>-<counter:04>-<device_id>`.
  Lexikografisch sortierbar. Bestimmt Last-Writer-Wins.
- **server_seq** – pro Konto monoton steigende Ganzzahl, vom Dienst beim Annehmen vergeben.
  Clients merken sich die zuletzt gesehene Sequenz.

## 2. Datensatz-Umschlag

```json
{
  "id": "01J9Z2Q7K5X8N3M4V6B7C8D9E0",
  "kind": "note",
  "hlc": "1757174400123-0001-01J9Z2Q7K5X8N3M4V6B7C8D9E1",
  "device_id": "01J9Z2Q7K5X8N3M4V6B7C8D9E1",
  "deleted": false,
  "format_version": 1,
  "nonce": "<base64, 24 B>",
  "ciphertext": "<base64>",
  "server_seq": 4711
}
```

- `ciphertext` = XChaCha20-Poly1305(Datensatz-Schlüssel, `nonce`, JSON des Klartext-
  Datensatzes, AAD). AAD = `format_version ‖ kind ‖ id` als UTF-8 mit `\x1f` als Trenner.
- Bei `deleted = true` sind `nonce` und `ciphertext` leer; der Umschlag ist ein Tombstone und
  bleibt dauerhaft (Kompaktierung siehe 7).
- Der Dienst prüft nur: Base64 gültig, `ciphertext` ≤ 256 KiB, `kind` aus der erlaubten
  Liste, `hlc` wohlgeformt.
- `server_seq` ist nur in Antworten des Dienstes gesetzt.

## 3. Konflikte

- Pro `id` gewinnt der Umschlag mit dem lexikografisch größten `hlc`. Der Dienst speichert
  nur diesen (Upsert mit `WHERE excluded.hlc > hlc`). Clients wenden dieselbe Regel lokal an.
- `note` ist append-only; die einzigen veränderlichen Felder sind `erledigt_am`. Kollisionen
  dort sind harmlos (beide Seiten wollten "erledigt").
- Ein Client, dessen Push verliert, erfährt das beim nächsten Pull (der gewinnende Umschlag
  kommt zurück). Bei `project` wird die verlierende Fassung als Notiz mit `quelle = sync` ins
  Logbuch geschrieben ("Konflikt: Gerät X hatte Kurs = …"), damit nichts stumm verschwindet.

## 4. Endpunkte

Basis-URL: `https://api.<domain>/v1`. Alle Bodies JSON. Fehler als
`{ "error": "<code>", "message": "<text>" }`.

### Konto

| Methode | Pfad | Body | Antwort |
|---|---|---|---|
| POST | `/auth/register` | `{ email, auth_key, recovery_auth_key, salt, kdf: {m,t,p}, wrapped_account_key, wrapped_account_key_recovery, device: {id, name, platform} }` | `{ account_id, session_token }` |
| POST | `/auth/login` | `{ email, auth_key, device: {id, name, platform} }` | `{ account_id, session_token, salt, kdf, wrapped_account_key }` |
| GET | `/auth/prelogin?email=` | – | `{ salt, kdf }` (nötig, um `auth_key` clientseitig zu berechnen) |
| POST | `/auth/logout` | – | `204` |
| POST | `/auth/password` | `{ old_auth_key, new_auth_key, new_salt, new_kdf, wrapped_account_key }` | `204`; widerruft alle anderen Sitzungen |
| POST | `/auth/recover` | `{ email, recovery_auth_key }` | `{ wrapped_account_key_recovery, salt, kdf }` |
| GET | `/devices` | – | `[{ id, name, platform, created_at, last_seen_at }]` |
| DELETE | `/devices/:id` | – | `204`; widerruft dessen Sitzungen |

`auth_key` ist der HKDF-abgeleitete Auth-Schlüssel (Base64, 32 B). Der Dienst speichert
nur einen gesalzenen Hash davon (siehe unten). `session_token` wird als
`Authorization: Bearer <token>` gesendet; der Dienst speichert `sha256(token)`.

`recovery_auth_key` = HKDF(Recovery Key, "lotse/recovery-auth"); der Dienst speichert dessen
Hash bei der Registrierung mit. E-Mail-Adressen werden vom Dienst kleingeschrieben und getrimmt.

Serverseitiges Hashing: Der Dienst hasht `auth_key` und `recovery_auth_key` mit PBKDF2-SHA256
(600 000 Iterationen, eigenes Salt) statt Argon2id, weil der Worker nur 10 ms CPU pro Anfrage
hat. Die Sicherheit trägt das clientseitige Argon2id; der Server-Hash verhindert nur, dass ein
DB-Dump direkt als Login taugt.

### Sync

| Methode | Pfad | Body | Antwort |
|---|---|---|---|
| POST | `/sync/push` | `{ records: [Umschlag…] }` (≤ 500) | `{ accepted: [id…], rejected: [{id, reason}], server_seq }` |
| GET | `/sync/pull?since=<seq>&limit=<n>` | – | `{ records: [Umschlag…], next_seq, has_more }` |
| GET | `/sync/status` | – | `{ server_seq, record_count, blob_bytes }` |

- `since` ist exklusiv. `limit` Default 500, max 1000.
- `push` ist idempotent: ein Umschlag mit gleichem oder kleinerem `hlc` als dem gespeicherten
  wird als `accepted` gemeldet, aber nicht geschrieben.
- Ein Client pusht erst, dann pullt er, dann wendet er Regel 3 lokal an.

### Anhänge

| Methode | Pfad | Body | Antwort |
|---|---|---|---|
| PUT | `/blobs/:id` | Roh-Ciphertext, `Content-Length` ≤ 100 MiB | `201` |
| GET | `/blobs/:id` | – | Roh-Ciphertext |
| DELETE | `/blobs/:id` | – | `204` |

Blob-IDs sind ULIDs; die Zuordnung zu Projekten steht im (verschlüsselten) `reference`-
Datensatz mit `typ = anhang`. Der Dienst kennt nur `account_id`, `id`, Größe.

## 5. Ablauf im Client

```
alle 2 s nach letzter lokaler Änderung, beim Start, bei Fokus, alle 5 min:
  1. changes = lokale Umschläge mit local_seq > last_pushed_seq  (max 500 pro Request)
  2. POST /sync/push
  3. last_pushed_seq = höchste gepushte local_seq
  4. loop:
       GET /sync/pull?since=last_server_seq
       für jeden Umschlag: wenn hlc > lokal → entschlüsseln, lokal upsert, Konfliktnotiz falls nötig
       last_server_seq = next_seq
     bis has_more == false
  5. FTS-Index inkrementell aktualisieren, Spiegel nachziehen
```

Offline: Schritte 2–4 werden übersprungen, lokale Änderungen sammeln sich im
Änderungsprotokoll. Es gibt keine Obergrenze für die Offline-Dauer.

## 6. Speicher im Dienst (D1)

```sql
CREATE TABLE accounts (
  id TEXT PRIMARY KEY, email TEXT UNIQUE NOT NULL,
  auth_hash TEXT NOT NULL, recovery_auth_hash TEXT NOT NULL,
  salt TEXT NOT NULL, kdf_m INTEGER NOT NULL, kdf_t INTEGER NOT NULL, kdf_p INTEGER NOT NULL,
  wrapped_account_key TEXT NOT NULL, wrapped_account_key_recovery TEXT NOT NULL,
  plan TEXT NOT NULL DEFAULT 'free', flags TEXT NOT NULL DEFAULT '{}',
  created_at INTEGER NOT NULL
);
CREATE TABLE devices (
  id TEXT PRIMARY KEY, account_id TEXT NOT NULL REFERENCES accounts(id),
  name TEXT NOT NULL, platform TEXT NOT NULL,
  created_at INTEGER NOT NULL, last_seen_at INTEGER NOT NULL
);
CREATE TABLE sessions (
  token_hash TEXT PRIMARY KEY, account_id TEXT NOT NULL, device_id TEXT NOT NULL,
  created_at INTEGER NOT NULL, expires_at INTEGER NOT NULL
);
CREATE TABLE records (
  account_id TEXT NOT NULL, id TEXT NOT NULL, kind TEXT NOT NULL,
  hlc TEXT NOT NULL, device_id TEXT NOT NULL, deleted INTEGER NOT NULL DEFAULT 0,
  format_version INTEGER NOT NULL, nonce TEXT NOT NULL, ciphertext TEXT NOT NULL,
  server_seq INTEGER NOT NULL,
  PRIMARY KEY (account_id, id)
);
CREATE INDEX records_seq ON records(account_id, server_seq);
CREATE TABLE seq (account_id TEXT PRIMARY KEY, value INTEGER NOT NULL);
CREATE TABLE blobs (
  account_id TEXT NOT NULL, id TEXT NOT NULL, size INTEGER NOT NULL,
  created_at INTEGER NOT NULL, PRIMARY KEY (account_id, id)
);
```

`plan` und `flags` sind ab Tag 1 da und in v1 immer `free` / `{}`.

## 7. Kompaktierung

Tombstones bleiben mindestens 90 Tage. Danach darf der Dienst Tombstones entfernen, deren
`server_seq` kleiner ist als die kleinste `last_server_seq` aller Geräte des Kontos, die in
den letzten 90 Tagen gesehen wurden. Geräte, die länger fehlen, machen beim nächsten Sync
einen vollständigen Pull (`since=0`).

## 8. Grenzen des Gratisplans und ihre Folgen

| Grenze | Wert | Folge im Design |
|---|---|---|
| D1 Schreibvorgänge/Tag | 100 000 Zeilen | Batching, Debounce 2 s; ein Push = eine Transaktion |
| D1 Lesevorgänge/Tag | 5 000 000 Zeilen | Pull ist indexbasiert (`records_seq`), kein Full Scan |
| Worker-Anfragen/Tag | 100 000 | Sync-Intervall 5 min im Leerlauf ≈ 300 Anfragen/Tag/Gerät |
| Worker CPU/Anfrage | 10 ms | Argon2id läuft **nicht** im Worker. Der Dienst hasht `auth_key` und `recovery_auth_key` mit PBKDF2-SHA256 (600 000 Iterationen, WebCrypto) nur bei Login, Registrierung und Wiederherstellung; die Sicherheit trägt das clientseitige Argon2id. |
| R2 Speicher | 10 GB | Anhänge über 25 MB werden nur referenziert |

## 9. Versionierung

`GET /v1/…` ist Version 1. Inkompatible Änderungen bekommen `/v2`. `format_version` im
Umschlag versioniert die Kryptografie unabhängig davon.
