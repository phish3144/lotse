# Referenzen

Zeiger nach draußen. Eine Referenz sagt: *das hier gehört zu diesem Vorhaben.* Der
Ordner mit den Plänen, das Repo, die Norm als PDF, der Kalender, der Aktenordner im
Regal, der Raspberry Pi im Keller.

Lotse **kopiert nichts** dorthin und holt nichts von dort. Referenzen sind Adressen, keine
Kopien – deshalb gibt es in Lotse nie eine veraltete Zweitfassung deiner Pläne.

Alle Felder: [[Datenmodell]].

---

## Wozu

Drei Dinge, die ohne Referenzen nicht gehen:

**Wiederfinden.** In sechs Monaten weißt du, dass es Pläne gab, aber nicht wo. Die
Projektseite weiß es.

**Zuordnen.** Steht eine Referenz auf einen Ordner, funktioniert `lotse log` aus diesem
Ordner ohne Projektangabe:

```
cd ~/bau && lotse log "Beton bestellt"
→ Gartenhaus (log)
```

**Mitschreiben.** Der Ordner-Beobachter und der Git-Leser arbeiten auf Referenzen. Ohne
sie gibt es keine automatischen Einträge.

---

## Typen

| Typ | Beispiel | Wird geprüft |
|---|---|---|
| `ordner` | `/home/du/bau` | im Dateisystem |
| `git_repo` | `/home/du/code/app` oder `https://github.com/du/app` | Pfad im Dateisystem, Adresse über das Netz |
| `url` | `https://example.com/statik` | über das Netz |
| `datei` | `~/Dokumente/statik.pdf` | im Dateisystem |
| `physisch` | »Aktenordner im Regal links, zweites Fach« | nein |
| `geraet` | »Raspberry Pi im Keller, 192.168.1.40« | nein |
| `passwortmanager` | »Proton Pass, Eintrag Fritzbox« | nein |
| `anhang` | eine an Lotse übergebene Datei | im Dateisystem |

### Warum `physisch` und `geraet` existieren

Weil die Hälfte aller Vorhaben nicht digital ist. Ein Gartenhaus hat einen Ordner im
Regal, eine Vereinskasse hat einen Karton mit Belegen, ein Maker-Projekt hat eine Platine
in einer Schublade. Wer das nicht notieren kann, notiert es nicht – und sucht dann.

Diese Typen sind immer `nicht_pruefbar`, und das ist kein Mangel: kein Programm kann in
dein Regal sehen.

### Warum `passwortmanager` und nicht der Tresor

Weil du deinen Passwortmanager vielleicht behalten willst. Eine Referenz vom Typ
`passwortmanager` sagt nur *dort liegt es*, ohne dass Lotse das Geheimnis kennt. Wer die
Zugangsdaten in Lotse haben will, nimmt den [[Tresor]] – beides ist legitim, und beides
nebeneinander auch.

---

## Rollen

Wofür die Referenz im Vorhaben steht. Drei Werte, zum Sortieren, nicht zum Steuern.

| Rolle | Bedeutung | Beispiel |
|---|---|---|
| `material` | geht hinein | Rohdaten, Vorlagen, Zulieferungen, der Arbeitsordner |
| `ergebnis` | kommt heraus | das fertige PDF, das Repo, die abgegebene Erklärung |
| `doku` | beschreibt | Anleitungen, Normen, Angebote, Genehmigungen |

Auf der Projektseite sind die Referenzen danach gruppiert. Das beantwortet »wo ist das
Ergebnis« ohne Suchen.

---

## Anlegen

```
lotse ref add Gartenhaus ordner /home/du/bau --rolle material
lotse ref add Gartenhaus url https://example.com/statik --rolle doku
lotse ref add Gartenhaus physisch "Aktenordner im Regal links, zweites Fach"
```

In der App: auf der Projektseite rechts, *Referenz hinzufügen*. Für Ordner und Dateien
gibt es einen Auswahldialog – bequemer und weniger fehleranfällig als Tippen.

Beim **Übernehmen** eines Kandidaten legt Lotse den Ordner automatisch als Referenz vom
Typ `ordner` mit Rolle `material` an ([[Erkennungsregeln]]).

---

## Prüfen

```
lotse ref liste Gartenhaus
01M28KB1CS4X6NSVFFXDK063EN  ordner          nicht_pruefbar  /home/du/bau
01M28KB1GMB5R9FV7F1TZ7THG8  url             nicht_pruefbar  https://example.com/statik

lotse ref pruefen 01M28KB1CS4X6NSVFFXDK063EN
```

| Status | Bedeutung |
|---|---|
| `ok` | Zuletzt erreichbar. |
| `nicht_erreichbar` | Ordner weg, Datei gelöscht, Adresse antwortet 404. |
| `nicht_pruefbar` | Noch nie geprüft, oder ein Typ, bei dem niemand nachsehen kann. |

### Was »erreichbar« bei Adressen heißt

`401` und `403` gelten als **erreichbar**: dort ist etwas, du darfst es nur nicht ohne
Anmeldung sehen. Nur `404` und `410` bedeuten »nicht da«.

Das ist wichtig für private Repos: `https://github.com/du/privat` antwortet ohne Token
mit 404 – GitHub verrät nicht, dass es existiert. Eine Referenz darauf kann also
`nicht_erreichbar` sein, obwohl alles in Ordnung ist. Deshalb löscht Lotse nie etwas
wegen eines Prüfergebnisses; es ist ein Hinweis, keine Diagnose.

### Gerätegebundene Referenzen

`/home/du/bau` gibt es auf dem Notebook, aber nicht auf dem Standrechner. Lotse merkt
sich deshalb bei Pfaden, **auf welchem Gerät** sie gelten (`geraet_id`), und prüft sie
nur dort. Auf anderen Geräten bleiben sie `nicht_pruefbar`, statt fälschlich rot zu sein.

Ein Ziel, das wie eine Adresse aussieht – `http://`, `https://`, `ssh://`, `git://` oder
`git@host:pfad` – wird **niemals** im Dateisystem gesucht und ist nie gerätegebunden,
auch wenn der Typ `git_repo` es normalerweise wäre.

> Bis 0.5.0 war das anders: eine Referenz auf `https://github.com/…` wurde im
> Dateisystem gesucht und kam als »nicht erreichbar« zurück. Bestehende Referenzen
> heilen sich beim nächsten Speichern selbst.

---

## Anhänge

Typ `anhang` ist eine Datei, die Lotse selbst verwahrt – verschlüsselt, und beim
Abgleich als eigener Blob übertragen ([[Sync-Protokoll]]). Der Dienst kennt davon nur
Konto, ID und Größe.

Anhänge sind für das gedacht, was sonst verloren geht: das eingescannte Angebot, das
Foto vom Typenschild. Für alles, was ohnehin schon ordentlich irgendwo liegt, ist eine
`datei`- oder `ordner`-Referenz besser – dann bleibt es dort, wo du es auch ohne Lotse
findest.

Grenze je Anhang: 100 MiB.

---

## Löschen

Eine Referenz zu löschen löscht **nur den Zeiger**. Der Ordner, die Datei, das Repo
bleiben unangetastet. Das gilt auch beim Löschen eines ganzen Vorhabens.

Die einzige Ausnahme ist `anhang`: dort verwahrt Lotse die Daten selbst, und mit der
Referenz sind sie weg.
