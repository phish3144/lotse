# Verbindungen

Lotse holt aus zwei Quellen etwas dazu, was der Ordner allein nicht weiß. Beides ist
optional und lesend.

---

## GitHub und GitLab

Holt zu Vorhaben mit einem Repo auf der Gegenseite: offene Pull- bzw. Merge-Requests,
die Zahl offener Issues und ob der Prüflauf rot ist. Das landet als **eine verdichtete
Zeile** im Logbuch.

> Issues werden **nicht** zu offenen Fäden. Tickets und Backlogs sind erklärtes
> Nicht-Ziel (siehe **[[Nicht-Ziele]]**); ein Ticketsystem in Lotse hineinzuspiegeln
> würde aus dem Logbuch ein schlechtes Jira machen.

### Verbinden

Einstellungen → *Verbindungen* → GitHub oder GitLab → Token einfügen → *Verbinden*.

Ein Feld, ein Knopf. Den Tresor-Eintrag legt Lotse selbst an; das Token liegt dort
verschlüsselt und wird nur beim Abfragen gelesen.

Berechtigungen, mehr braucht es nicht:

| Hoster | Bereich | Token erstellen |
|---|---|---|
| GitHub | `repo` | [Settings → Tokens](https://github.com/settings/tokens) |
| GitLab | `read_api` | [Personal Access Tokens](https://gitlab.com/-/user_settings/personal_access_tokens) |

Je Hoster ein eigener Eintrag – **ein GitHub-Token geht nie an GitLab**.

Ohne Token geht es auch, dann aber nur für öffentliche Repos und mit einem kleinen
Kontingent pro Stunde.

*Trennen* vergisst den Zeiger und lässt das Geheimnis im Tresor stehen. Löschen ist
Sache des Tresors – dort still etwas wegzuräumen wäre eine Überraschung.

### Wie Lotse das Repo findet

Zwei Wege, in dieser Reihenfolge:

1. **Eine Referenz mit Adresse**: `https://github.com/name/repo` oder
   `git@github.com:name/repo.git`.
2. **Eine Ordner-Referenz**: Lotse liest das Git-Remote des Ordners selbst. Das macht
   am wenigsten Arbeit – ein geklonter Arbeitsordner genügt.

### Wann abgefragt wird

Auf Knopfdruck, oder mit dem Ordner-Beobachter zusammen, wenn *Mit dem Ordner-Beobachter
mitlaufen lassen* eingeschaltet ist: höchstens alle 30 Minuten, damit das Kontingent
reicht.

```bash
lotse gegenseite
```

---

## Kalender

Lotse **liest** Kalender, es führt keinen. Termine wandern nicht ins Logbuch – sie
bleiben dort, wo sie gepflegt werden.

### Einrichten

Auf der Projektseite unter **Referenzen** eine Referenz vom Typ *URL* mit der
Abonnement-Adresse anlegen. Sie endet auf `.ics` oder beginnt mit `webcal://`; jeder
gängige Dienst gibt so eine Adresse aus (Google Kalender, Nextcloud, Outlook, Proton).

Auf der Projektseite steht dann rechts unter **Was ansteht**, was in den nächsten 90
Tagen kommt.

```bash
lotse termine --tage 90
```

### Was Lotse kann und was nicht

- Geholt wird der Kalender beim Öffnen der Projektseite, danach höchstens alle 15 Minuten neu.
- **Uhrzeiten stehen so da, wie sie im Kalender stehen.** Ohne Zeitzonendatenbank wäre jede Umrechnung geraten – lieber ehrlich unverändert.
- **Wiederholungen** rechnet Lotse aus, solange die Regel einfach ist: täglich, wöchentlich, monatlich, jährlich, auch mit Wochentagen.
- Bei komplizierteren Regeln – „zweiter Montag im Monat“ – steht nur der erste Termin und der Hinweis, dass er sich wiederholt. Falsch zu rechnen wäre schlimmer als es zuzugeben.

---

## Was nach außen geht

Alle Abrufe – Sync-Dienst, GitHub, GitLab, Kalender, KI-Ziel – laufen über **eine**
Stelle im Programm (`netz.rs`), über TLS mit dem Wurzelspeicher des Betriebssystems.
Das ist dieselbe Wahl, die Browser, `git` und `curl` auf demselben Rechner treffen: in
Netzen mit TLS-Prüfung (Firmen-Proxy) funktioniert Lotse dann wie alles andere auch.

Ohne Verbindungen, ohne Abgleich und ohne KI redet Lotse mit niemandem außer der
Update-Prüfung – und auch die lässt sich abschalten.
