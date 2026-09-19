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

## 0.11.0 — laufende Fassung

Diese Fassung beantwortet eine Rückmeldung, die nicht zu beschönigen war: *»Die Anwendung
fühlt sich immer noch nicht wie eine Arbeitserleichterung an.«* Sechs Punkte standen darin,
und fünf davon waren im Quelltext zu belegen. Der gemeinsame Nenner: Lotse **zeichnete
auf**, statt Arbeit abzunehmen. Es ließ dich beschreiben, einsortieren und pflegen – und
zeigte dir danach deine eigenen Eingaben.

### Das Logbuch füllt sich wieder von selbst

»Historien werden nicht geladen« war kein Eindruck, sondern **vier Fehler**, die sich
gegenseitig verdeckt haben:

| | |
|---|---|
| Die Beobachtung startete **nie** von selbst | Nach jedem Programmstart war sie aus – erkennbar nur an einer Knopfbeschriftung tief in den Einstellungen. |
| Sie ließ sich **ohne Suchordner gar nicht einschalten** | Wer seine Vorhaben per Hineinziehen angelegt hatte, kam nie an eine laufende Beobachtung. |
| Beobachtet wurden **nur Suchordner**, nie die Ordner der Vorhaben | Ein Projektordner außerhalb jeder Suchwurzel wurde auch bei laufender Beobachtung nicht mitgeschrieben. |
| Ein **nachträglich angehängtes Repo** las nie seine Historie | Der naheliegendste Weg – Vorhaben anlegen, Ordner später anhängen – führte zu einem leeren Logbuch. Gemeldet wurde nur »Referenz angelegt«. |

Jetzt: Die Beobachtung nimmt ihre Arbeit auf, sobald du entsperrst. Die Ordner deiner
Vorhaben werden immer beobachtet – **ohne Einstellung**, denn wer einen Ordner an ein
Vorhaben hängt, hat damit gesagt, dass er dazugehört. Ein Suchordner ist etwas anderes und
kommt nur dazu, wenn Lotse dort auch nach *neuen* Vorhaben sehen soll. Und ein Repo bringt
seine Vorgeschichte mit, egal auf welchem Weg es an ein Vorhaben kommt; Tage, für die schon
eine Notiz steht, werden dabei übersprungen. Auch Referenzen aus älteren Fassungen bekommen
ihre Historie beim nächsten Lauf nachgereicht. Siehe [[Beobachter und Erkennung|Beobachter-und-Erkennung]].

### Aus einem Ordner wird ein beschriebenes Vorhaben

Bisher ergab ein eingelesener Ordner: den Ordnernamen als Titel
(»heizungssteuerung-esp32«), eine aus Dateinamen geratene Vorlage – und als eigentliche
Ausbeute einen offenen Faden **»Kurs festlegen: worum geht es, was ist das Ziel?«**. Das
Programm reichte dir die Arbeit zurück.

Die KI konnte das Richtige längst und hing an der falschen Stelle: im Importweg wurde sie
nie gefragt. Jetzt steht im Befund *Von der KI deuten lassen*, und was herauskommt, sind
Titel in Worten, ein Satz zum Ziel, Themen und **die offenen Fäden, die schon im Ordner
stehen**. Den vollständigen Text siehst du vorher; übernommen wird nur, was auch etwas
sagt, und einzelne Fäden lassen sich wegnehmen.

### Die KI wird angeboten statt versteckt

Sie lag in einem Reiter der Einstellungen und zeigte standardmäßig auf ein lokales Ollama,
das die wenigsten laufen haben. Gefunden hat sie kaum jemand. Jetzt steht beim Start eine
Karte im Hafen, die es anbietet und durch die Einrichtung führt – Ollama oder ein Schlüssel
für einen gehosteten Dienst. **»Nicht mehr fragen« ist eine echte Antwort**: danach fragt
Lotse nicht wieder, und die Einrichtung steht weiter unter Einstellungen → KI. Siehe [[KI]].

### Eine Referenz ist ein Feld

»Der Unterschied der Materialien und anderen Dingen ist nicht klar« – kein Wunder: das
Formular verlangte **acht Typen und drei Rollen**, bevor überhaupt etwas dastand. 24
Kombinationen für das, was du als »da liegt das« denkst, und keine der beiden Achsen wurde
in der Oberfläche erklärt.

