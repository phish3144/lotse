# Der Wo-war-ich-Brief

Die eine Sache, um die Lotse gebaut ist. Alles andere – Erkennung, Beobachter,
Abgleich, Tresor – ist Zulieferung für diesen Brief.

Wenn du ein Vorhaben nach längerer Pause öffnest, beantwortet er in fünf Zeilen: *Wo war
ich hier stehengeblieben, und was war der nächste Schritt?*

---

## Wann er erscheint

Beim Öffnen eines Vorhabens, wenn seit deinem letzten Kontakt mehr Zeit vergangen ist als
das Erwartungsintervall ([[Vorhaben]]). Bei einem Software-Vorhaben also nach mehr als
14 Tagen, bei einem Gartenhaus nach mehr als 60.

**Oder wenn etwas aus deiner Abwesenheit vorliegt**: eine Übergabe, offene Fäden, oder
Aktivität, seit du zuletzt selbst geschrieben hast. Dann hat der Brief etwas zu sagen, auch
wenn die Frist noch nicht um ist.

Bei einem Vorhaben, das du gestern angefasst hast und in dem seitdem nichts passiert ist,
erscheint er nicht. Ein Brief, der jedes Mal kommt, wird weggeklickt und ist dann nichts
wert.

Abrufen lässt er sich immer – in der App über die Projektseite, im Terminal über
`lotse projekt zeige <PROJEKT>`.

**Bis 0.11 war beides kaputt.** »Letzter Kontakt« zählte in der App ab der letzten Notiz
*welcher Quelle auch immer*. In einem Vorhaben mit laufendem [[Beobachter und
Erkennung|Beobachter-und-Erkennung]] rückte dieser Punkt jeden Tag nach: der Brief wurde
nie fällig. Und im Kern zählte die Aktivitätsliste nur, wenn es mindestens eine eigene
Notiz gab – in einem übernommenen Vorhaben (nur `import`, dann der Beobachter) blieb sie
dauerhaft leer. Genau die Vorhaben, die Lotse selbst anlegt, hatten damit keinen Brief.

---

## Was darin steht

Fünf Angaben, in dieser Reihenfolge. Die Reihenfolge ist die Antwortreihenfolge auf »wo
war ich«.

### 1 · Tage seit dem letzten Kontakt

Gezählt ab der letzten Notiz mit Quelle `mensch` oder `cli` – also ab dem letzten Mal,
als **du** etwas geschrieben hast. Nicht ab der letzten Aktivität überhaupt: sonst wäre
ein Vorhaben, in dem der Beobachter täglich mitschreibt, immer »null Tage her«, obwohl
du ein halbes Jahr nicht hingesehen hast.

Hast du in diesem Vorhaben noch nie selbst geschrieben – der Normalfall direkt nach dem
Übernehmen –, zählt es ab dem **Anlegen**. Nie weiter zurück als das: ein Vorhaben kann
nicht länger still sein, als es existiert.

Die Farbe im Hafen zählt anders, nämlich ab der letzten Notiz *jeder* Quelle
([[Vorhaben]]). Das ist Absicht: die Farbe fragt »ist hier etwas passiert?«, der Brief
fragt »war **ich** hier?«.

### 2 · Die letzte Übergabenotiz

Der Wortlaut dessen, was du beim Pausieren oder Warten notiert hast.

> Warte auf Angebot vom Statiker.

Das ist die wertvollste Zeile im Brief, und sie existiert nur, weil `pausiert` und
`wartet` eine Notiz verlangen. Genau deswegen verlangen sie sie.

### 3 · Die letzte Notiz überhaupt

Falls es keine Übergabe gibt – weil das Vorhaben einfach eingeschlafen ist statt bewusst
pausiert zu werden. Schlechter als eine Übergabe, besser als nichts. In der App steht sie
als *Letzter Eintrag*; bis 0.11 zeigte nur die Kommandozeile sie an.

### 4 · Die offenen Fäden

Alle Einträge der Art `offen`, die nicht abgehakt sind. Das ist die Antwort auf »was war
der nächste Schritt« ([[Logbuch]]).

### 5 · Aktivität seit deinem letzten Besuch, gezählt je Quelle

> seit deinem letzten Besuch: 40 Dateiänderungen, 12 Commits, 3 Einträge vom Assistenten

Nicht »55 Einträge«, sondern aufgeschlüsselt. Der Unterschied ist die ganze Information:
40 Dateiänderungen heißen, dass gearbeitet wurde; 12 Commits heißen, dass etwas fertig
wurde; drei MCP-Einträge heißen, dass ein Assistent daran saß.

Gezählt wird alles **nach** deiner letzten `mensch`- oder `cli`-Notiz, und nur, was
nicht von dir kam. Deine eigenen Einträge sind kein »das ist passiert, während ich weg
war«. Ohne eigene Notiz zählt es ab dem Anlegen – sonst hätte ein übernommenes Vorhaben
nie eine Zeile hier.

Dazu nennt der Brief die Auffälligkeit des Vorhabens, damit die Zahl »120 Tage« gleich
eingeordnet ist.

---

## Warum kein Sprachmodell nötig ist

Der Brief ist eine reine Funktion über die Notizen. Dieselbe Funktion in App,
Kommandozeile und Kern – kein Dienst, keine Anfrage, kein Netz, kein Schlüssel. Er
funktioniert im Flugzeug und in zehn Jahren.

Wer ihn kürzer will, kann ihn von einem Modell auf fünf Sätze verdichten lassen:
[[KI]]. Das ist eine **Ergänzung**. Der Brief selbst bleibt ohne, und das ist Absicht: die
Antwort auf »wo war ich« darf nicht davon abhängen, dass ein Anbieter erreichbar ist.

---

## Wie man ihn nützlich macht

Der Brief kann nur zeigen, was drinsteht. Drei Gewohnheiten, die ihn von »nett« zu
»unverzichtbar« machen:

**Am Ende einer Sitzung einen Satz schreiben.** Nicht was du getan hast – das steht
schon in den Dateien –, sondern **was als Nächstes dran ist**.

```
lotse log "Schalung fertig. Morgen: Beton bestellen, 3 m³ C25/30."
```

**Beim Weglegen den Grund notieren.** `pausiert` und `wartet` verlangen es ohnehin. Nimm
es nicht als Formalität: dieser Satz ist in drei Monaten die Differenz zwischen
»irgendwas war da noch« und »ah ja, der Statiker«.

**Offene Fragen als offenen Faden festhalten**, nicht als normalen Eintrag. Dann stehen
sie im Brief und in [[Offene Punkte|Offene-Punkte]], statt in der Zeitachse zu versinken.

Mehr dazu in [[Erste Schritte|Erste-Schritte]] unter *Der Rhythmus*.

---

## Für Assistenten

Ein KI-Assistent kann denselben Brief abrufen – das MCP-Werkzeug `get_project_context`
liefert ihn. Das ist der empfohlene erste Aufruf am Anfang einer Sitzung, damit der
Assistent nicht bei Null anfängt. Siehe
[[Assistenten (MCP)|Assistenten-MCP]].
