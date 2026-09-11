# Die App-Schnittstelle

Jedes Kommando, das die Oberfläche im Kern aufrufen kann. Das ist **keine öffentliche
API** – sie liegt zwischen Tauri-Hülle und Oberfläche, ist nicht stabil und von außen
nicht erreichbar.

Sie steht hier trotzdem vollständig, aus zwei Gründen: sie ist das genaueste Verzeichnis
dessen, was Lotse kann, und wer die Oberfläche ändert oder eine eigene baut, braucht sie.

Wer die Anwendung von außen ansprechen will, nimmt die [[Kommandozeile-Referenz]] oder
[[Assistenten (MCP)|Assistenten-MCP]].

---

## Aufbau

Drei Schichten, eine Liste:

```
apps/web/src/lib/data/provider.ts   ← was die Oberfläche verlangt
apps/desktop/src-tauri/src/lib.rs   ← was die Hülle liefert
crates/lotse-core                   ← wo es wirklich passiert
```

`provider.ts` ist die Schnittstelle: `tauri.ts` spricht den Kern, `mock.ts` liefert
Beispieldaten im Browser. Deshalb läuft dieselbe Oberfläche in der Desktop-App und im
Browser, und deshalb muss jedes neue Kommando an **beiden** Stellen auftauchen.

Zeitstempel werden nur in `tauri.ts` von Millisekunden in ISO umgerechnet – eine Stelle,
damit es nicht an fünf Orten halb passiert.

---

## Konto und Sitzung

| Kommando | Argumente | Was |
|---|---|---|
| `konto_status` | – | Gibt es ein Konto, ist es entsperrt, wo liegt der Datenordner. |
| `einrichten` | `passwort`, `geraet` | Richtet das Konto ein. Liefert Wiederherstellungscode und Desktop-Schlüssel **zur einmaligen Anzeige**. |
| `wiederherstellungscode_pruefen` | `code` | Kalte Wiedereingabe zur Bestätigung nach der Einrichtung. |
| `entsperren` | `passwort`, `desktop_schluessel` | Öffnet die Sitzung. |
| `sperren` | – | Beendet sie. Der Schlüssel verlässt den Speicher; Beobachter und MCP-Zugang enden mit. |
| `konto_wiederherstellen` | `code`, `neues_passwort` | Der Weg für ein vergessenes Master-Passwort. |
| `passwort_aendern` | `altes_passwort`, `neues_passwort`, `code` | Wechselt das Passwort. Der Wiederherstellungscode muss dabei neu verankert werden, weil er am Salt hängt – daher der neue Code als Rückgabe. |
| `auto_lock` / `auto_lock_setzen` | – / `minuten` | Minuten bis zum selbsttätigen Sperren; `0` = nie. |
| `kann_nur_desktop` | – | Ist ein Desktop-Schlüssel vorhanden? Entscheidet, ob die Stufe »nur Desktop« angeboten wird. |

Siehe [[Schlüssel und Krypto|Schluessel-und-Krypto]] und [[Einstellungen]].

---

## Vorhaben

| Kommando | Argumente | Was |
|---|---|---|
| `hafen` | – | Die Startseite: Karten mit Auffälligkeit, Tagen, offenen Fäden. |
| `projekte` | – | Alle Vorhaben. |
| `projekt` | `id` | Eines. |
| `projekt_anlegen` | `titel`, `vorlage`, `kurs` | |
| `projekt_speichern` | `projekt` | Das Ganze zurückschreiben. |
| `projekt_loeschen` | `id` | Mit Logbuch, Referenzen und Crypto-Shredding der Tresor-Einträge. |
| `status_setzen` | `projekt_id`, `status`, `uebergabe`, `wiedervorlage` | `pausiert` und `wartet` verlangen eine Übergabe. |
| `postkorb` | – | Das Auffangprojekt. Wird beim ersten Zugriff angelegt. |
| `brief` | `projekt_id` | Der [[Wo-war-ich-Brief|Wo-war-ich-Brief]]. |

Feldbedeutungen: [[Vorhaben]], [[Datenmodell]].

---

## Logbuch

