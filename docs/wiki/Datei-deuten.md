# Datei deuten

Eine einzelne Datei einem Sprachmodell vorlegen und fragen: *worum geht es hier, was ist
zu tun?* Gedacht für das, was man bekommt und nicht lesen will – der zwölfseitige
Nebenkostenbescheid, das Angebot des Dachdeckers, die Vereinssatzung.

Das ist bewusst etwas anderes als der [[Beobachter und Erkennung|Beobachter-und-Erkennung]].
Der sieht nur Namen und Zeitstempel und liest **nie** Inhalte. Hier gibst du ausdrücklich
**eine** Datei her, siehst den Auszug vor dir und entscheidest dann, ob er an ein Modell
geht.

Braucht ein eingerichtetes Ziel: [[KI]].

---

## Der Ablauf

1. Auf der Projektseite *Datei deuten*, Datei auswählen.
2. Lotse zieht den Text heraus und zeigt ihn dir – **vollständig**, so wie er gesendet
   würde.
3. Du entscheidest: senden oder abbrechen.
4. Das Ergebnis erscheint als Vorschlag. Du übernimmst es ins Logbuch (Quelle `ki`) oder
   verwirfst es.

Zwei Zustimmungen, zwei Entscheidungen: einmal, ob der Text das Gerät verlässt, einmal, ob
die Antwort ins Logbuch kommt. Nichts davon passiert von allein.

---

## Welche Formate

| Format | Endungen | Wie |
|---|---|---|
| Textdatei | `.txt`, `.log`, `.text` | direkt |
| Markdown | `.md`, `.markdown` | direkt |
| Tabelle (CSV) | `.csv`, `.tsv` | direkt |
| JSON | `.json` | direkt |
| PDF | `.pdf` | Textebene wird ausgelesen |

Alles andere lehnt Lotse ab. Kein `.docx`, kein `.xlsx`, keine Bilder.

### Warum nicht mehr

Jedes weitere Format ist eine weitere Bibliothek, die eine fremde Datei parst – und
Dateiparser sind seit dreißig Jahren die zuverlässigste Quelle für Sicherheitslücken. Die
fünf Formate hier sind die, bei denen der Nutzen den Aufwand trägt.

Für `.docx`: in ein PDF drucken oder als Text speichern.

### PDF ohne Textebene

Ein gescanntes PDF enthält Bilder, keinen Text. Lotse sagt das ausdrücklich, statt eine
leere Zusammenfassung zu liefern. Texterkennung baut Lotse nicht ein – das ist ein
eigenes Werkzeug.

---

## Was nie gelesen wird

Dieselbe Ausschlussliste wie beim Beobachter, an **einer** Stelle im Code für alle
Aufrufer:

```
.env    *.pem    *.key    id_rsa*    id_ed25519*    *.kdbx    *.p12
```

Auch wenn du eine solche Datei ausdrücklich auswählst. Ein Sicherheitsversprechen mit
Ausnahme für »aber ich wollte es doch« ist keines: die eine Datei, die man versehentlich
anklickt, ist genau der Fall, den die Liste abfangen soll.

---

## Grenzen

| Grenze | Wert |
|---|---|
| Auszug | auf eine Obergrenze gekürzt, die der Aufrufer setzt |
| Anfrage an das Modell | 40 000 Zeichen |

Ist der Text länger, sagt Lotse das, bevor gesendet wird – nicht danach, wenn die
Rechnung schon läuft.

---

## Was nicht passiert

| Nicht | Warum |
|---|---|
| Kein Ordner wird durchgegangen | Es gibt keine Schleife. Der Aufrufer nennt **genau eine** Datei. Ein »alle PDFs im Ordner deuten« wäre genau die stille Massenübertragung, die Lotse nicht macht. |
| Der Auszug wird nicht gespeichert | Er wird zurückgegeben und ist danach weg. Was bleibt, ist die Antwort – und nur, wenn du sie übernimmst. |
| Die Datei wird nicht kopiert | Sie bleibt, wo sie ist. Wer sie dauerhaft am Vorhaben haben will, legt eine [[Referenz|Referenzen]] an. |
| Kein Tresor-Zugriff | Das Modul importiert nicht aus `vault`, nachgewiesen bei jedem Bau ([[Sicherheit]]). |

---

## Die Anweisung an das Modell

Festgelegt, nicht frei eingebbar:

> Du liest ein Dokument für jemanden, der wenig Zeit hat. Schreibe auf Deutsch: worum es
> geht, die wichtigsten Zahlen, Fristen und Namen, und was daraus zu tun wäre. Höchstens
> zehn Sätze. Nenne nur, was im Text steht; wenn etwas fehlt oder unklar ist, sage das,
> statt es zu ergänzen. Keine Anrede, keine Überschrift.

Der letzte Satz ist der wichtigste. Ein Modell, das eine fehlende Frist erfindet, ist
schlimmer als keine Zusammenfassung – deshalb steht ausdrücklich da, dass es Lücken
benennen soll.

Ein freies Eingabefeld gibt es nicht. Jede Fähigkeit bekommt ihren eigenen Zweck mit
eigener Anweisung, weil sonst jede neue Fähigkeit eine Kopie des Zustimmungsflusses wäre –
und irgendeine Kopie hätte den Hinweis dann nicht.

---

## Was im Protokoll landet

Jede Anfrage wird mitgeschrieben: wann, Zweck *Datei deuten*, Zeichen, Token hin und
zurück. Nachzulesen in den [[Einstellungen]] unter *KI* → *Was bisher gesendet wurde*.
Der Text selbst wird **nicht** protokolliert.

---

## Das Ergebnis ist ein Vorschlag

Es wird nie automatisch zu einer Notiz und nie zu einem offenen Faden. Du übernimmst es
oder du verwirfst es.

Übernommen bekommt es die Quelle `ki`. Damit ist in drei Monaten sichtbar, dass diese
Zeile von einem Modell kam und nicht von dir – der Unterschied, der zählt, wenn eine
Angabe sich als falsch erweist ([[Logbuch]]).
