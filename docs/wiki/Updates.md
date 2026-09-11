# Updates

Seit **v0.6.0** hält Lotse sich selbst aktuell: es lädt die neue Fassung, prüft ihre
Signatur und tauscht sich aus.

## Wie es aussieht

Einstellungen → *Version*. Steht dort eine neuere Fassung, gibt es einen Knopf *Jetzt
aktualisieren*. Lotse lädt, zeigt den Fortschritt, **sperrt**, tauscht sich aus und
startet neu.

Das Sperren ist Absicht: der Schlüssel liegt im Speicher dieses Prozesses und soll den
Neustart nicht überleben.

## Warum das sicher ist

Ein Programm, das sich mit Heruntergeladenem überschreibt, ist ein bequemer Weg für
jeden, der die Verbindung oder das Konto kontrolliert. Deshalb hat Lotse bis v0.5.0
**nur Bescheid gesagt** – die Bauten waren unsigniert.

Jetzt wird jede Fassung mit einem Schlüssel unterschrieben (minisign). Die öffentliche
Hälfte steckt in der App:

```
4BDDE1BB5195DE67
```

Was nicht dazu passt, wird verworfen. Der private Schlüssel liegt ausschließlich als
GitHub-Geheimnis; er ist nie im Quelltext und nie auf einem Entwicklerrechner im
Klartext.

> **Das ist nicht Code-Signierung.** Windows SmartScreen und macOS Gatekeeper warnen
> beim *ersten* Start weiterhin. Die Update-Signatur schützt den Weg von einer Fassung
> zur nächsten, nicht den ersten Download.

## Wo es nicht geht

| Installation | Selbst-Austausch |
|---|---|
| Windows `.exe` / `.msi` | ja |
| macOS `.dmg` (aus `/Programme`) | ja |
| Linux **AppImage** | ja |
| Linux `.deb` / `.rpm` | **nein** |

Aus einem Paket heraus gehört die Installation der **Paketverwaltung**. Sich dort selbst
zu überschreiben würde deren Buchführung zerreißen. Lotse merkt das am fehlenden
`APPIMAGE` in der Umgebung, versucht es gar nicht erst und zeigt stattdessen den
Download-Weg samt Begründung.

Wer Updates aus der App will, nimmt unter Linux das **AppImage**.

## Nachsehen, ohne zu installieren

Das Nachsehen ist getrennt vom Austauschen und lässt sich abschalten (Einstellungen →
*Version*). Eingeschaltet sieht Lotse beim Entsperren nach, höchstens einmal am Tag.

Nachsehen heißt: **eine Anfrage** an die Veröffentlichungen des Projekts. GitHub sieht
dabei die IP-Adresse, sonst nichts – kein Konto, keine Kennung, keine Nutzungsdaten.

```bash
lotse update
```

Die Kommandozeile sagt nur Bescheid und nennt die Datei für dein System. Sie lädt nichts
herunter: eine CLI, die sich selbst ersetzt, wäre eine Überraschung im falschen Moment.

## Wo die Liste liegt

Zwei Adressen, in fester Reihenfolge:

1. `https://phish3144.github.io/lotse/latest.json`
2. `https://github.com/phish3144/lotse/releases/latest/download/latest.json`

Beide stehen fest im Programm und lassen sich nicht zur Laufzeit umbiegen – wer sie
ändern will, muss eine neue Fassung ausliefern, und die müsste wiederum signiert sein.

Warum zwei? Die Veröffentlichungen sind Vorabversionen, und GitHub lässt Vorabversionen
aus `/releases/latest` heraus – über die zweite Adresse findet der Updater also erst
etwas, wenn eine Fassung keine Vorabversion mehr ist. Bis dahin trägt die erste allein,
und der Release-Workflow spiegelt die Liste dafür auf die Landing Page.

> In 0.6.0 ist diese Spiegelung ausgefallen (ein `git diff` sah die noch nicht verfolgte
> Datei nicht und meldete »unverändert«). Beide Adressen antworteten deshalb mit 404 und
> 0.6.0 fand nie ein Update. Behoben mit 0.6.1; da die Adresse außerhalb der App liegt,
> findet auch ein installiertes 0.6.0 danach wieder etwas.

## Wenn etwas schiefgeht

| Meldung | Was tun |
|---|---|
| „Die Liste der Fassungen war nicht erreichbar“ | Netz prüfen. Der Download von Hand geht weiterhin – *Was ist neu* führt hin. |
| „Austausch fehlgeschlagen“ auf macOS | Liegt Lotse in `/Programme`? Aus dem DMG heraus geht es nicht. |
| „Austausch fehlgeschlagen“ auf Windows | Rechte im Installationsordner. Notfalls den Installer von Hand ausführen. |
| Hinweis auf `.deb`/`.rpm` | Kein Fehler. `apt`/`dnf` benutzen oder auf AppImage wechseln. |

Ein misslungener Austausch lässt die alte Fassung stehen. Lotse überschreibt sich erst,
wenn die neue Datei geladen **und** ihre Signatur geprüft ist.