| Kommando | Argumente | Was |
|---|---|---|
| `notizen` | `projekt_id` | Das Logbuch eines Vorhabens. |
| `notiz` | `id` | Eine Notiz. |
| `notiz_anlegen` | `projekt_id`, `text`, `art` | Quelle ist `mensch`. |
| `faden_erledigen` | `id` | Setzt `erledigt_am`; löscht nichts. |
| `notiz_verschieben` | `id`, `projekt_id` | Der Weg aus dem Postkorb. Der Text bleibt unangetastet. |
| `offene_faeden` | – | Alle offenen Fäden ([[Offene Punkte\|Offene-Punkte]]). |
| `suche` | `anfrage` | Volltextsuche. Tresor-Inhalte sind nicht im Index. |

Siehe [[Logbuch]].

---

## Referenzen und Hafeneinfahrt

| Kommando | Argumente | Was |
|---|---|---|
| `referenzen` | `projekt_id` | |
| `referenz_anlegen` | `projekt_id`, `typ`, `ziel`, `rolle` | |
| `referenz_pruefen` | `id` | Pfade auf der Platte, Adressen im Netz. Der Netzteil läuft **außerhalb** der Sitzungssperre – sonst stünde die ganze Oberfläche, bis eine langsame Adresse antwortet. |
| `kandidaten` | – | Die Hafeneinfahrt. |
| `kandidat_uebernehmen` | `pfad`, `titel` | Legt Vorhaben, Ordner-Referenz und Markerdatei an. |
| `kandidat_verwerfen` | `pfad` | |
| `scan` | `wurzeln` | Durchsucht Ordner ([[Erkennungsregeln]]). |
| `ordner_waehlen` | – | Systemdialog. `None` bei Abbruch – spart das Abtippen von Pfaden. |
| `datei_waehlen` | `name` | Systemdialog für einen Speicherort. |
| `oeffnen` | `ziel` | Öffnet eine Referenz im System: Ordner im Dateimanager, Adresse im Browser. **Nur auf ausdrücklichen Klick**, nie von allein. |

Siehe [[Referenzen]].

---

## Beobachter

| Kommando | Argumente | Was |
|---|---|---|
| `beobachter_status` | – | Läuft er, welche Wurzeln. |
| `beobachter_starten` | `wurzeln` | Im Hintergrund. Der Thread verarbeitet Ereignisse, ohne die Sitzung zu sperren. |
| `beobachter_stoppen` | – | |

Siehe [[Beobachter und Erkennung|Beobachter-und-Erkennung]].

---

## Tresor

| Kommando | Argumente | Was |
|---|---|---|
| `tresor_liste` | `projekt_id` | Titel und Feldnamen, **nie** Werte. |
| `tresor_anlegen` | `titel`, `projekt_id`, `felder`, `stufe` | |
| `tresor_feld_lesen` | `id`, `feld` | Entschlüsselt **genau ein** Feld. Die Oberfläche zeigt es und verwirft es wieder. |
| `tresor_loeschen` | `id` | Crypto-Shredding. |

Nur die Oberfläche und die Kommandozeile dürfen Tresor-Werte lesen – kein anderes Modul.
Siehe [[Tresor]].

---

## Verbindungen

| Kommando | Argumente | Was |
|---|---|---|
| `forge_status` | – | Welche Hoster verbunden sind. |
| `forge_token_einfuegen` | `anbieter`, `token` | Nimmt ein Token und erledigt den Rest: Tresor-Eintrag anlegen, Zeiger merken. Vorher waren das sechs Schritte von Hand. |
| `forge_trennen` | `anbieter` | Lotse benutzt das Token nicht mehr. |
| `forge_token_setzen` | `anbieter`, `eintrag_id`, `feld` | Für ein Token, das schon im Tresor liegt. Leere Kennung löst die Bindung. |
| `forge_auto_setzen` | `an` | Selbsttätige Abfrage im Beobachter. |
| `forge_projekt` | `projekt_id` | Das Repo eines Vorhabens, für die Projektseite. |
| `forge_abfragen` | `projekt_id` | Von Hand angestoßen; ohne Angabe alle mit erkanntem Repo. |
| `kalender_termine` | `projekt_id`, `tage` | Anstehende Termine aus den Kalender-Referenzen. |

Siehe [[Gegenseite]] und [[Kalender]].

---

## KI

