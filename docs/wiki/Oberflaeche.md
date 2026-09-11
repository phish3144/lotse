# Die Oberfläche

Fünf Bereiche in der Kopfzeile, zwei Dialoge auf Tastendruck.

| Taste | Was |
|---|---|
| <kbd>Strg</kbd>+<kbd>K</kbd> | Schnellerfassung – von überall etwas ins Logbuch |
| <kbd>Strg</kbd>+<kbd>P</kbd> | Springen – zu einem Vorhaben |
| <kbd>Alt</kbd>+<kbd>←</kbd> | Zurück |
| <kbd>Strg</kbd>+<kbd>Enter</kbd> | Im Erfassungsfeld: speichern |

---

## Hafen

Die Startseite. Links die Vorhaben in vier Blöcken (**Heute wichtig**, **Auf See**,
**Vor Anker**, **Ideen**), rechts die **Hafeneinfahrt**.

Jede Karte zeigt Titel, Kurs, den letzten Logbuch-Eintrag und wie lange er her ist. Der
Zustand steht zweimal: als Punkt neben dem Titel und als farbige Kante links – auf einem
Raster mit einem Dutzend Karten findet das Auge die Kante zuerst.

Ist noch nichts da, zeigt der Hafen stattdessen drei Wege hinein, jeden mit dem Knopf
daneben, der ihn geht.

## Projektseite

Zweispaltig. **Links ist das Logbuch die Seite**, rechts steht alles Begleitende.

### Links

- **Erfassung** ganz oben: schreiben, Art wählen, eintragen. Nicht unten, weil man
  zuerst schreiben will und nicht zuerst scrollen.
- **Logbuch** als Zeitachse: eine Marke je Tag, Einträge an einer Linie, Punkte in der
  Farbe der Quelle. Der Pfeil am Eintrag ordnet ihn einem anderen Vorhaben zu; er
  erscheint erst, wenn man den Eintrag anfasst.

### Rechts

| Karte | Inhalt |
|---|---|
| **Kurs** | Der eine Satz, plus Vorlage, Erwartungsintervall, Wiedervorlage, Tags. |
| **Offene Fäden** | Abhakbar. |
| **Was ansteht** | Termine aus abonnierten Kalendern, 90 Tage voraus. Nur wenn ein `.ics`-Abo als Referenz hinterlegt ist. |
| **Referenzen** | Wo das Material liegt, mit Prüfstatus. |
| **Zugänge** | Tresor-Einträge dieses Vorhabens. |
| **Datei deuten** | Eine einzelne Datei aufmachen und auslesen. |

Ganz oben, wenn das Vorhaben lange geruht hat: der **Wo-war-ich-Brief**.

Unter 1024 px Breite fällt die rechte Spalte nach unten – gleiche Reihenfolge, nur
gestapelt.

## Offene Punkte

Alle offenen Fäden über alle Vorhaben, **gebündelt nach Vorhaben** und nicht nach Datum:
ein Faden ohne sein Projekt ist eine Zeile ohne Zusammenhang.

Die Bündel stehen nach Auffälligkeit des Vorhabens – was lange ruht, steht oben, weil
dort am ehesten etwas hängen bleibt.

Rechts: die am längsten offenen Fäden, und die aktiven Vorhaben *ohne* Faden. Letzteres
ist kein Vorwurf, nur ein Hinweis: dort weiß beim nächsten Mal niemand, wo es weitergeht.

## Suche

Es gibt keine Überschrift „Suche“ über dem Suchfeld – das Feld ist die Seite.

Ein Strom statt vier getrennter Listen, links die Arten als Filter mit Zahlen. Der
Treffer wird im Text unterstrichen, nicht eingefärbt. Bei mehrzeiligen Einträgen zeigt
Lotse die Zeile **mit** dem Treffer, nicht stur die erste.

Gesucht wird über Projekte, Logbücher, Referenzen und **Tresor-Titel**. Tresor-*Werte*
sind nie dabei – weder hier noch über die Schnittstelle für Assistenten.

## Tresor

Alle Zugänge als Raster. „nur Desktop“ steht als Etikett *und* als Kante links. Werte
bleiben Punkte, bis du *Zeigen* drückst. Siehe **[[Tresor]]**.

## Einstellungen

Sieben Reiter. Der gewählte steht in der Adresse (`#/einstellungen/ki`), damit
Lesezeichen und die Zurück-Taste funktionieren.

| Reiter | Inhalt |
|---|---|
| **Ordner** | Wurzelordner durchsuchen, Beobachter starten |
| **Verbindungen** | MCP-Zugang, GitHub/GitLab, Kalender |
| **Abgleich** | Sync-Dienst, Geräte |
| **KI** | Ziel, Modell, Verbrauch – standardmäßig aus |
| **Export** | Spiegel und Bundle |
| **Sicherheit** | Passwort ändern, automatisches Sperren, sperren |
| **Version** | Fassung, Updates |

## Schnellerfassung

<kbd>Strg</kbd>+<kbd>K</kbd>, von überall. Tippen, `@projekt` für die Zuordnung (Lotse
schlägt beim Tippen vor), Art wählen, Enter.

Ohne Zuordnung landet der Gedanke im **Postkorb**. Nichts geht verloren, während du
überlegst.

## Springen

<kbd>Strg</kbd>+<kbd>P</kbd>. Tippen, Pfeiltasten, Enter. Ersetzt keine Navigation –
macht sie überflüssig, solange du weißt, wohin du willst.

---

## Dunkel und hell

Lotse folgt der Systemeinstellung. Die Farben stammen aus dem App-Symbol: Marineblau als
Text, Stahlblau für Links, Pergament als Grund, Bernstein für **genau eine** Handlung je
Ansicht und für die Zustandspunkte. Im dunklen Thema wird der Logo-Verlauf zur Fläche.

Kein Umschalter in der App – dein System weiß besser als Lotse, wann es dunkel sein soll.

## Automatisches Sperren

Nach 15 Minuten ohne Regung sperrt Lotse von selbst (Einstellungen → *Sicherheit*,
`0` schaltet es ab). 30 Sekunden vorher erscheint ein Band mit der Möglichkeit
abzubrechen.

Sperren heißt: der Schlüssel verlässt den Speicher. Der Ordner-Beobachter und der
MCP-Zugang enden mit.

---

## Tiefer

Diese Seite ist der Überblick. Im Einzelnen:

| Ansicht | Seite |
|---|---|
| Hafen, Auffälligkeit, vor Anker | [[Vorhaben]] |
| Zeitachse, Arten, Quellen | [[Logbuch]] |
| Rechte Spalte der Projektseite | [[Referenzen]], [[Kalender]], [[Gegenseite]] |
| Die Überlagerung beim Öffnen | [[Wo-war-ich-Brief]] |
| Offene Punkte | [[Offene Punkte|Offene-Punkte]] |
| Tresor-Ansicht | [[Tresor]] |
| Einstellungen, alle sieben Reiter | [[Einstellungen]] |
| Schnellerfassung, Springen, Esc | [[Tastenkürzel|Tastenkuerzel]] |

Was die Oberfläche im Kern aufrufen kann, steht in [[App-Schnittstelle]].
