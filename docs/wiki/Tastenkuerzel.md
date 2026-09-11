# Tastenkürzel

Wenige, dafür überall dieselben. Lotse legt keine Kürzel auf Buchstaben, die im
Betriebssystem schon etwas bedeuten – mit einer begründeten Ausnahme.

`Strg` unter Linux und Windows, `Cmd` auf dem Mac. Lotse nimmt beides an, egal auf welchem
System.

---

## Überall

| Kürzel | Was |
|---|---|
| **Strg/Cmd + K** | Schnellerfassung öffnen. |
| **Strg/Cmd + P** | Springen: Vorhaben suchen und hin. |
| **Alt + ←** | Zurück. |
| **Esc** | Die oberste Überlagerung schließen – Schnellerfassung, Springen, Brief, Rückfrage. |

### Warum Strg/Cmd + P

Das ist normalerweise *Drucken*. In Lotse gibt es nichts zu drucken: der Bestand ist ein
Logbuch, kein Dokument, und wer etwas auf Papier will, nimmt den Klartext-Spiegel
([[Export und Fluchtweg|Export-und-Fluchtweg]]).

Dafür ist *P* die Taste, die in jedem Editor »zu etwas springen« bedeutet – und genau das
tut sie hier.

---

## In der Schnellerfassung

| Kürzel | Was |
|---|---|
| **Enter** | Eintragen und schließen. |
| **Shift + Enter** | Zeilenumbruch, ohne einzutragen. |
| **Esc** | Verwerfen. |

Enter trägt ein, weil der häufigste Eintrag eine Zeile ist. Wer mehrere Zeilen schreibt,
merkt das beim ersten Umbruch und lernt Shift + Enter – umgekehrt hätte jeder einzeilige
Eintrag einen Klick mehr gekostet.

## Beim Springen

| Kürzel | Was |
|---|---|
| **↑ / ↓** | Durch die Treffer. |
| **Enter** | Hin. |
| **Esc** | Schließen. |

Getippt wird unscharf gesucht: »gart« findet *Gartenhaus*, »kasse« findet
*Vereinskasse 2026*.

## In Textfeldern

| Kürzel | Was |
|---|---|
| **Strg/Cmd + Enter** | Abschicken, ohne zur Schaltfläche zu greifen. |
| **Esc** | Abbrechen. |

---

## Was es nicht gibt

| Fehlt | Warum |
|---|---|
| Frei belegbare Kürzel | Eine Einstellung, die fünf Leute benutzen, kostet alle anderen eine Zeile in den [[Einstellungen]]. |
| Vim-Modus | Lotse ist kein Editor. |
| Kürzel für jede Ansicht | Vier Ansichten sind mit der Maus schneller erreicht als mit einem Kürzel, das man sich merken muss. |
| Globales Kürzel (aus anderen Programmen heraus) | Braucht ein Symbol im Infobereich und einen Hintergrundprozess. Steht auf der Liste, ist aber noch nicht gebaut. |

Bis dahin ist die Kommandozeile der schnellste Weg von außen:

```
lotse log "Beton bestellt" -p Gartenhaus
```

Wer das oft braucht, legt sich ein Kürzel in der Shell an – Beispiele in
[[Kommandozeile]].
