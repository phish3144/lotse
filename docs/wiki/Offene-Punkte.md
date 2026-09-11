# Offene Punkte

Alle offenen Fäden aus allen Vorhaben auf einer Seite. Die Antwort auf »was liegt
eigentlich überall herum«.

Erreichbar über *Offene Punkte* in der Kopfzeile, im Terminal über `lotse offen`.

---

## Wie sortiert wird

Gruppiert nach Vorhaben, die Vorhaben nach Auffälligkeit: **überfällig** zuerst, dann
**auffällig**, dann **ruhig**; bei gleicher Stufe alphabetisch nach deutschem
Sortierschema (Umlaute wie Grundbuchstaben).

Das ist die Reihenfolge, in der man eine Liste durchgehen will: nicht »neueste zuerst«,
sondern »was fällt hinten runter«. Ein Faden in einem überfälligen Vorhaben ist dringender
als einer in einem, das gestern angefasst wurde – auch wenn er älter ist.

Innerhalb eines Vorhabens: die ältesten Fäden oben.

---

## Die drei Abschnitte

### Nach Vorhaben

Der Hauptteil. Je Vorhaben eine Gruppe mit Titel, Auffälligkeit und den offenen Fäden.
Jeder Faden hat ein Häkchen zum Erledigen – ohne Rückfrage, weil es rückgängig ist
(der Eintrag bleibt im Logbuch).

### Am längsten offen

Die drei ältesten Fäden über alle Vorhaben hinweg.

Das ist der nützlichste Abschnitt und der unbequemste. Was ein Jahr offen ist, ist
meistens eines von zwei Dingen: wichtig und verdrängt, oder erledigt und nie abgehakt.
Beides lohnt fünf Minuten.

### Ohne Faden

Aktive Vorhaben, die **keinen** offenen Faden haben.

Das ist kein Lob. Ein aktives Vorhaben ohne offenen Faden heißt oft: niemand weiß mehr,
was der nächste Schritt ist. Der Abschnitt ist die Einladung, einen Faden nachzutragen –
oder das Vorhaben ehrlich auf `pausiert` zu setzen ([[Vorhaben]]).

---

## Was das hier **nicht** ist

Die Seite sagt es selbst, und es steht dort mit Absicht.

**Keine Aufgabenverwaltung.** Keine Prioritäten, keine Fälligkeiten je Faden, keine
Unteraufgaben, keine Zuweisung, kein Wiederkehren. Wer das braucht, soll ein Werkzeug
dafür benutzen – Lotse baut es dauerhaft nicht ([[Nicht-Ziele]]).

**Kein Backlog.** Offene Fäden sind das, was *dieses Vorhaben* offen lässt, nicht alles,
was man sich vorstellen könnte. Eine Liste mit 200 Einträgen wird nicht gelesen, und dann
ist auch der eine wichtige Eintrag verloren.

**Keine Issues.** Die Zahl offener Issues aus GitHub oder GitLab kommt als verdichtete
Zeile ins Logbuch, nicht als Fäden ([[Gegenseite]]). Sonst hätte man dieselben Tickets
zweimal, an zwei Orten, mit zwei Ständen.

---

## Einen Faden anlegen

```
lotse log --offen "Statiker fragen wegen Dachlast" -p Gartenhaus
```

In der App: Schnellerfassung mit Strg/Cmd + K, dort die Art auf *offen* stellen.

Ein guter Faden nennt eine **Handlung** oder eine **Frage**, nicht ein Thema. »Dachlast«
ist kein Faden. »Statiker fragen wegen Dachlast« ist einer: man kann feststellen, ob er
erledigt ist.

## Abhaken

```
lotse offen
01M28KAQB1308AJGF0326ZDMZT  [Gartenhaus] Statiker fragen wegen Dachlast
lotse erledigt 01M28KAQB1308AJGF0326ZDMZT
```

Der Eintrag bleibt im Logbuch und bekommt `erledigt_am`. Dass etwas erledigt wurde und
wann, ist selbst eine Information – und in drei Monaten manchmal die wichtigere.

Mehr zum Logbuch und seinen Arten: [[Logbuch]].
