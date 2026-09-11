# Datenmodell

Jedes Feld, das Lotse speichert, und was es bedeutet. Wer nur wissen will, wie man mit
Lotse arbeitet, braucht diese Seite nicht – **[[Begriffe]]** reicht. Wer exportierte
Daten liest, ein eigenes Werkzeug anschließt oder verstehen will, was beim Abgleich
übertragen wird, ist hier richtig.

Die Feldnamen sind deutsch, weil sie sonst zweimal existieren müssten – einmal im Code,
einmal in der Oberfläche. Nur die Sync-Umschläge und die API sprechen englisch, weil
sie eine Schnittstelle sind ([[Sync-Protokoll]]).

**Zeitstempel** sind Unix-Millisekunden als vorzeichenbehaftete 64-Bit-Ganzzahl.
**Datumsangaben** sind `JJJJ-MM-TT`. **IDs** sind ULIDs: 26 Zeichen, lexikografisch
nach Entstehungszeit sortierbar, ohne zentrale Vergabestelle – wichtig, weil zwei
Geräte offline dieselbe Art von Datensatz anlegen dürfen, ohne zu kollidieren.

---

## Projekt

Ein Vorhaben. Die Klammer um alles andere.

| Feld | Typ | Bedeutung |
|---|---|---|
| `id` | ULID | Unveränderlich, auch über Umbenennen und Abgleich hinweg. |
| `titel` | Text | Was du siehst. Darf sich ändern. |
| `kurs` | Text | Der eine Satz: worum geht es, was ist das Ziel. Darf leer sein, sollte es aber nicht – siehe [[Vorhaben]]. |
| `status` | Status | Siehe unten. |
| `wiedervorlage` | Datum oder leer | Ab diesem Tag wird das Vorhaben überfällig, unabhängig vom Status. |
| `erwartungsintervall_tage` | Ganzzahl | Nach wie vielen Tagen ohne Kontakt das Vorhaben auffällt. Vorbelegt aus der Vorlage. |
| `tags` | Liste von Text | Freie Schlagworte. Aus der Vorlage vorbelegt. |
| `vorlage` | Vorlage | Die Art des Vorhabens. Bestimmt Intervall und Tags beim Anlegen. |
| `abgeleitet_von` | ULID oder leer | Aus welchem Vorhaben dieses hervorgegangen ist. Für Abspaltungen. |
| `angelegt` | Zeitstempel | Wann es entstand. |
| `zuletzt_beruehrt` | Zeitstempel | Letzter Kontakt. Ersatzwert für die Auffälligkeit, wenn es noch keine Notiz gibt. |

### Status

Sechs Zustände. Nur `aktiv` kann auffallen; alles andere ruht bewusst.

| Wert | Bedeutung | Auffällig? |
|---|---|---|
| `idee` | Noch nicht angefangen. | nein |
| `aktiv` | Läuft. Auf See. | ja |
| `pausiert` | Bewusst unterbrochen. Verlangt eine Übergabenotiz. | nein |
| `wartet` | Hängt an etwas Externem – Antwort, Lieferung, Genehmigung. Verlangt eine Übergabenotiz. | nein |
| `abgeschlossen` | Fertig. | nein |
| `eingemottet` | Aufgegeben oder für unbestimmte Zeit weggelegt. | nein |

Eine verstrichene `wiedervorlage` macht ein Vorhaben überfällig, **auch** wenn es
`pausiert` oder `wartet` ist. Das ist der einzige Weg, wie ein ruhendes Vorhaben von
sich aus wieder auf sich aufmerksam macht.

### Vorlage

Acht Arten von Vorhaben. Die Vorlage wirkt nur beim Anlegen: sie belegt
`erwartungsintervall_tage` und `tags` vor. Danach ist alles frei änderbar.

| Wert | In der Oberfläche | Intervall | Tags |
|---|---|---|---|
| `software` | Software | 14 Tage | `software` |
| `hardware_maker` | Hardware & Maker | 30 Tage | `hardware`, `maker` |
| `haus_garten` | Haus & Garten | 60 Tage | `haus` |
| `kreativ` | Kreativ | 30 Tage | `kreativ` |
| `finanzen_verwaltung` | Finanzen & Verwaltung | 90 Tage | `verwaltung` |
| `lernen_forschung` | Lernen & Forschung | 30 Tage | `lernen` |
| `reise_veranstaltung` | Reise & Veranstaltung | 120 Tage | – |
| `generisch` | Allgemein | 30 Tage | – |