Jetzt schreibst du hin, wo es liegt, und Lotse sieht es sich an: eine Adresse ist eine
Adresse, ein Ordner mit `.git` darin ein Repo, »Keller, Regal 3, blaue Kiste« ein Ort. Die
Rolle ist keine Frage mehr und steht nur noch dort, wo sie von *Material* abweicht – an
jedem Eintrag »Material« war die Voreinstellung als Auskunft ausgegeben. Wer eine der drei
Arten braucht, die sich nicht ansehen lassen, klappt *Genauer* auf. Siehe [[Referenzen]].

### Ein Name ist ein Vorhaben

Wer »Dachboden ausbauen« tippte, bekam einen zweiten Bildschirm mit Titel, Vorlage und Kurs
– also die Frage nach dem, was er gerade geschrieben hatte. Jetzt wird angelegt: ein Klick,
ein Vorhaben.

Steckt mehr dahinter, heißt der Bildschirm **»Das habe ich gefunden«** statt »Befund«, die
Quelle steht oben, und Geratenes trägt die Marke *vorgeschlagen* – die verschwindet, sobald
du das Feld anfasst. Vorher sah Vorgeschlagenes aus wie selbst Getipptes.

### Die Oberfläche hört auf, sich zu erklären

**»Heute wichtig«** stand über den drei Vorhaben, bei denen am *längsten nichts passiert
ist* – zuletzt berührt vor 20, 40 und 45 Tagen. Die Überschrift versprach Dringlichkeit und
lieferte eine Schuldliste. Sie heißt jetzt **»Wartet auf dich«**.

Der Brief zählte Quellen statt Inhalte: *»Git: 1 Einträge, MCP: 1 Einträge«* – über Notizen,
die direkt darunter im Wortlaut standen, und die Mehrzahl auch bei einem einzigen Eintrag.
Jetzt steht dort, **was** passiert ist.

Und weg sind die Rechtfertigungen: *»Was das hier nicht ist«*, *»das ist kein Mangel«*,
*»Das ist in Ordnung – nur weiß beim nächsten Mal niemand …«*, *»Nichts davon ist
übernommen«*. Ein Werkzeug, das trägt, muss sich nicht verteidigen.

---

## 0.10.0

**Ein zweites Gerät braucht nur E-Mail und Passwort.** Bisher musste dafür die Adresse des
Sync-Dienstes abgetippt werden — im Anmeldefenster stand ein Platzhalter
(`https://api.example.invalid`), auf der Kommandozeile war `--url` Pflicht. Genau das hat
Anmelden wie Einrichtungsarbeit aussehen lassen. Die Adresse ist jetzt eingebaut: in der
Oberfläche vorbelegt und hinter »ändern« versteckt, auf der Kommandozeile optional.
Sichtbar bleibt sie, weil man wissen soll, wohin die Umschläge gehen; änderbar, weil
Selbsthosten möglich bleibt — es ist nur nicht mehr die Voraussetzung. Siehe
[[Einstellungen]].

**Das Konto lässt sich löschen.** `lotse sync konto-loeschen` oder Einstellungen →
Abgleich. Weg sind Konto, Umschläge, Anhänge, Geräte und Sitzungen; die E-Mail-Adresse ist
danach wieder frei, sonst wäre Löschen eine Sperre. Es verlangt dein Master-Passwort und
nicht nur die offene Sitzung — ein gestohlenes Token darf kein Konto ausradieren. Den
Wiederherstellungscode verlangt es dagegen **nicht**: wer nicht mehr hineinkommt, hat
trotzdem das Recht, seine Daten loszuwerden.

Was es ausdrücklich **nicht** anfasst: die Daten auf diesem Gerät. Das sind zwei
Entscheidungen mit zwei Knöpfen — »nicht mehr abgleichen« und »hier alles weg«
(`lotse zuruecksetzen`). Ein Knopf, der still beides täte, wäre eine Falle.

**Die Kommandozeile kennt den Befund.** `lotse deuten` nimmt dasselbe wie das Feld »Was
gibt's?« in der App: einen Ordner, eine Repo-Adresse, eine Datei oder einfach einen Namen.
Dann kommt der Befund mit Nummern und eine Frage: Enter übernimmt alles, `n` bricht ab,
Nummern lassen einzelne Funde weg — das Gegenstück zum Häkchen in der Oberfläche. Ohne
Terminal und ohne `--ja` entsteht nichts: in einer Pipeline soll nichts anfallen, was
niemand bestätigt hat. Siehe [[Kommandozeile]].

