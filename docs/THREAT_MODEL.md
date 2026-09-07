# Lotse – Bedrohungsmodell und Kryptografie

Stand: 2026-09-06. Dieses Dokument benennt, wogegen Lotse verteidigt, wogegen ausdrücklich
nicht, und wie die Kryptografie aufgebaut ist. Jede Änderung an der Schlüsselhierarchie
erhöht `FORMAT_VERSION` und wird hier protokolliert.

## 1. Schutzgüter

| Gut | Vertraulichkeit | Integrität | Verfügbarkeit |
|---|---|---|---|
| Tresor-Einträge (Zugangsdaten, Lizenzen, sensible Notizen) | hoch | hoch | mittel |
| Logbücher, Kurs, Referenzen, Pfade | mittel | hoch | hoch |
| Master-Passwort, Desktop-Schlüssel, Wiederherstellungscode | höchst | – | – |
| Metadaten beim Sync (Zeitstempel, IDs, Größen, Gerätenamen) | niedrig | mittel | – |
| An ein KI-Ziel gesendeter Inhalt (Brief, offene Fäden, künftig mehr) | mittel | – | – |

## 2. Angreifer und Szenarien

### Verteidigt

| Szenario | Antwort |
|---|---|
| **Laptop verloren oder gestohlen** (ausgeschaltet oder gesperrt) | Lokale Datenbank ist SQLCipher; Schlüssel nur im OS-Schlüsselbund bzw. aus Master-Passwort. Ohne Entsperren nichts lesbar. |
| **Sync-Dienst kompromittiert** (Cloudflare-Konto, D1-Dump, R2-Bucket) | Nur Ciphertext, IDs, HLC-Zeitstempel, Größen, Gerätenamen. Kein Schlüsselmaterial außer dem passwort-gewrappten Account-Schlüssel, der ohne Passwort wertlos ist. |
| **Passives Mitlesen im Netz** | TLS überall; zusätzlich ist der Inhalt bereits verschlüsselt. |
| **Server versucht, Ciphertext zu manipulieren** | AEAD (XChaCha20-Poly1305) mit `id`, `kind`, `format_version` als Additional Authenticated Data; ein vertauschter oder veränderter Datensatz wird beim Entschlüsseln erkannt. |
| **Angreifer kennt das Master-Passwort, aber sitzt an einem fremden Rechner** | `nur_desktop`-Einträge brauchen zusätzlich den Desktop-Schlüssel, der nie den Server oder Browser erreicht. |
| **Blick über die Schulter** | Werte verdeckt bis Klick, Auto-Lock, Zwischenablage nach 20 s geleert. |
| **Vergessenes Master-Passwort** | Wiederherstellungscode, beim Setup einmal angezeigt, mit Pflicht zur kalten Wiedereingabe vor Abschluss des Setups. |
| **Geräteverlust bei aktiver Sitzung** | Gerät im Konto widerrufen; Sitzungstoken verfallen; Passwortwechsel wrappt den Account-Schlüssel neu. |
| **Alte Backups/Sync-Snapshots eines gelöschten Tresor-Eintrags** | Envelope-Encryption pro Eintrag; Löschen vernichtet den Eintragsschlüssel (Crypto-Shredding). |
| **KI-Ziel liest mit oder protokolliert** | Der gesendete Text verlässt das Gerät im Klartext – daran ändert keine Verschlüsselung etwas, das ist die Natur der Sache. Verteidigt wird deshalb an drei Stellen: standardmäßig aus; vor jedem Senden ist der vollständige Text sichtbar; der Tresor ist strukturell unerreichbar (`scripts/modulgrenzen.sh`). Wer nichts senden will, nimmt ein lokales Modell — dann verlässt nichts den Rechner. Ein Protokoll in den Einstellungen hält fest, was wann an welches Ziel ging; der Text selbst wird dabei nicht aufbewahrt. |
| **Bösartige Abhängigkeit** | Nur RustCrypto/`age`/`zeroize`; Versionen gepinnt; `cargo-deny` und `cargo-audit` in CI; keine Fremdskripte in der Web-App. |
| **Manipuliertes Update** | Solange die Bauten unsigniert sind, aktualisiert sich Lotse **nicht** selbst: es nennt nur die neue Version und die Datei dazu, herunterladen und installieren tut der Mensch. Ein Programm, das sich selbst mit unsignierten Binärdaten überschreibt, wäre der bequemste Angriffsweg überhaupt. Sobald signierte Bauten existieren (Tauri-Updater mit minisign, privater Schlüssel offline bzw. im Passwortmanager), kann daraus ein echter Updater werden. |

