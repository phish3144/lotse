# Schlüssel und Krypto

Wie Lotse verschlüsselt, welcher Schlüssel woher kommt und was passiert, wenn einer
verloren geht. Wer nur wissen will, was geschützt ist, liest **[[Sicherheit]]**. Diese
Seite ist für den, der es nachrechnen will.

Die maßgebliche Fassung steht in `docs/THREAT_MODEL.md` im Repository.

---

## Grundsatz: keine eigenen Primitive

Lotse erfindet keine Kryptografie. Es setzt geprüfte Bausteine zusammen:

| Baustein | Wofür | Herkunft |
|---|---|---|
| **Argon2id** | Passwort → Schlüssel | RustCrypto |
| **HKDF-SHA256** | ein Schlüssel → viele, getrennt nach Verwendung | RustCrypto |
| **XChaCha20-Poly1305** | verschlüsseln und authentifizieren | RustCrypto |
| **SQLCipher** | die lokale Datenbank | Bibliothek |
| **age** | der Export-Fluchtweg | `age`-Crate |
| **minisign** | Signatur der Updates | über das Tauri-Updater-Plugin |
| **zeroize** | Schlüssel beim Verwerfen überschreiben | RustCrypto |

Jede Änderung an der Zusammensetzung erhöht `FORMAT_VERSION` und wird im Bedrohungsmodell
protokolliert. Das ist keine Formalität: eine stille Änderung würde alte Daten unlesbar
machen, ohne dass jemand sagen könnte, warum.

---

## Vom Passwort zu den Schlüsseln

```
Master-Passwort
    │ Argon2id (64 MiB, 3 Runden, 1 Spur, 16-Byte-Salt)
    ▼
┌───────────────┬────────────────┐
│ Auth-Schlüssel│ Wrap-Schlüssel │     HKDF-SHA256, getrennte info-Strings
└───────┬───────┴────────┬───────┘
        │                │ wrappt
   geht zum Server       ▼
   (dort noch einmal   Konto-Schlüssel  ← der eigentliche Schlüssel, 32 Byte, zufällig
    gehasht)             │
                         ├─ HKDF "lotse/local-db"       → SQLCipher-Schlüssel
                         ├─ HKDF "lotse/records"        → Umschläge für den Abgleich
                         ├─ HKDF "lotse/vault"          → Tresor, Stufe »überall«
                         ├─ HKDF "lotse/blobs"          → Anhänge
                         └─ + Desktop-Schlüssel
                            └─ HKDF "lotse/vault-desktop" → Tresor, Stufe »nur Desktop«
```

### Argon2id

64 MiB Speicher, 3 Runden, eine Spur. Das ist die Untergrenze für Produktionsdaten und
der Grund, warum das Entsperren einen Moment dauert – derselbe Moment kostet einen
Angreifer mit erbeuteter Datei pro Rateversuch dasselbe.

Die Parameter stehen in `konto.json` und werden von dort gelesen, nicht aus dem Programm.
So kann eine spätere Fassung sie erhöhen, ohne alte Konten auszuschließen.

`LOTSE_KDF_SCHNELL` setzt sie für Tests auf 8 MiB und eine Runde herunter. Niemals für
echte Daten ([[Umgebungsvariablen]]).

### Warum zwei Schlüssel aus dem Passwort

Der **Auth-Schlüssel** geht zum Server. Der **Wrap-Schlüssel** bleibt lokal und wrappt
den Konto-Schlüssel. Beide kommen aus derselben Argon2id-Ausgabe, aber über HKDF mit
verschiedenen `info`-Strings – und aus einem lässt sich der andere nicht berechnen.

Deshalb hilft dem Server der Auth-Schlüssel nicht beim Entschlüsseln: er ist der falsche
Ast des Baums. Er hasht ihn noch einmal mit PBKDF2-SHA256, 600 000 Runden, eigenem Salt,
und speichert nur den Hash – ein Datenbank-Dump taugt damit nicht als Login
([[Sync-Protokoll]]).

### Domain Separation

Jede Verwendung hat ihren eigenen `info`-String:

```
lotse/auth          lotse/wrap         lotse/recovery-auth
lotse/records       lotse/local-db     lotse/vault
lotse/vault-desktop lotse/blobs
```

Der Zweck: ein Schlüssel, der an einer Stelle austritt, taugt an keiner anderen. Ohne
Trennung wäre der SQLCipher-Schlüssel derselbe wie der für die Sync-Umschläge, und ein
gestohlenes Notebook hätte auch die Umschläge geöffnet.

### Der Konto-Schlüssel

32 Byte aus dem Zufallsgenerator des Systems. Er wird **nie** aus dem Passwort abgeleitet,
sondern nur damit gewrappt – deshalb ist ein Passwortwechsel billig: neu wrappen, fertig.
Nichts muss neu verschlüsselt und nichts neu hochgeladen werden.

Er liegt gewrappt in `konto.json` und – damit ein neues Gerät ihn holen kann – gewrappt
beim Dienst.

---

## Der Desktop-Schlüssel

32 Byte Zufall, **nicht** aus dem Passwort ableitbar, **nie** auf dem Server. Er liegt im
Schlüsselbund des Betriebssystems:

| System | Wo |
|---|---|
| Linux | GNOME Keyring / Secret Service |
| macOS | Keychain |
| Windows | Credential Manager |

Ausweichweg, wo kein Schlüsselbund erreichbar ist: `LOTSE_DESKTOP_KEY`.