Dabei sind zwei Fehler herausgekommen, die beide auch die App betrafen. Ein Ordner, der
schon zu einem Vorhaben gehörte, ergab einen **leeren** Befund — das Anhängen hatte also
nie etwas anzuhängen. Jetzt wird weitergesucht, nur ohne Vorschlag: ein zweiter Aufruf
nach Wochen findet genau die Unterprojekte und Dokumente, die seitdem dazugekommen sind.
Und beim Anhängen wurde die Ordner-Referenz jedes Mal erneut angelegt; was mit gleichem
Typ und Ziel schon dranhängt, wird jetzt übersprungen.

**Geplant, noch nicht gebaut:** der Web-Client. Der Weg steht in `docs/WEB_CLIENT.md` —
fünf Phasen mit Abschlusskriterien, Adresse `app.lotse.sanctora.eu`. Die tragende
Entscheidung: der Browser braucht keine zweite Datenbank. Er hält versiegelte Umschläge und
baut den Index im Arbeitsspeicher. Ein Konto anlegen geht dort später auch — mit demselben
Abtippschritt für den Wiederherstellungscode wie auf der Kommandozeile. Siehe [[Roadmap]].

---

## 0.9.0

**Eine Repo-Adresse bringt jetzt etwas mit.** Bisher ergab sie Name und Link, mehr nicht —
Lotse hat GitHub nie gefragt. Das war kein Fehler, sondern eine nie gebaute Funktion. Jetzt
holt es den Steckbrief des Repos und füllt damit den Befund:

| Von GitHub oder GitLab | Wird hier zu |
|---|---|
| Beschreibung | Der **Kurs** — sie ist als Einzeiler geschrieben, eine README nicht |
| README, erste brauchbare Zeile | Der Kurs, falls keine Beschreibung da ist |
| Themen / Topics | Tags, im Befund abwählbar |
| Projektseite | Eine Referenz mit Rolle *Doku* |
| »archiviert« | Ein Hinweis: dort passiert nichts mehr |

Das gilt auch für einen **Ordner**, dessen Git-Remote dorthin zeigt — der häufigere Fall.
Was aus deinen eigenen Dateien gelesen wurde, bleibt dabei stehen: eine README auf der
Platte kennt das Vorhaben besser als ein Einzeiler auf GitHub.

Ein Token braucht es dafür **nicht**; öffentliche Repos antworten auch ohne, nur knapper
(GitHub: 60 Anfragen je Stunde und Adresse). Für private schon. Und schlägt die Abfrage
fehl, steht das im Befund und das Anlegen geht trotzdem: wer eine Adresse einfügt, will ein
Vorhaben und keinen Netzwerkfehler. Siehe [[Gegenseite]].

---

## 0.8.1

**Das Anlegen in 0.8.0 war kaputt.** Der Dialog sprang nach jedem Klick sofort zurück:
»Deuten« und »Ohne Quelle« sahen beide aus wie tot. Ursache war ein Effekt in der
Oberfläche, der eine Element-Referenz gelesen hat — die wird beim Wechsel zum Befund
ausgehängt, der Effekt lief dadurch noch einmal und machte seine eigene Rücksetzung
zunichte. Kein Test hat das gefangen, weil es für Dialoge keinen gab; inzwischen läuft
einer im Browser durch den ganzen Ablauf.

**Und es ist jetzt ein Feld statt zweier Schritte.** Das Fenster fragt **Was gibt's?** und
wartet auf eine Zeile. Die nimmt alles:

| Hineingeschrieben | Was daraus wird |
|---|---|
| `/home/ich/Gartenhaus` | Der Ordner, mit allem darin |
| `https://github.com/ich/lotse` | Das Repo als Gegenseite |
| `~/Downloads/angebot.pdf` | Die Datei als Referenz |
| `Gartenhaus` | Ein Vorhaben mit diesem Namen |

Ordner und Dateien lassen sich auch ins Fenster **ziehen** oder über *Durchsuchen* wählen.
Die Frage »ist das ein Ordner, eine Adresse oder ein Titel?« kann das Programm selbst
beantworten — sie zu stellen verlangte eine Einordnung, die gerade niemand vorhatte.

Zwei Dinge daran sind bewusst streng: ein Pfad mit Tippfehler ist ein **Fehler** und kein
Titel, sonst bekäme man still ein Vorhaben namens `/home/ich/Grten`. Und ohne Anker
(`/`, `~/`, `./`) bleibt »Haus/Garten« ein Titel, denn in ein Dialogfeld tippt niemand
relative Pfade, Titel mit Schrägstrich aber schon.

---

## 0.8.0