### Nicht verteidigt (bewusst, dokumentiert)

| Szenario | Warum nicht |
|---|---|
| **Malware auf dem eigenen, entsperrten Gerät** | Wer den Prozess kontrolliert, liest den Speicher. Kein Client-Design ändert das. |
| **Keylogger oder Bildschirmaufnahme auf einem fremden Rechner** | Das Master-Passwort und alle angezeigten `ueberall`-Werte sind dort kompromittierbar. Antwort ist die Stufe `nur_desktop` für das Wertvollste, nicht Kryptografie. |
| **Kompromittierte Auslieferung des Web-Codes** (Cloudflare-Konto oder Build) | Web-Krypto ist nur so vertrauenswürdig wie der ausgelieferte Code. Minderung: Cloudflare-Konto mit Passkey/2FA, Deploy nur aus dem Repo, strikte CSP, `nur_desktop`. Restrisiko akzeptiert. |
| **Verkehrsanalyse** (wann wurde wie viel synchronisiert) | Metadaten sind sichtbar; für einen persönlichen Dienst kein Schutzziel. |
| **Erzwungene Herausgabe des Passworts** | Kein Duress-Modus, keine versteckten Tresore. |
| **Klartext-Spiegel** | Opt-in, ausdrücklich unverschlüsselt, verlässt sich auf Festplattenverschlüsselung des OS. Wird beim Aktivieren so angezeigt. |

## 3. Schlüsselhierarchie

```
Master-Passwort (MP)                  Wiederherstellungscode (RC, 128 Bit, Base32)
        │                                      │
   Argon2id(MP, salt_user)                Argon2id(RC, salt_user)
        │                                      │
   Stretched Key SK (32 B)                Recovery Key RK (32 B)
        │
   HKDF-SHA256(SK)
   ├── info "lotse/auth"  → Auth-Schlüssel AK_auth (geht zum Server, wird dort nochmals gehasht)
   └── info "lotse/wrap"  → Wrap-Schlüssel WK (bleibt lokal)

Account-Schlüssel AK (32 B, zufällig)
   ├── wrap(WK, AK)   → auf Server und lokal gespeichert
   └── wrap(RK, AK)   → auf Server und lokal gespeichert

HKDF-SHA256(AK)
   ├── info "lotse/records"  → Datensatz-Schlüssel   (alle Sync-Datensätze außer Tresorwerten)
   ├── info "lotse/local-db" → SQLCipher-Schlüssel   (lokale Datenbank)
   ├── info "lotse/vault"    → Tresor-Schlüssel VK   (wrappt DEKs der Stufe `ueberall`)
   └── info "lotse/blobs"    → Anhang-Schlüssel

Desktop-Schlüssel DK (32 B, zufällig, einmal angezeigt, nur OS-Schlüsselbund)
   HKDF-SHA256(AK ‖ DK, info "lotse/vault-desktop") → VK_desktop (wrappt DEKs der Stufe `nur_desktop`)

Pro Tresor-Eintrag: DEK (32 B, zufällig)
   Wert = XChaCha20-Poly1305(DEK, nonce, klartext, aad)
   wrapped_dek = XChaCha20-Poly1305(VK oder VK_desktop, nonce, DEK, aad)
```

### Parameter