Er wird nur für Tresor-Einträge der Stufe `nur_desktop` gebraucht. Deren Schlüssel wird
aus Konto- **und** Desktop-Schlüssel abgeleitet: wer das Master-Passwort erbeutet, kommt
an diese Einträge trotzdem nicht ([[Tresor]]).

---

## Der Wiederherstellungscode

Ein zweiter Weg zum Konto-Schlüssel, für den Fall, dass das Passwort weg ist. Der
Konto-Schlüssel liegt ein zweites Mal gewrappt beim Dienst – diesmal mit einem Schlüssel
aus dem Wiederherstellungscode.

Der Code wird **einmal** angezeigt, bei `lotse init` und nach jedem Passwortwechsel. Lotse
speichert ihn nirgends. Bei der Einrichtung von Hand verlangt es, dass du ihn abtippst –
das beweist, dass er angekommen ist.

Eine Wiederherstellung widerruft **alle** Sitzungen, auch die eigene. Wer den Code
benutzt, meldet damit jedes Gerät neu an.

---

## Was verloren ist, wenn was fehlt

| Fehlt | Folge |
|---|---|
| Passwort, Code vorhanden | Wiederherstellen, neues Passwort setzen. Alles bleibt. |
| Code, Passwort vorhanden | Nichts verloren. Neuen Code über einen Passwortwechsel erzeugen. |
| **beides** | **Alles verloren.** Endgültig. |
| Desktop-Schlüssel | Nur `nur_desktop`-Einträge sind unlesbar. Alles andere bleibt. |

Das »endgültig« ist kein Mangel, sondern die Definition von Ende-zu-Ende-Verschlüsselung.
Ein Hintertürchen für diesen Fall wäre eines für jeden Fall. Deshalb: beide in einen
Passwortmanager, heute.

---

## Wie ein Ciphertext aussieht

XChaCha20-Poly1305 mit 24-Byte-Nonce. Mitauthentifiziert (AAD) wird

```
FORMAT_VERSION · kind · id
```

Das bindet jeden Ciphertext an seinen Platz. Ein Umschlag, der als Notiz verschlüsselt
wurde, lässt sich nicht als Vorhaben ausgeben, und einer mit ID A nicht unter ID B
einsetzen – das Etikett schlägt fehl. Ohne AAD wäre genau das eine Angriffsfläche beim
Abgleich.

Schlüssel sind im Speicher der Typ `Key32` und werden beim Verwerfen überschrieben. Kein
Schlüssel wird als `Vec<u8>` oder `String` weitergegeben: die kopieren beim Umkopieren und
hinterlassen Reste, die niemand löscht.

---

## Crypto-Shredding

Einen Tresor-Eintrag zu löschen heißt: den **gewrappten Eintragsschlüssel entfernen**. Die
verschlüsselten Werte bleiben liegen, aber niemand kann sie mehr öffnen – auch nicht in
alten Sync-Kopien, auch nicht in Sicherungen von letztem Jahr.

Das ist der einzige Weg, in einem verteilten System wirklich zu löschen. Ein
»Löschen«-Befehl an alle Kopien erreicht die Sicherung auf der Platte im Schrank nicht.

---

## Signatur der Updates

Jede ausgelieferte Fassung ist mit einem minisign-Schlüssel signiert. Der öffentliche
Schlüssel steht **im Programm** (in `tauri.conf.json`), der private nur als
GitHub-Geheimnis – nie im Repository, nie in einem Protokoll.

Die App tauscht sich nur gegen etwas aus, dessen Signatur zu diesem Schlüssel passt. Erst
diese Prüfung macht einen Selbstaustausch vertretbar: ein Programm, das sich mit
ungeprüften Binärdaten überschreibt, wäre ein bequemer Weg für jeden, der die Verbindung
oder das Konto kontrolliert. Siehe [[Updates]].

Das ist **nicht** Code-Signierung: Windows SmartScreen und macOS Gatekeeper warnen beim
ersten Start weiterhin, weil dafür Zertifikate von Microsoft und Apple nötig wären. Die
minisign-Signatur schützt den Update-Weg, nicht die Erstinstallation.

---

## Trennwände im Programm

Krypto allein genügt nicht, wenn ein Modul, das nach außen redet, an die Klartexte kommt.
Deshalb gilt: `ai`, `detect`, `dokument`, `export::spiegel`, `forge`, `git`, `kalender`,
`mcp`, `netz`, `update` und `watcher` importieren **nie** aus `vault`.

`scripts/modulgrenzen.sh` prüft das bei jedem Bau und in CI, über Modulnamen statt
Dateinamen – auch für Module, die als Block in einer anderen Datei stehen. Das ist keine
Absichtserklärung, sondern ein Test, der rot wird ([[Für Entwickler|Fuer-Entwickler]]).

---

## Was nicht geschützt ist

Vollständig in [[Sicherheit]]. Die wichtigsten drei:

**Metadaten beim Abgleich.** Der Dienst sieht, dass es Datensätze gibt, welcher Art, wann
geändert, von welchem Gerät. Nicht deren Inhalt.

**Ein kompromittierter Rechner.** Wer mitliest, während Lotse entsperrt ist, liest mit.
Kein Programm kann das verhindern.

**Der Klartext-Spiegel.** `lotse export spiegel` schreibt Markdown ohne Verschlüsselung.
Absicht – das ist der Fluchtweg. Der Ordner muss selbst geschützt werden
([[Export und Fluchtweg|Export-und-Fluchtweg]]).
