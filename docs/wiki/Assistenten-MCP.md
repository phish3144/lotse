# Assistenten (MCP)

Ein KI-Assistent wie Claude Code kann Projektkontext **lesen** und ins Logbuch
**schreiben** – über das Model Context Protocol.

Der Nutzen: der Assistent weiß beim Start, wo du stehengeblieben bist, und trägt am Ende
einer Sitzung ein, was er getan hat. Aus Lotses Sicht ist das eine Quelle wie jede
andere; die Einträge tragen die Quelle `mcp` und lassen sich sammelweise filtern und
löschen.

## Die fünf Werkzeuge

| Werkzeug | Was es liefert |
|---|---|
| `list_projects` | Alle Vorhaben mit Status, Kurs, Tagen seit letztem Kontakt, Zahl offener Fäden. |
| `get_project_context` | Der Wo-war-ich-Brief: Kurs, letzte Übergabe, offene Fäden, Aktivität, Referenzen, letzte Logbuch-Einträge. |
| `log_activity` | Einen Eintrag ins Logbuch schreiben. |
| `list_open_threads` | Offene Fäden, projektübergreifend. |
| `search` | Volltextsuche über Projekte, Logbuch und Referenzen. |

**Es gibt kein Werkzeug für den Tresor.** Keines, das Einträge auch nur auflistet. Das
ist im Kern festgelegt und wird bei jedem Commit geprüft – nicht hier eingestellt. Die
Volltextsuche wirft Treffer der Art *tresor* weg, bevor sie antwortet.

## Zwei Wege

### Aus der laufenden App (empfohlen)

Einstellungen → *Verbindungen* → **Zugang für Assistenten** → *Öffnen*.

Lotse öffnet einen Zugang auf `127.0.0.1` und zeigt eine fertige Zeile zum Einfügen:

```bash
claude mcp add --transport http lotse http://127.0.0.1:7457/mcp \
  --header "Authorization: Bearer <token>"
```

Der Zugang **benutzt die schon entsperrte Sitzung** – es muss kein Passwort in eine
Konfigurationsdatei. Er schließt sich, sobald Lotse sperrt; nach dem Entsperren öffnest
du ihn hier wieder.

Warum nicht automatisch wieder auf? Ein Port, der sich nach jedem Entsperren von selbst
öffnet, ist kein Opt-in mehr.

### Über stdin/stdout

```bash
claude mcp add lotse -- lotse mcp
```

Hier startet der **Client** den Prozess. Der braucht dann das Master-Passwort in der
Umgebung (`LOTSE_PASSWORD`) – deshalb ist das der Weg für Skripte und Dienste, nicht für
den Alltag.

## Wie der Zugang abgesichert ist

In dieser Reihenfolge geprüft:

| Maßnahme | Wogegen |
|---|---|
| Lauscht nur auf `127.0.0.1`, nie auf `0.0.0.0` | Zugriff aus dem lokalen Netz |
| `Origin` muss localhost sein, wenn gesetzt; keine CORS-Kopfzeilen, keine Antwort auf `OPTIONS` | **DNS-Rebinding** – eine Seite im Browser, die sich als lokaler Client ausgibt |
| `Authorization: Bearer` mit 32 Byte Zufall, verglichen ohne frühen Abbruch | ein anderes Programm auf demselben Rechner; Rückschlüsse aus der Laufzeit |
| Endet mit dem Sperren – und *Sperren* schließt den Port sofort | Zugriff, während niemand am Rechner ist |
| Standardmäßig aus | ein stiller Port |
| Zeitgrenzen, 8 KiB Kopf, 1 MiB Rumpf | hängende oder überlange Anfragen |

Token und Port stehen im lokalen Speicher und werden **nicht** abgeglichen – der Zugang
gilt für dieses Gerät.

*Token erneuern* schließt den Zugang und macht alle bisher eingetragenen Token ungültig.

**Was nicht verteidigt wird:** ein anderes Programm mit deinen Rechten auf demselben
Rechner. Es kann den lokalen Speicher lesen, sobald Lotse entsperrt ist – der MCP-Port
ändert daran nichts, er ist nur kein zusätzlicher Weg dorthin.

## Protokollfassungen

Lotse bedient `2025-06-18`, `2025-03-26` und `2024-11-05`. Bittet ein Client um eine
davon, bekommt er sie bestätigt; sonst nennt Lotse die eigene, neueste. Für einen
Server, der nur Werkzeuge anbietet, ist das Drahtformat in allen dreien dasselbe.

## Nachweis statt Behauptung

Dass der Tresor unerreichbar ist, steht nicht nur als Zusage da. Der Test
`crates/lotse-core/tests/mcp_http.rs` legt einen echten Tresor-Eintrag mit Passwort an,
startet den Zugang auf einem echten Socket und sucht darüber danach – weder Wert noch
Benutzername noch Titel tauchen auf. Entfernt man den Filter, schlägt der Test an.

## Ein guter Ablauf

Sag deinem Assistenten einmal, was er tun soll. Zum Beispiel in `CLAUDE.md`:

```markdown
Hol zu Beginn den Kontext mit `get_project_context` und trage am Ende
mit `log_activity` kurz ein, was getan wurde und was der nächste Schritt ist.
```

Genau das schlägt Lotse dem Assistenten beim Handschlag auch selbst vor.