**Ein Vorhaben anlegen fragt jetzt nur noch eines: woher kommt es?** Der alte Dialog wollte
Titel, Vorlage und Kurs – also genau das, was Lotse aus einem Ordner selbst herauslesen
kann. Jetzt wählst du einen Ordner, fügst eine Adresse ein oder wirfst ein paar Dateien
hinein, und Lotse zeigt einen **Befund**: Titel, Kurs und Vorlage schon ausgefüllt, dazu was
es gefunden hat – die Gegenseite auf GitHub oder GitLab, eigene Vorhaben in Unterordnern,
lesbare Dokumente. Alles vorangekreuzt, alles abwählbar ([[Erste Schritte|Erste-Schritte]]).

Der Befund ist **nie eine Frage, immer eine Feststellung**. Wer mit einer GitHub-Adresse
anfängt, wird nicht mehr gefragt, ob er von GitHub importieren will – das stand ja schon in
der Adresse. Und liegt in dem Ordner bereits eine Kennung von Lotse, entsteht kein zweites
Vorhaben: dann kommt das Abgehakte zum vorhandenen dazu.

**Löschen gibt den Ordner wieder frei.** Das ist der Weg zurück, wenn ein Befund zu weit
ging. Bisher blieb die Markerdatei liegen, und der Ordner galt für immer als »gehört schon
dazu« – er ließ sich nie wieder anlegen.

**Wenn ein Zugang fehlt, sagt Lotse es dort, wo du hinsiehst.** Bisher stand »401« in einer
Fehlerliste in den Einstellungen; die öffnet niemand, solange nichts wehtut. Jetzt steht im
betroffenen Vorhaben ein offener Faden – mit der Folge statt der Einstellung: »keine offenen
Anfragen, kein Prüflauf-Status, private Repos gar nicht«. Genau einmal, nur wenn wirklich
kein Token hinterlegt ist, und nie bei Netzausfall ([[Gegenseite]]).

**Kalender: einmal hinterlegen, dann nachsehen lassen.** Dieselbe lange `.ics`-Adresse für
jedes Vorhaben abzutippen war Unfug. Unter **Einstellungen → Verbindungen → Kalender**
steht jetzt eine Liste der Kalender, die dir gehören, und auf einer Projektseite ohne
Kalender ein Knopf: *Nach „Gartenhaus" suchen*. Lotse zeigt die passenden Termine mit den
Suchbegriffen dabei; angehängt wird auf Klick. Die Liste ist ein **Vorrat, keine
Zuordnung** – gelesen wird ein Kalender weiterhin nur dort, wo er als Referenz am Vorhaben
hängt ([[Kalender]]).

**Die KI bekommt eine Aufgabe, die sich lohnt: den Kurs.** Titel und Vorlage liest die
Erkennung aus Marken – dafür wäre ein Modell Verschwendung. Aber die erste Zeile einer
README ist als Kurs meistens eine Überschrift, und den Satz, der dir in drei Monaten sagt,
was du eigentlich wolltest, kann keine Regel schreiben. Unter *Bearbeiten* steht neben dem
Kursfeld **Kurs von der KI vorschlagen lassen**; gesendet wird Name, Marken, README-Anfang
und Dateinamen, und du siehst den Text vorher ([[KI]]).

Dabei kam ein **Fehler** heraus: »Datei deuten« konnte nie funktionieren. Die Zweck-Erkennung
lief über eine handgeschriebene Liste mit einem Eintrag, und jede Anfrage scheiterte an
»Unbekannter Zweck«. Behoben, und ein Test geht jetzt alle Zwecke durch statt einen.

**Linux: das AppImage richtet sich ein, wenn du es lässt.** Ein AppImage wird nicht
installiert – es liegt da, wo der Browser es hingelegt hat. Daraus folgte: kein Eintrag im
Menü, ein Selbst-Update, das am Download-Ordner hängt, und ein Dateiname, der nach dem
ersten Tausch die alte Version nennt. Beim ersten Start fragt Lotse jetzt **einmal**, legt
sich nach `~/.local/share/lotse/Lotse.AppImage` und schreibt Desktop-Datei und Icon. Ein
»nein« wird gemerkt; der Knopf bleibt unter **Einstellungen → Version → Platz im System**,
und *Eintrag entfernen* nimmt es wieder weg ([[Installation]]).

Unter der Haube: die Suche eines Ordners hat jetzt ein Budget (20 000 Ordner oder drei
Sekunden) und sagt im Befund, wenn sie daran aufgehört hat – eine Liste, die man für
vollständig hält, ist schlimmer als eine, die ihre Grenze nennt. Maschinelle Logbuch-
Einträge wiederholen denselben Satz nicht mehr und kommen höchstens einmal am Tag; von Hand
angestoßen weiterhin sofort.

