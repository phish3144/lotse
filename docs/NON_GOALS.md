# Lotse – Nicht-Ziele

Diese Liste ist die Bremse gegen Feature-Enthusiasmus. Sie wird **nur beim wöchentlichen
Soll-Ist-Check** geändert, nie im Moment einer guten Idee. Jede Streichung braucht einen
konkreten Mangel bei einem echten Projekt, keine Spekulation.

## Aufnahmetest für jedes neue Feature

Ein Feature kommt nur ins Produkt, wenn alle drei Fragen mit Ja beantwortet sind:

1. Löst es eines der vier Kernprobleme (Wiedereinstieg, Faden halten, Rad nicht neu
   erfinden, Geheimnisse sicher aufbewahren)?
2. Verlangt es **keine** laufende Pflegearbeit vom Nutzer?
3. Gibt es einen konkreten, dokumentierten Mangel beim **dritten** echten Projekt?

## Dauerhaft nicht

- **Multi-User, Teilen, Rechte, Kollaboration.** Lotse ist ein Einzelnutzer-Werkzeug.
  Mandantenfähigkeit des Sync-Dienstes ist Infrastruktur, kein Feature.
- **Freies Block- oder Datenbank-System.** Kein Notion-Nachbau, kein Laufzeit-Schema-Editor.
- **Backlink-Graph über Notizen, Graph-Visualisierung.**
- **Sprints, Tickets, Backlogs, Burndown, Gantt, Zeiterfassung.**
- **Browser-Autofill, TOTP-Verwaltung, SSH-Agent.** Dafür gibt es Proton Pass.
- **KI als Gedächtnis statt Log; Chat als Hauptoberfläche; automatisch erzeugte Aufgaben.**
- **Stille Telemetrie.** Keine Nutzungsdaten ohne ausdrückliches Opt-in, auch später nicht.
- **Fremdskripte, CDNs, Analytics in der Web-App.**

## Nicht in Version 1 (mit Auslöser für später)

| Nicht jetzt | Auslöser |
|---|---|
| Bausteine-Bibliothek | derselbe Inhalt dreimal von Hand kopiert |
| Facetten (Stückliste, Kosten) | dreimal eine Tabelle in Freitext gequetscht |
| Kanban-Ansicht | ein Projekt mit dauerhaft > 15 offenen Fäden |
| Projekt-zu-Projekt-Kanten | Unterprojekte treten real mehrfach auf |
| Feldweises Mergen beim Sync | zweiter realer Datenverlust durch Last-Writer-Wins |
| Semantische Suche, Embeddings | Volltextsuche findet nachweislich nicht mehr |
| ICS-Kalender lesend | ein terminlastiges Projekt existiert real |
| Proton-Pass-CLI-Anbindung | Verweise werden häufig angeklickt und nerven |
| Rückfluss aus dem Klartext-Spiegel | Spiegel wird regelmäßig extern bearbeitet |
| Native Mobile-App | sechs Monate stabile Desktop- und Web-Nutzung |
| Sync-Dienst als eigenes Rust-Binary | Cloudflare ändert Bedingungen oder Selbsthosting wird gewünscht |
| Bezahlfunktionen, Konten-Pläne | siehe `BUSINESS.md` |
| Code-Signierung/Notarisierung | Verteilung an Dritte |
| Browser-Extension, VS-Code-Extension | nie geplant; CLI und MCP decken die Fälle ab |
| Notion-/Trello-/Obsidian-Import | nie geplant; Ordner-Erkennung ist der Import |
| Webhook-Engine mit Mapping-UI | nie geplant |

## Vorzeitig aufgenommen

| Feature | Auslöser laut Liste | Warum trotzdem |
|---|---|---|
| KI-Verdichtung des Briefs | „Brief regelmäßig länger als ein Bildschirm“ | Ausdrücklich vom Nutzer priorisiert. Aufnahmetest: Frage 1 ja (Wiedereinstieg), Frage 2 ja (nichts läuft von allein, jeder Aufruf ist ein Klick), Frage 3 nein. Der Grundsatz aus `CONCEPT.md` Abschnitt 9 ist eingehalten: Opt-in, vor jedem Senden ist der vollständige Text sichtbar, kein Tresor-Zugriff, und das Ergebnis landet nur im Logbuch, wenn der Mensch es übernimmt. |
| Remote-Git (Issues, PRs/MRs, Prüflauf) per Abruf, GitHub und GitLab | „lokaler Git-Log reicht nachweislich nicht“ | Ausdrücklich vom Nutzer priorisiert, bevor der Auslöser eingetreten war. Bewusst überschrieben, nicht übersehen. Der Aufnahmetest ist dabei eingehalten: Frage 1 ja (Faden halten), Frage 2 ja, Frage 3 nein. Issues werden **nicht** zu offenen Fäden — sonst wäre es die Ticketliste, die dauerhaft ausgeschlossen bleibt. Zu Frage 2 im Einzelnen: Das Repo erkennt Lotse am Git-Remote eines Ordners, es ist also nichts zu pflegen. Der Abruf läuft auf Knopfdruck; er kann zusätzlich im Ordner-Beobachter mitlaufen, das ist aber ausgeschaltet, bis jemand es einschaltet, hängt am Beobachter und fragt höchstens alle 30 Minuten. Auch dann entsteht keine Liste, die gepflegt werden will, sondern dieselbe eine Zeile im Logbuch. |
| Kalender lesend (.ics) | vom Nutzer priorisiert | Aufnahmetest: Frage 1 ja (der Wiedereinstieg braucht nicht nur „was war zuletzt“, sondern auch „was steht an“), Frage 2 ja (nichts zu pflegen: gelesen wird der Kalender, der ohnehin geführt wird; Lotse schreibt nichts hinein), Frage 3 nein — Lotse wird dadurch kein Kalender, weil es keine Termine anlegt, ändert oder erinnert. Termine landen auch nicht im Logbuch; sonst stünde derselbe Termin an zwei Orten und einer davon wäre irgendwann falsch. |

## Abbruchkriterium

Werden drei Monate nach Alltagstauglichkeit des Desktop-MVP nicht mindestens fünf echte
Projekte aktiv geführt und Lotse nicht wöchentlich freiwillig geöffnet, wird Lotse
eingefroren statt weiter ausgebaut. Lotse ist selbst Projekt Nr. 1 in Lotse; sein Logbuch
ist der Nachweis.