Auf der Kommandozeile sind die Kurznamen `software`, `hardware`, `haus`, `kreativ`,
`finanzen`, `lernen`, `reise`, `generisch` erlaubt.

---

## Notiz

Ein Logbuch-Eintrag. Ausführlich in **[[Logbuch]]**.

| Feld | Typ | Bedeutung |
|---|---|---|
| `id` | ULID | Das Argument für `lotse erledigt`. |
| `projekt_id` | ULID | Zu welchem Vorhaben. |
| `ts` | Zeitstempel | Wann. Bestimmt die Reihenfolge im Logbuch. |
| `quelle` | Quelle | Wer geschrieben hat. |
| `art` | Art | Was für ein Eintrag. |
| `text` | Text | Der Inhalt. Wird nie automatisch gekürzt. |
| `erledigt_am` | Zeitstempel oder leer | Nur bei Art `offen`: wann abgehakt. Leer = noch offen. |

### Quelle

Acht Quellen. Sie machen sichtbar, was in deiner Abwesenheit passiert ist – die
Zeitachse in der App färbt danach, und der [[Wo-war-ich-Brief|Wo-war-ich-Brief]] zählt
je Quelle, was seit deinem letzten Besuch dazukam.

| Wert | Wer |
|---|---|
| `mensch` | Du, in der App. |
| `cli` | Du, über `lotse log`. |
| `datei` | Der Ordner-Beobachter: Dateien geändert. |
| `git` | Commits aus einem lokalen Repo. |
| `import` | Von außen eingelesen. |
| `mcp` | Ein KI-Assistent über [[Assistenten (MCP)|Assistenten-MCP]]. |
| `ki` | Eine Verdichtung, die du übernommen hast ([[KI]]). |
| `sync` | Vom Abgleich hereingekommen. |

`mensch` und `cli` gelten als *dein* Kontakt. Alles andere ist Aktivität, die passiert
ist, während du nicht hingesehen hast – genau die Unterscheidung, die den Brief nützlich
macht.

### Art

| Wert | Bedeutung |
|---|---|
| `log` | Der Normalfall: etwas ist passiert. |
| `offen` | Ein offener Faden. Bleibt in [[Offene Punkte|Offene-Punkte]] stehen, bis er abgehakt ist. |
| `entscheidung` | Eine Entscheidung mit Begründung. Wird in der App hervorgehoben, weil man sie später sucht. |
| `status` | Ein Statuswechsel. Schreibt Lotse selbst. |
| `uebergabe` | Die Notiz beim Pausieren oder Warten. Das Erste, was der Brief zeigt. |

---

## Referenz

Ein Zeiger nach draußen. Ausführlich in **[[Referenzen]]**.

| Feld | Typ | Bedeutung |
|---|---|---|
| `id` | ULID | |
| `projekt_id` | ULID | |
| `typ` | ReferenzTyp | Was für ein Ziel. |
| `ziel` | Text | Pfad, Adresse oder Beschreibung. |
| `rolle` | Rolle | Wofür es im Vorhaben steht. |
| `geraet_id` | ULID oder leer | Bei gerätegebundenen Typen: auf welchem Gerät der Pfad gilt. Ein Pfad vom Notebook soll auf dem Standrechner nicht als »nicht erreichbar« gelten. |
| `zuletzt_geprueft` | Zeitstempel oder leer | |
| `pruefstatus` | Pruefstatus | |

### ReferenzTyp

| Wert | Beispiel | Prüfbar |
|---|---|---|
| `ordner` | `/home/du/bau` | im Dateisystem |
| `git_repo` | `/home/du/code/app` oder `https://github.com/du/app` | Pfad im Dateisystem, Adresse über das Netz |
| `url` | `https://example.com/statik` | über das Netz |
| `datei` | `~/Dokumente/statik.pdf` | im Dateisystem |
| `physisch` | »Ordner im Regal links, zweites Fach« | nein |
| `geraet` | »Raspberry Pi im Keller« | nein |
| `passwortmanager` | »Proton Pass, Eintrag Fritzbox« | nein |
| `anhang` | Eine an Lotse übergebene Datei | im Dateisystem |

Ein Ziel, das wie eine Adresse aussieht (`http://`, `https://`, `ssh://`, `git://`,
`git@host:pfad`), wird **nie** im Dateisystem gesucht – auch dann nicht, wenn der Typ
`git_repo` gerätegebunden wäre. Das war bis 0.5.0 anders und führte dazu, dass ein
GitHub-Repo als »nicht erreichbar« angezeigt wurde.

