# Nicht-Ziele

Diese Liste ist die Bremse gegen Feature-Enthusiasmus. Sie steht verbindlich in
[`docs/NON_GOALS.md`](https://github.com/phish3144/lotse/blob/main/docs/NON_GOALS.md)
und wird **nicht im Moment einer guten Idee** geändert.

## Der Aufnahmetest

Ein Feature kommt nur ins Produkt, wenn **alle drei** Fragen mit Ja beantwortet sind:

1. Löst es eines der vier Kernprobleme? *(Wiedereinstieg, Faden halten, Rad nicht neu erfinden, Geheimnisse sicher aufbewahren)*
2. Verlangt es **keine** laufende Pflegearbeit vom Nutzer?
3. Gibt es einen konkreten, dokumentierten Mangel beim **dritten** echten Projekt?

Punkt 2 ist der schärfste. Ein Werkzeug, das Pflege verlangt, wird nicht gepflegt – und
ein ungepflegtes Logbuch ist schlimmer als keins, weil man ihm nicht mehr traut.

## Dauerhaft nicht

**Zusammenarbeit.** Kein Multi-User, kein Teilen, keine Rechte. Lotse ist ein
Einzelnutzer-Werkzeug. Sobald zwei Menschen dasselbe Logbuch führen, braucht es
Konfliktauflösung für Menschen statt für Daten – und das ist ein anderes Produkt.

**Ein freies Block- oder Datenbanksystem.** Kein Notion-Nachbau, kein
Schema-Editor zur Laufzeit. Die Struktur ist bewusst eng: Vorhaben, Logbuch, Faden,
Referenz, Zugang. Wer mehr Freiheit will, verliert das, was Lotse ausmacht.

**Backlink-Graph, Graph-Ansicht.** Sieht gut aus, hilft selten. Die Frage „wo war ich?“
beantwortet ein Verlauf, kein Netz.

**Sprints, Tickets, Backlogs, Burndown, Gantt, Zeiterfassung.** Ein offener Faden ist
eine Frage, keine Aufgabe. Wer Tickets braucht, braucht ein Ticketsystem – und Lotse
holt Issues von GitHub bewusst nur als *Zahl*, nie als Fäden.

**Browser-Autofill, TOTP, SSH-Agent.** Dafür gibt es Passwortmanager. Der Tresor ist für
das, was an einem Vorhaben hängt – nicht für die zweihundert Web-Logins des Alltags.

**KI als Gedächtnis statt als Werkzeug.** Kein Chat als Hauptoberfläche, keine
automatisch erzeugten Aufgaben. Die KI verdichtet, was da ist; sie erfindet nichts und
entscheidet nichts.

**Stille Telemetrie.** Keine Nutzungsdaten ohne ausdrückliches Opt-in – auch später
nicht, auch nicht „anonymisiert“.

## Was das für dich heißt

Wenn du eines dieser Dinge brauchst, ist Lotse das falsche Werkzeug – und es ist besser,
das vorher zu wissen als nach drei Monaten.

Wenn dir etwas fehlt, das *nicht* auf dieser Liste steht: Issue aufmachen und den
konkreten Fall beschreiben. Der Aufnahmetest fragt nach dem dritten echten Projekt, nicht
nach einer Idee – ein Bericht aus der Praxis wiegt deshalb schwer.