---

## 0.7.0

**Lotse lässt sich von einem Gerät wieder entfernen.** Bisher gab es dafür keinen Weg –
weder ein Kommando noch einen Knopf. Wer Lotse loswerden wollte, löschte den Datenordner
von Hand und ließ dabei jedes Mal den Desktop-Schlüssel im Schlüsselbund zurück, weil die
Geräte-ID in genau der Datei steht, die man gerade gelöscht hatte. Jetzt gibt es
`lotse zuruecksetzen` ([[Kommandozeile-Referenz]]) und **Einstellungen → Sicherheit →
Gerät zurücksetzen** ([[Einstellungen]]). Beide räumen Kontodatei, Datenbank **und**
Schlüsselbund ab, und beide brauchen kein Master-Passwort: Wer es vergessen hat, ist genau
der, der hier herauskommen will. Bestätigt wird getippt, nicht geklickt.

**Der Gerätename kommt jetzt vom System.** Beim Einrichten stand »Dieser Rechner« im Feld,
und wer nichts änderte, hatte in `lotse sync geraete` drei Zeilen, die alle gleich hießen.
Jetzt schlägt Lotse den Namen vor, unter dem der Rechner ohnehin bekannt ist – auf macOS
ohne das angehängte `.local`. Ändern kann man ihn weiterhin.

**Die Tastenkürzel zeigen das Richtige.** In der Kopfzeile stand `⌘P` und `⌘K`, auch auf
Windows und Linux, wo die Taste nicht existiert. Gehorcht hat Lotse immer beiden
Modifikatoren; jetzt steht dort auch `Strg+P` und `Strg+K`, wo es hingehört
([[Tastenkuerzel]]).

**Das Thema ist umschaltbar.** Standard bleibt, was das Betriebssystem sagt – neu ist, dass
man es unter **Einstellungen → Darstellung** überstimmen kann, in beide Richtungen. Die
Wahl gilt für dieses Gerät und wird nicht abgeglichen.

Unter der Haube: `rustls` auf 0.23.45 gehoben (RUSTSEC-2026-0285, TLS-1.3-Handshake-
Nachrichten wurden auf der falschen Verschlüsselungsebene angenommen).

---

## 0.6.2

**Die Landing Page läuft unter [lotse.sanctora.eu](https://lotse.sanctora.eu/), und der
Updater fragt dort zuerst nach.** Bis 0.6.1 stand dort die GitHub-Adresse; die bleibt als
zweiter Endpunkt drin, damit installierte ältere Fassungen weiter fündig werden – GitHub
leitet von dort auf die neue Domain um.

Dazu die **Lizenz: AGPL-3.0-only**. Vorher war das Repository öffentlich einsehbar *ohne*
`LICENSE`, was rechtlich »alle Rechte vorbehalten« bedeutet – für ein Werkzeug, dessen
Sicherheitsversprechen von der Prüfbarkeit lebt, der schlechteste aller Zustände. Die
Begründung steht im Entscheidungsprotokoll (`CONCEPT.md` 12): verkauft werden die Dienste,
nicht der Client, und §13 der AGPL verhindert genau das, was dem im Weg stünde – eine
geschlossene Fassung als Konkurrenzdienst.

Und ein Fehler im Release-Ablauf, gefunden bevor er zuschlug: die Prüfung, die nachfasst,
ob die Update-Liste wirklich ausgeliefert ist, stand im Release-Lauf selbst – und wartete
dort auf einen Deploy, den erst das **Ende** genau dieses Laufs auslöst. Jedes Release wäre
am Schluss rot gelaufen. Die Prüfung sitzt jetzt in `pages.yml`, direkt hinter dem Deploy,
wo sie prüfen kann, was sie gerade veröffentlicht hat.

**Für dich:** nichts zu tun. Wer 0.6.1 installiert hat, bekommt 0.6.2 über den eingebauten
Updater.

---

## 0.6.1

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

Vollständig, mit Phasen und Auslösern: **[[Roadmap]]**.

Die größte offene Lücke ist der Zugriff aus dem **Browser** – der Kern ist WASM-tauglich
gehalten, die Brücke fehlt.

Was **dauerhaft** nicht kommt und warum, steht in [[Nicht-Ziele]]. Die Liste ist kürzer,
wenn man sie einmal liest, als die Enttäuschung, wenn man es nicht tut.