### Rolle

| Wert | Bedeutung |
|---|---|
| `material` | Geht hinein: Quellen, Vorlagen, Zulieferungen. |
| `ergebnis` | Kommt heraus: das, was entsteht. |
| `doku` | Beschreibt: Anleitungen, Normen, Angebote. |

### Pruefstatus

| Wert | Bedeutung |
|---|---|
| `ok` | Zuletzt erreichbar. |
| `nicht_erreichbar` | Ordner weg, Adresse antwortet 404. |
| `nicht_pruefbar` | Noch nie geprüft, oder ein Typ, bei dem kein Programm nachsehen kann. |

Bei Adressen zählen `401` und `403` als **erreichbar**: etwas ist dort, du darfst es nur
nicht sehen. Nur `404` und `410` bedeuten »nicht da«.

---

## Tresor-Eintrag

Zugangsdaten. Der Eintrag steht hier nur der Vollständigkeit halber; die Werte sind
einzeln verschlüsselt und werden vom Modell nicht angefasst. Siehe **[[Tresor]]**.

### Stufe

| Wert | Lesbar wo | Schlüssel |
|---|---|---|
| `ueberall` | jedes Gerät mit Master-Passwort | Tresor-Schlüssel, abgeleitet aus dem Konto-Schlüssel |
| `nur_desktop` | nur Desktops mit Desktop-Schlüssel | abgeleitet aus Konto- **und** Desktop-Schlüssel |

Der Desktop-Schlüssel liegt im Schlüsselbund des Betriebssystems, ist nicht aus dem
Passwort ableitbar und war nie auf dem Server. Wer das Passwort erbeutet, kommt an
`nur_desktop`-Einträge trotzdem nicht.

---

## Geraet

Ein Gerät am Konto.

| Feld | Typ | Bedeutung |
|---|---|---|
| `id` | ULID | Steckt auch in jeder HLC – daran hängt die Konfliktauflösung. |
| `name` | Text | Was du bei `init` oder `sync login` angegeben hast. |
| `plattform` | Text | `linux`, `macos`, `windows`. |
| `angelegt` | Zeitstempel | |
| `zuletzt_sync` | Zeitstempel oder leer | Letzter erfolgreicher Abgleich. |

---

## Kandidat

Ein Ordner, den die Erkennung für ein Vorhaben hält. Wartet in der Hafeneinfahrt auf
deine Entscheidung. Regeln in **[[Erkennungsregeln]]**.

| Feld | Typ | Bedeutung |
|---|---|---|
| `pfad` | Text | Absoluter Pfad des Ordners. |
| `name` | Text | Vorgeschlagener Titel, aus dem Ordnernamen. |
| `vorlage` | Vorlage | Vorgeschlagene Art, aus den gefundenen Marken. |
| `marken` | Liste von Text | Was gefunden wurde, z. B. `cargo.toml`, `.git`. Damit ist der Vorschlag nachvollziehbar statt geraten. |
| `bekannte_id` | ULID oder leer | Gesetzt, wenn der Ordner eine `.lotse-projekt`-Marke hat und damit einem Vorhaben schon zugeordnet ist. |
| `hat_git` | ja/nein | Ob ein `.git` daneben liegt. |
| `readme` | Text oder leer | Die ersten Zeilen einer README, als Vorschlag für den Kurs. Das Einzige, wovon die Erkennung je den *Inhalt* liest. |

Kandidaten werden nicht abgeglichen. Was auf einem Gerät in der Hafeneinfahrt liegt,
ist auf einem anderen bedeutungslos – die Ordner gibt es dort nicht.

---

## Was fest ist

| Konstante | Wert | Bedeutung |
|---|---|---|
| `FORMAT_VERSION` | `1` | Fassung der Krypto-Komposition. Steckt in jedem Ciphertext als mitauthentifizierte Zusatzinformation. Eine Änderung der Schlüsselhierarchie erhöht sie. |
| `SCHEMA_VERSION` | `1` | Fassung des Datenschemas. Steht als `lotse_schema` im Kopf jeder exportierten `projekt.md`. |

Beide zu erhöhen ist eine bewusste Entscheidung mit Eintrag in `THREAT_MODEL.md`, nicht
etwas, das beim Aufräumen passiert.
