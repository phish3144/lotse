# Änderungen

Was sich je Fassung geändert hat, und was das für dich bedeutet. Die
Veröffentlichungen mit den Installern liegen
[bei den Releases](https://github.com/phish3144/lotse/releases).

Alle Fassungen sind noch als **Vorabversion** markiert. Das heißt nicht »kaputt«, sondern
»das Datenformat kann sich noch ändern«. Bis 1.0 gilt: der Klartext-Spiegel ist die
Sicherung, der du trauen kannst ([[Export und Fluchtweg|Export-und-Fluchtweg]]).

Diese Seite wird mit jeder Fassung erweitert – wie und warum das erzwungen ist, steht in
[[Wiki pflegen|Wiki-pflegen]].

---

## 0.6.1 — laufende Fassung

**Nur der reparierte Release-Ablauf. Am Programm ist nichts geändert.**

0.6.0 konnte sich nicht selbst austauschen, obwohl genau das seine Neuerung war. Drei
Fehler, alle still:

- `args` und `prerelease` standen in der Workflow-Datei am Ende hinter einem `exit 1` –
  beim Umbauen aus dem richtigen Block gerutscht. Ohne `args` baute auf macOS **auch der
  Intel-Job Apple Silicon** und überschrieb dessen Artefakte. In der Update-Liste fehlte
  `darwin-x86_64` ganz; Intel-Macs hätten nie ein Update bekommen.
- `git diff --quiet` sieht eine noch nicht verfolgte Datei nicht und meldete
  »unverändert«. Die Spiegelung der Update-Liste auf die Landing Page blieb deshalb aus.
  Zusammen damit, dass GitHub Vorabversionen aus `/releases/latest` heraushält,
  antworteten **beide** Adressen mit 404.
- Windows zeigte auf die MSI statt auf den NSIS-Installer. Nur der tauscht eine laufende
  Installation ohne Rückfrage aus.

Damit das nicht wieder unbemerkt durchgeht, zählt der Release-Ablauf jetzt alle vier
Plattformen nach und bricht ab, wenn eine fehlt oder keine Signatur hat.

**Für dich:** weil die Update-Adresse außerhalb der App liegt, findet ein bereits
installiertes 0.6.0 nach dieser Veröffentlichung wieder ein Update. 0.6.1 selbst muss noch
von Hand geholt werden.

> **Der Weg zurück für 0.6.0.** Dass 0.6.1 ausgeliefert ist, genügt nicht – 0.6.0 muss die
> Liste auch *finden*. Nachgemessen, nicht angenommen:
>
> | Geprüft | Ergebnis |
> |---|---|
> | `latest.json` im Release | 11 Einträge, alle signiert, alle vier Plattformen |
> | dieselbe Datei auf `main` und unter der Pages-Adresse | byteweise identisch |
> | die vier Download-Adressen darin | alle antworten `200` |
> | Schlüsselkennung der vier Signaturen | `4BDDE1BB5195DE67` – dieselbe wie der in der App eingebaute öffentliche Schlüssel |
>
> Auf dem Weg dorthin kamen zwei weitere Fehler heraus. Der Pages-Ablauf war auf `main`
> noch nie durchgelaufen: alle vier Läufe dort scheiterten in `Set up job`, alle vier auf
> dem Feature-Branch waren erfolgreich – eine Branch-Regel am `github-pages`-Umfeld, die
> noch den früheren Standard-Branch nannte. Und der Spiegel-Commit wird mit dem
> `GITHUB_TOKEN` geschoben, was keinen Workflow auslöst; `pages.yml` hat dafür jetzt einen
> `workflow_run`-Auslöser. Seit beidem liefert die Seite auch die Landing Page im aktuellen
> Stand – vorher war es die Fassung vom 9. September.
>
> Damit so ein Ausfall nicht wieder grün durchgeht, fasst der Release-Ablauf die Adresse
> jetzt selbst nach und bricht ab, wenn sie nicht die gerade veröffentlichte Fassung nennt.

---

## 0.6.0

**Die erste Fassung, die sich selbst austauscht.** Sie lädt die neue Fassung, prüft ihre
Signatur gegen den eingebauten öffentlichen minisign-Schlüssel und startet neu – auf
Windows, macOS und aus dem AppImage heraus. Aus einem `.deb` oder `.rpm` nicht; dort
aktualisiert die Paketverwaltung, und Lotse zeigt das statt eines Knopfes, der nichts täte.

Vorher öffnete der Updater einen Browser und lud den Installer herunter – den Rest musste
man selbst machen. Details: [[Updates]], Signaturen in
[[Schlüssel und Krypto|Schluessel-und-Krypto]].

Beim Austausch wird gesperrt: der Schlüssel überlebt einen Neustart nicht, und das ist
richtig so.

> Diese Fassung findet aus eigener Kraft **kein** Update – siehe 0.6.1.

---

## 0.5.0

**Die Oberfläche, neu gedacht.** Farben aus dem Logo: Marine, Stahlblau, Pergament, und
Bernstein als Leuchtfeuer für genau **eine** Handlung je Ansicht. Vier gleich helle
Knöpfe auf einer Seite sind vier Knöpfe, die keiner drückt.

- Die Projektseite ist eine zweispaltige Arbeitsfläche: links das Logbuch als Zeitachse
  mit Tagesmarken und nach Quelle gefärbten Punkten, rechts Kurs, offene Fäden,
  Referenzen und Termine.
- Hafen, [[Offene Punkte|Offene-Punkte]], Suche, [[Tresor]], der leere Erststart, die Tür
  und die Überlagerungen nachgezogen.
- Die [[Einstellungen]] haben sieben Reiter statt einer Bahn von 1234 Zeilen.

Dazu zwei Reparaturen:

- Eine Referenz auf `https://github.com/…` wurde im **Dateisystem** gesucht und kam als
  »nicht erreichbar« zurück. Bestehende Referenzen heilen sich beim nächsten Speichern
  selbst ([[Referenzen]]).
- »Mit GitHub verbinden« kostete sechs Schritte. Jetzt ein Feld ([[Gegenseite]]).

---

## 0.4.0

**Der MCP-Zugang aus der laufenden App**, auf `127.0.0.1`: Bearer-Token,
Origin-Prüfung, kein CORS, und er endet mit dem Sperren. Vorher ging MCP nur über
stdin/stdout, und der Assistent brauchte dafür das Master-Passwort.

Dazu die Aushandlung der Protokollfassung, damit ältere Clients nicht abbrechen. Siehe
[[Assistenten (MCP)|Assistenten-MCP]].

Ein Nebenbefund, der schwerer wog als das Feature: `scripts/modulgrenzen.sh` übersprang
fehlende Dateien **still** und hatte `export::spiegel` deshalb nie geprüft – weil das
Modul als Block in einer anderen Datei steht. Die Prüfung geht jetzt über Modulnamen und
schlägt laut fehl, wenn sie etwas nicht findet ([[Für Entwickler|Fuer-Entwickler]]).

---

## 0.3.0

- **Gegenseite:** GitHub und GitLab, am Git-Remote erkannt, auf Wunsch vom Beobachter
  mitgenommen ([[Gegenseite]]).
- **Kalender**, lesend, mit `RRULE` soweit sicher berechenbar ([[Kalender]]).
- **KI-Verdichtung** mit Zweck, Verbrauchszählung und Protokoll ([[KI]]).
- **Update-Hinweis** – noch ohne Selbstaustausch.
- Dreizehn Befunde aus der Prüfung behoben.

---

## 0.2.0

Erste Fassung mit bedienbarer Oberfläche: anlegen, erfassen, Fäden abhaken, Status mit
Übergabe wechseln, [[Referenzen]], [[Tresor]] und Ordner-Scan.

---

## Was noch kommt

Kein Versprechen, eine Richtung.

| Vorhaben | Stand |
|---|---|
| Symbol im Infobereich, globales Erfassen-Kürzel | geplant ([[Tastenkürzel\|Tastenkuerzel]]) |
| Browser-Fassung (Kern als WebAssembly) | der Kern baut schon ohne Dateisystem-Anteile |
| Konto im Dienst löschen | noch nicht gebaut |
| Selbst gehostete Git-Instanzen (GitHub Enterprise, eigenes GitLab) | noch nicht erkannt |
| Lizenz | noch nicht festgelegt (`LICENSE-PENDING`) |

Was **dauerhaft** nicht kommt und warum, steht in [[Nicht-Ziele]]. Die Liste ist kürzer,
wenn man sie einmal liest, als die Enttäuschung, wenn man es nicht tut.