| Kommando | Argumente | Was |
|---|---|---|
| `ki_status` | – | Ist ein Ziel eingerichtet. |
| `ki_ziel_setzen` | `basis_url`, `modell`, `schluessel_eintrag`, `schluessel_feld` | Der Schlüssel liegt im Tresor; hier steht nur der Zeiger. |
| `ki_modelle` | `basis_url` | Modelle eines Ziels, damit niemand einen Namen abtippen muss. |
| `ki_anfrage_text` | `projekt_id` | **Genau der Text, der gesendet würde** – die Oberfläche zeigt ihn, bevor etwas das Gerät verlässt. |
| `ki_verdichten` | `zweck`, `eingabe` | Die Anfrage. |
| `ki_verbrauch` | – | Zähler und die letzten Protokolleinträge. |
| `ki_verbrauch_loeschen` | – | Was Lotse über den eigenen Gebrauch führt, muss man auch loswerden können. |
| `datei_auszug` | `pfad` | Macht **eine** ausdrücklich gewählte Datei auf. Gespeichert wird dabei nichts. |
| `datei_referenzen` | `projekt_id` | Welche Dateien eines Vorhabens Lotse aufmachen kann. |

Dass `ki_anfrage_text` existiert und vor jeder Anfrage aufgerufen wird, ist die technische
Form der Zusage »nichts fließt still ab«. Siehe [[KI]] und
[[Datei deuten|Datei-deuten]].

---

## Assistenten (MCP)

| Kommando | Argumente | Was |
|---|---|---|
| `mcp_status` | – | Läuft er, auf welchem Port. |
| `mcp_starten` | `port` | Öffnet den Dienst auf `127.0.0.1`. Er lebt nur, solange die Sitzung entsperrt ist. |
| `mcp_stoppen` | – | |
| `mcp_token_erneuern` | – | Neues Token. Der Dienst wird dabei angehalten – ein Assistent mit dem alten Token soll nicht weiterreden. |

Siehe [[Assistenten (MCP)|Assistenten-MCP]].

---

## Abgleich

| Kommando | Argumente | Was |
|---|---|---|
| `sync_status` | – | Ausstehende Änderungen, Stand lokal und entfernt. |
| `sync_register` | `url`, `email`, `code` | Bestehendes Konto anmelden. Braucht den Wiederherstellungscode. |
| `sync_login` | `url`, `email`, `passwort`, `geraet` | Neues Gerät – ersetzt die Einrichtung. |
| `sync_jetzt` | – | Pushen, dann pullen. |
| `sync_geraete` | – | |
| `sync_geraet_widerrufen` | `id` | Das eigene lässt sich nicht widerrufen – dafür gibt es Sperren. |

Siehe [[Abgleich]] und [[Sync-Protokoll]].

---

## Export

| Kommando | Argumente | Was |
|---|---|---|
| `export_spiegel` | `ziel` | Markdown, das `grep` und Obsidian lesen. **Ohne** Tresor-Werte. |
| `export_bundle` | `ziel`, `passphrase` | Alles als JSON, mit `age` verschlüsselt. Ohne Lotse zu öffnen. |

Siehe [[Export und Fluchtweg|Export-und-Fluchtweg]].

---

## Updates

| Kommando | Argumente | Was |
|---|---|---|
| `update_pruefen` | `erzwingen` | Sieht nach. Lädt nichts und führt nichts aus. |
| `update_installieren` | – | Holt die neue Fassung, prüft die Signatur, tauscht aus, startet neu. Sperrt vorher – der Schlüssel überlebt einen Neustart nicht. |
| `update_automatisch_setzen` | `an` | Nachsehen beim Start. |

Während des Austauschs sendet die Hülle Fortschrittsmeldungen (`update-fortschritt`), aus
denen die Oberfläche den Balken baut. Siehe [[Updates]].

---

## Wenn du hier etwas hinzufügst

Ein neues Kommando muss an **drei** Stellen auftauchen:

1. `crates/lotse-core` – wo es passiert.
2. `apps/desktop/src-tauri/src/lib.rs` – als `#[tauri::command]`, und in der Liste in
   `invoke_handler`.
3. `apps/web/src/lib/data/provider.ts` **und** `mock.ts` – sonst fällt die Browser-Fassung
   aus.

Und im Wiki, auf dieser Seite. `scripts/wiki_pruefen.sh` liest die
`#[tauri::command]`-Funktionen ab und wird rot, wenn eine nirgends beschrieben ist – siehe
[[Wiki pflegen|Wiki-pflegen]].