| Baustein | Wahl | Anmerkung |
|---|---|---|
| Passwort-KDF | Argon2id, m = 64 MiB, t = 3, p = 1 | Untergrenze; Desktop kalibriert beim Setup auf ~500 ms und speichert die Parameter im Konto-Header. Browser nutzt dieselben Parameter (WASM). |
| Salt | 16 B zufällig pro Konto | im Konto-Header, nicht geheim |
| KDF-Ableitung | HKDF-SHA256 mit festen `info`-Strings | Domain Separation zwischen allen Verwendungen |
| AEAD | XChaCha20-Poly1305, 24-Byte-Nonce zufällig | keine Nonce-Buchführung nötig |
| AAD | `format_version ‖ kind ‖ record_id` | verhindert Vertauschen von Datensätzen |
| Wrapping | AEAD mit Schlüssel als Klartext | kein eigenes Key-Wrap-Schema |
| Zufall | `getrandom` (OS-CSPRNG, im Browser WebCrypto) | |
| Speicher | `zeroize` auf allen Schlüssel-Typen, `Drop` überschreibt | |
| Export | `age` mit Passphrase (scrypt) über ein JSON-Bundle | mit generischem `age`-CLI entschlüsselbar |

### Was der Server speichert

`account_id`, `email`, einen gesalzenen PBKDF2-SHA256-Hash von AK_auth (serverseitig nochmals
gehasht, damit ein DB-Dump nicht als Login taugt; Argon2id läuft clientseitig), `salt_user`, KDF-Parameter, `wrap(WK, AK)`, `wrap(RK, AK)`,
Geräte-Liste, Sitzungstoken (gehasht), Datensätze als `{id, kind, hlc, device_id, deleted,
nonce, ciphertext, format_version, server_seq}`, Blobs als Ciphertext.

Der Server speichert **nie**: MP, SK, WK, RK, AK, DK, VK, DEKs, Klartext irgendeines Feldes,
Tresor-Titel im Klartext. (Tresor-Titel sind lokal im Klartext-Index, im Sync-Datensatz aber
innerhalb des Ciphertexts.)

## 4. Entsperren im Alltag

- **Desktop:** Beim Setup wird AK zusätzlich mit einem zufälligen Geräteschlüssel gewrappt,
  der im OS-Schlüsselbund liegt (Rust `keyring`: Keychain, Credential Manager, Secret
  Service). Alltag: OS-Login bzw. Biometrie genügt. Master-Passwort wird verlangt bei:
  Passwortwechsel, Geräte widerrufen, Wiederherstellungscode anzeigen, Export.
  Auf Linux ohne Secret Service: Fallback auf Master-Passwort bei jedem Start.
- **Web:** Immer Master-Passwort. Modus "fremder Rechner" (Default, wenn das Gerät unbekannt
  ist): keine Persistenz in IndexedDB/localStorage, Schlüssel nur im Tab-Speicher, Auto-Lock
  nach 5 Minuten Inaktivität und beim Verlassen des Tabs, `nur_desktop`-Einträge sind gar
  nicht erst sichtbar.
- **Auto-Lock** überall: Inaktivität (Desktop 15 min, Web 5 min), OS-Sperre, Tab-Verlust.
- **Zwischenablage:** nach 20 s leeren, nur wenn der Inhalt seither unverändert ist.

## 5. Web-Client-Härtung

- `Content-Security-Policy: default-src 'self'; script-src 'self' 'wasm-unsafe-eval';
  connect-src 'self' https://api.<domain>; img-src 'self' data:; frame-ancestors 'none'`
- Keine Fremdskripte, keine CDNs, keine Analytics.
- Kein `innerHTML` für Nutzertext; sanitisierender Markdown-Renderer.
- Cloudflare Access (Einmalcode per E-Mail) vor der App-Adresse. Die API-Adresse hat eigene
  Authentifizierung (Auth-Schlüssel + Sitzungstoken) und Rate-Limiting.
- Sitzungstoken: 32 B zufällig, serverseitig gehasht, Lebensdauer 30 Tage Desktop, 12 h Web,
  an Geräte-ID gebunden, widerrufbar.

## 6. Isolation innerhalb der App

Der Tresor ist ein eigenes Modul in `lotse-core` (`vault`). Folgende Module haben
**keinen** Import-Pfad dorthin, geprüft von `scripts/modulgrenzen.sh` bei jedem
CI-Lauf: `watcher`, `detect`, `mcp`, `ai`, `forge`, `git`, `kalender`, `update`,
`export::spiegel`. Token und Zugangsdaten für Fremddienste reicht die Hülle herein; die
Module holen sie nie selbst.
KI-Funktionen sehen nur, was ihnen explizit übergeben wird, und zeigen es vor dem Senden an.

