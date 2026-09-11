# Export und Fluchtweg

**Wenn Lotse morgen weg ist, kommst du trotzdem an alles.** Das ist keine Nettigkeit,
sondern Voraussetzung dafür, dass man einem Programm seine Vorhaben anvertraut.

Zwei Wege, für zwei verschiedene Zwecke.

---

## Klartext-Spiegel

Ein Ordner voller Markdown – je Vorhaben eine Datei mit Kurs, Status, Referenzen und dem
vollständigen Logbuch.

```bash
lotse export spiegel ~/Backup/lotse-markdown
```

Oder Einstellungen → *Export* → *Klartext-Spiegel schreiben*.

Gedacht für `grep`, Obsidian, einen Texteditor – für alles, was Markdown lesen kann. Auch
gut als Beilage in einem Backup, damit man in fünf Jahren nicht erst Lotse bauen muss.

> **Ohne Tresor.** Der Spiegel enthält keine Zugänge. Das Modul, das ihn schreibt, hat
> keinen Zugriffsweg dorthin – es könnte sie nicht einmal, wenn es wollte.

## Verschlüsseltes Bundle

Der **gesamte** Bestand als eine Datei, mit [`age`](https://age-encryption.org)
verschlüsselt – Tresor eingeschlossen.

```bash
lotse export bundle ~/Backup/lotse.json.age
```

Oder in den Einstellungen, mit einer Passphrase (mindestens 8 Zeichen).

Wieder aufmachen geht **auch ohne Lotse**:

```bash
age -d ~/Backup/lotse.json.age > lotse.json
```

Heraus kommt JSON, das man lesen kann. `age` ist ein kleines, verbreitetes Werkzeug, das
es in jeder Paketverwaltung gibt – bewusst kein Eigenformat.

Die Bilanz nach dem Schreiben nennt: Vorhaben, Notizen, Zugänge – und wie viele
Tresor-Einträge auf **diesem** Gerät nicht lesbar waren. Das betrifft die Stufe *nur
Desktop*, wenn der Desktop-Schlüssel fehlt. Ein solches Bundle ist dann unvollständig,
und Lotse sagt es dir, statt es zu verschweigen.

---

## Was wann

| Zweck | Weg |
|---|---|
| Regelmäßiges Backup | **Bundle**, z. B. monatlich in einen Cloud-Ordner |
| Lesen ohne Lotse | **Spiegel** |
| Umzug auf einen neuen Rechner | **[[Abgleich]]**, oder Bundle plus `lotse init` |
| Vor dem Deinstallieren | beides |
| Vor einem Passwortwechsel | **Bundle** – sicher ist sicher |

## Ein brauchbarer Rhythmus

```bash
# monatlich, in einen Ordner, der selbst gesichert wird
lotse export bundle ~/Proton\ Drive/lotse-$(date +%Y-%m).json.age
```

Die Passphrase gehört in den Passwortmanager – **nicht** dieselbe wie das
Master-Passwort. Ein Backup, dessen Passphrase mit dem verlorenen Passwort identisch
ist, hilft im Ernstfall nicht.

## Was *nicht* geht

**Den Datenordner kopieren** ist kein Backup-Ersatz. Er enthält die verschlüsselte
Datenbank plus `konto.json`, und eine im Betrieb kopierte SQLite-Datei kann inkonsistent
sein. Für ein Backup nimm das Bundle.

**Den Datenordner in einen synchronisierten Cloud-Ordner legen** beschädigt ihn, sobald
zwei Rechner gleichzeitig schreiben. Dafür gibt es den **[[Abgleich]]**.

## Daten importieren

Es gibt keinen Import aus fremden Programmen. Der Weg hinein ist der Ordner-Scan: zeig
Lotse, wo deine Sachen liegen, und es schlägt vor, was es findet. Der Verlauf davor
entsteht nicht rückwirkend – bei Git-Repos holt Lotse aber die vorhandenen Commits als
Logbuch-Einträge mit.
