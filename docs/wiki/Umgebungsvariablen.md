# Umgebungsvariablen

Alle Variablen, auf die Lotse hört. Für den Alltag braucht man keine – sie sind für
Skripte, Server ohne Schlüsselbund, und für Fälle, in denen ein interaktives Abfragen
nicht geht.

> **Ein Hinweis, der jede dieser Variablen betrifft:** was in einer Umgebungsvariablen
> steht, sehen andere Prozesse desselben Nutzers (`/proc/<pid>/environ`), landet in
> Shell-Historien und in CI-Protokollen, wenn man nicht aufpasst. Für Geheimnisse ist
> das der schlechtere Weg. Nimm sie, wo es nicht anders geht – nicht aus Bequemlichkeit.

---

## Ort der Daten

### `LOTSE_HOME`

Datenordner. Gleichwertig mit `--home`; die Option gewinnt, wenn beide gesetzt sind.
Ohne beides das Datenverzeichnis des Systems ([[Datenablage]]).

```bash
LOTSE_HOME=/media/stick/lotse lotse hafen
```

Nützlich für einen zweiten, getrennten Bestand – etwa Arbeit und Privates in
verschiedenen Ordnern.

---

## Entsperren

### `LOTSE_PASSWORD`

Master-Passwort. Ohne diese Variable fragt die Kommandozeile interaktiv.

Für Skripte, die regelmäßig `lotse gegenseite` oder `lotse sync jetzt` laufen lassen.
Besser als ein Passwort im Skript: aus einem Passwortmanager holen, der auf dem Rechner
ohnehin entsperrt ist.

```bash
LOTSE_PASSWORD="$(pass lotse/master)" lotse sync jetzt
```

### `LOTSE_DESKTOP_KEY`

Der Desktop-Schlüssel, wenn kein Schlüsselbund erreichbar ist – in Containern, auf
Servern, in Desktop-Sitzungen ohne Secret-Service. Ohne ihn sind
`nur_desktop`-Tresor-Einträge nicht lesbar; alles andere geht.

Das Format ist das, was `lotse init` einmalig angezeigt hat (Gruppen, mit Bindestrichen).

### `LOTSE_RECOVERY_CODE`

Der Wiederherstellungscode. Nur `lotse sync register` fragt danach, weil daraus der
Wiederherstellungsweg gewrappt wird. Für nicht-interaktive Registrierung.

### `LOTSE_SKIP_CONFIRM`

Auf einen beliebigen Wert gesetzt: `lotse init` verlangt nicht, dass du den
Wiederherstellungscode zur Kontrolle abtippst. Gleichbedeutend mit
`--ohne-bestaetigung`.

Für Skripte und Tests. Bei einer Einrichtung von Hand ist das Abtippen der Punkt: es
beweist, dass der Code angekommen ist. Ohne Code und ohne Passwort sind die Daten
endgültig verloren – das ist kein Fehler, das ist Ende-zu-Ende-Verschlüsselung.

---

## Abgleich

### `LOTSE_SYNC_URL`

Adresse des Sync-Dienstes als Standardwert, damit `--url` nicht bei jedem Aufruf
mitmuss. Siehe [[Abgleich]].

---

## Gegenseite

Der Token für GitHub oder GitLab. Ist keine dieser Variablen gesetzt, nimmt Lotse den
Tresor-Eintrag, den die App hinterlegt hat ([[Gegenseite]]).

| Variable | Gilt für |
|---|---|
| `LOTSE_GITHUB_TOKEN` | nur GitHub |
| `LOTSE_GITLAB_TOKEN` | nur GitLab |
| `LOTSE_FORGE_TOKEN` | beide – Ausweichweg, wenn nur ein Hoster im Spiel ist |

Die hosterspezifischen Variablen gewinnen. Warum getrennt: ein Token für GitHub wird
dann nie an GitLab geschickt, auch nicht versehentlich.

Nötige Rechte: bei GitHub ein Feingranular-Token mit *Contents: Read* und *Pull
requests: Read* für die betroffenen Repos; bei GitLab `read_api`. Lesend genügt – Lotse
schreibt dort nichts.

---

## Export

### `LOTSE_EXPORT_PASSPHRASE`

Passphrase für `lotse export bundle`. Ohne sie wird interaktiv gefragt. Für eine
monatliche Sicherung aus einem Cron-Job:

```bash
LOTSE_EXPORT_PASSPHRASE="$(pass lotse/export)" \
  lotse export bundle ~/sicherung/lotse-$(date +%F).age
```

Siehe [[Export und Fluchtweg|Export-und-Fluchtweg]].

---

## Nicht für den Gebrauch

### `LOTSE_KDF_SCHNELL`

Setzt die Schlüsselableitung auf Testparameter herunter (8 MiB statt 64 MiB Speicher,
eine Runde statt drei). Damit läuft die Testsuite in Sekunden statt Minuten.

**Niemals für echte Daten.** Ein Konto, das damit eingerichtet wurde, ist gegen einen
Angriff mit erbeuteter Datei wesentlich schlechter geschützt. Die Variable existiert
nur, weil eine Testsuite, die zehn Minuten braucht, nicht gelaufen wird.

---

## Was Lotse **nicht** liest

Keine Konfigurationsdatei, keine `.lotserc`, keine Variablen außer den hier genannten.
Alles Einstellbare steht in der Datenbank unter `meta` ([[Datenablage]]) und ist über
die [[Einstellungen]] erreichbar. Ein zweiter Ort für Einstellungen wäre ein zweiter Ort
zum Nachsehen, wenn etwas unerwartet ist.

`HTTPS_PROXY` und Verwandte wertet die HTTP-Bibliothek aus, nicht Lotse selbst. TLS
prüft gegen den Wurzelspeicher des Systems – dieselbe Wahl, die `git` und `curl` auf
demselben Rechner treffen. In Netzen mit TLS-Prüfung funktioniert Lotse damit, während
eine eigene mitgelieferte Zertifikatsliste dort ausfiele.