## 6a. Verbindungen nach außen

Alle Abrufe (Sync-Dienst, GitHub, GitLab, Kalender, KI-Ziel) laufen über TLS mit dem
Wurzelspeicher des Betriebssystems (`ureq` mit `native-certs`), nicht über eine im
Programm mitgelieferte Liste. Das ist die Wahl, die Browser, `git` und `curl` auf
demselben Rechner ebenfalls treffen: In Netzen mit TLS-Prüfung (Firmen-Proxy) hat der
Betreiber ohnehin eine eigene CA im System, und eine mitgelieferte Liste würde Lotse dort
schlicht ausfallen lassen, ohne etwas zu schützen. Wer diese CA kontrolliert, sieht die
Abrufe bei GitHub, GitLab und dem Kalender samt der dabei gesendeten Token — dieselbe
Lage wie für `git` selbst. Der Abgleich bleibt davon unberührt: Der Sync-Dienst bekommt
nur Umschläge, deren Inhalt vor dem Senden verschlüsselt wird, und der Wiederherstellungs-
und Kontoschlüssel verlässt das Gerät nie.

Kehrseite, ausdrücklich: Auf einem System ohne eingerichteten Wurzelspeicher (manche
minimalen Container) schlagen alle HTTPS-Abrufe fehl. Auf Desktop-Systemen — dem Ziel
dieser App — gibt es diesen Fall nicht.

Welche dieser Verbindungen ohne Zutun des Nutzers entstehen, steht vollständig in
`README.md` unter »Wann Lotse von allein ins Netz geht«. Jede davon ist abschaltbar,
keine überträgt Inhalte: der Update-Hinweis und die Abfrage der Gegenseite senden nur
die Anfrage selbst, der Kalender wird gelesen, nicht beschrieben.

## 6b. KI: was gesendet wird und was nicht

Der Grundsatz, an dem sich jede künftige KI-Fähigkeit messen lassen muss:

1. **Standardmäßig aus.** Keine KI-Funktion läuft, bevor ein Mensch sie eingeschaltet hat.
2. **Einzelfreigabe, nicht Generalvollmacht.** Freigegeben wird ein Vorgang, nicht ein
   Bestand. Ein privates Logbuch kann Gesundheit, Weltanschauung oder Familie enthalten
   (Artikel 9 DSGVO); deshalb ist die Freigabe je Aufruf die Bauart, nicht die Ausnahme.
3. **Vor dem Senden sichtbar.** Der vollständige Text, der das Gerät verlassen würde,
   ist abrufbar, bevor etwas geschieht (`ai::anfrage_text`).
4. **Kein Tresor.** `ai` hat keinen Import-Pfad zu `vault`, geprüft in CI.
5. **Nichts wird stillschweigend behalten.** Der gesendete Text wird nicht aufbewahrt.
   Protokolliert wird nur, dass und wohin gesendet wurde, samt Umfang — das ist der
   Nachweis, den Rechenschaftspflicht verlangt, und zugleich das, was der Nutzer sehen
   können muss.
6. **Ein Deckel von Anfang an.** `ai::MAX_EINGABE_ZEICHEN` begrenzt eine einzelne
   Anfrage. Ein Deckel, der erst nach dem ersten Kostenschock eingezogen wird, ist teuer.
7. **Ein gehosteter KI-Vermittler bleibt ein eigener Dienst.** Er wird nie Teil des
   Sync-Dienstes. Nur so bleibt der Satz »der Sync-Dienst sieht nur Umschläge« wahr,
   während ein zweiter, klar benannter Dienst Klartext verarbeitet, den ihm jemand
   ausdrücklich gegeben hat.

Die Zusage »Dateiinhalte liest Lotse nicht« (`detect`, `watcher`, Landing Page) gilt für
die automatische Erkennung und den Ordner-Beobachter und bleibt dort gültig. Eine spätere
Funktion, die Dateien deutet, ist etwas anderes: ausdrücklich benannt, einzeln
freigegeben, nie im Hintergrund.

## 7. Betrieb und Konto-Sicherheit

- Cloudflare- und GitHub-Konto mit Passkey oder TOTP (in Proton Pass).
- Signaturschlüssel für Updates nie im Repo; öffentlicher Schlüssel im Tauri-Config.
- Wiederherstellungscode und Desktop-Schlüssel gehören in Proton Pass. Das Setup sagt das
  ausdrücklich und verlangt die kalte Wiedereingabe des Wiederherstellungscodes.
- Backup ist der E2E-verschlüsselte Bestand auf dem Sync-Dienst plus optionaler
  periodischer `age`-Export in einen Ordner der Wahl (z. B. Proton Drive).

## 8. Formatversionen

| `FORMAT_VERSION` | Datum | Inhalt |
|---|---|---|
| 1 | 2026-09-06 | Hierarchie wie oben. |

## 9. Behobene Mängel

| Datum | Mangel | Behebung |
|---|---|---|
| 2026-09-07 | **Wiederherstellung erreichte den Dienst nicht.** `konto_wiederherstellen` setzte lokal ein neues Passwort, meldete das aber nirgends: `/auth/password` verlangt den alten Auth-Schlüssel, und den hat nicht, wer sein Passwort vergessen hat. Ein Weg, eine Wiederherstellung beim Dienst abzuschließen, fehlte im Protokoll ganz. Folge: Das Gerät kannte das neue Passwort, der Dienst weiter das alte; aufgefallen wäre es erst beim nächsten Login auf einem zweiten Gerät – erneut im schlechtesten Moment. | Neuer Endpunkt `POST /auth/recover/complete`, beglaubigt mit dem `recovery_auth_key` statt mit dem alten Passwort, setzt Salt, KDF, Wrapping und beide Auth-Hashes neu und widerruft **alle** Sitzungen. Die Hülle ruft ihn vor dem lokalen Schreiben auf: lehnt der Dienst ab, bleibt lokal alles, wie es war. Tests: `completes a recovery and leaves the new password usable`, `rejects a recovery completion with the wrong recovery_auth_key`. |
| 2026-09-07 | **Passwortwechsel konnte Gerät und Dienst dauerhaft trennen.** Der Dienst wurde vor der lokalen Datei aktualisiert. Schlug das Schreiben fehl, kannte der Dienst das neue Passwort und das Gerät das alte; ein zweiter Versuch bräuchte den alten Auth-Schlüssel, den der Dienst nicht mehr akzeptiert. | Erst lokal schreiben, dann den Dienst; lehnt er ab, wird die alte Kontodatei zurückgeschrieben. Gelingt auch das nicht, sagt die Meldung genau, wie der Zustand ist und dass der Wiederherstellungscode ihn auflöst. |
| 2026-09-07 | **Passwortwechsel entwertete den Wiederherstellungscode.** Der Recovery Key wird aus dem Wiederherstellungscode **und dem Salt** abgeleitet. `passwort_wechseln` zog einen neuen Salt, ließ `wrapped_account_key_recovery` aber unverändert; im Dienst schrieb `/auth/password` denselben neuen Salt, ohne Wrapping und `recovery_auth_hash` mitzuziehen. Danach lieferte `/auth/recover` einen neuen Salt zu einem alten Wrapping: der Code öffnete das Konto nicht mehr. Bemerkt hätte man es erst, wenn das Passwort weg ist. | `passwort_wechseln` verankert das Recovery-Wrapping am neuen Salt und verlangt dafür `RecoveryWechsel`: entweder den bisherigen Code (wird vorher gegen den alten Header geprüft, damit ein Tippfehler nicht stillschweigend zum neuen Code wird) oder einen neu erzeugten, der einmal angezeigt wird. `/auth/password` verlangt `recovery_auth_key` und `wrapped_account_key_recovery` als Pflichtfelder. Keine Änderung an der Komposition, deshalb bleibt `FORMAT_VERSION = 1` und bestehende Konten öffnen weiter. Tests: `wiederherstellung_ueberlebt_passwortwechsel`, `passwortwechsel_mit_neuem_code_entwertet_den_alten`, `passwortwechsel_lehnt_falschen_code_ab`, `keeps recovery working after a password change`. |
