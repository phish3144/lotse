# Lotse – Web-UI (Skelett)

Svelte 5 (Runes) + Vite + TypeScript. Diese Oberfläche ist eine einzige Codebasis für
zwei Auslieferungswege (siehe `docs/CONCEPT.md` Abschnitt 2 und 10):

- **Desktop:** dieselbe Oberfläche läuft im Tauri-2-Webview.
- **Web:** dieselbe Oberfläche läuft eigenständig, ausgeliefert über Cloudflare Pages,
  hinter Cloudflare Access.

Beide Auslieferungswege sprechen später denselben `lotse-core` (Rust) an – nativ im
Desktop, als WebAssembly im Browser. **In diesem Stand gibt es noch keinen Rust-Kern.**
Die Datenschicht ist ein In-Memory-Mock hinter einer festen Schnittstelle (siehe unten),
damit UI-Arbeit und Kern-Arbeit unabhängig voneinander laufen können.

## Struktur

```
index.html              Einstiegspunkt, enthält die CSP (siehe THREAT_MODEL.md Abschnitt 5)
src/
  main.ts               Mountet App.svelte
  app.css               Globales Plain-CSS mit CSS-Variablen, hell/dunkel über prefers-color-scheme
  App.svelte            Kopfzeile, Navigation, Hash-Router-Auswahl, globale Schnellerfassung
  lib/
    router.svelte.ts    Minimaler Hash-Router ("#/", "#/projekt/:id", "#/offen", "#/suche?q=")
    brief.ts            Reine Funktionen: "Wo war ich"-Brief, Auffälligkeit (mit Unit-Tests)
    brief.test.ts
    format.ts            Datum/Alter-Formatierung, deutsche Label-Tabellen für Enums
    components/
      NoteText.svelte              Notiztext als Klartext mit Absätzen – kein Markdown, kein {@html}
      AuffaelligkeitPunkt.svelte   Ampel-Punkt ruhig/auffällig/überfällig
      ProjektKarte.svelte          Karte für den Hafen
      Schnellerfassung.svelte      Strg/Cmd+K – ein Feld, Enter speichert
    data/
      types.ts           Datenmodell 1:1 aus CONCEPT.md Abschnitt 3 (deutsche Feldnamen)
      provider.ts         DataProvider-Schnittstelle – die Grenze zum künftigen WASM-Kern
      mock.ts             In-Memory-Implementierung mit Beispieldaten (siehe unten)
      store.ts            Einzige Provider-Instanz der App
      version.svelte.ts   Reaktiver Zähler, der Bildschirme nach Mutationen neu laden lässt
  routes/
    Hafen.svelte          Startseite: Heute wichtig, Hafeneinfahrt, Auf See/Vor Anker/Ideen
    ProjektSeite.svelte   Brief, Kurs, Status mit Übergabe-Pflicht, Fäden, Referenzen, Zugänge, Logbuch
    OffenePunkte.svelte   Alle offenen Fäden projektübergreifend
    Suche.svelte          Volltextsuche über Projekte/Logbuch/Referenzen/Tresor-Titel
```

## Der Mock-Provider

`src/lib/data/provider.ts` definiert `DataProvider` – alles, was die UI über Projekte,
Notizen, Referenzen, Zugänge, Suche und die Hafeneinfahrt wissen muss. `src/lib/data/mock.ts`
implementiert das In-Memory, mit acht Beispielprojekten (eine je Start-Vorlage aus
CONCEPT.md Abschnitt 3), dazu einem "Postkorb"-Projekt für die Schnellerfassung ohne
erkanntes Zielprojekt, und drei Hafeneinfahrt-Kandidaten.

**Das hier ist bewusst kein echtes Backend:** keine Persistenz (ein Reload setzt alles
zurück), keine echte Kryptografie (Tresor-Werte sind feste Platzhalter-Strings, keine
Ciphertexte), keine Websocket-/Sync-Anbindung. Sobald `lotse-core` als WebAssembly
kompiliert vorliegt, ersetzt eine neue Implementierung von `DataProvider`
(`src/lib/data/mock.ts` → z. B. `src/lib/data/wasm.ts`) den Mock in `store.ts`.
Die UI-Komponenten ändern sich dafür nicht, weil sie ausschließlich gegen die
`DataProvider`-Schnittstelle programmieren.

## Sicherheits-/Design-Vorgaben, die dieses Skelett schon einhält

- Kein UI-Framework, kein Tailwind, keine Icon-Pakete, keine Fremdschriften: plain CSS
  mit `prefers-color-scheme` für hell/dunkel.
- Notiztext wird als Klartext mit Absatz-/Zeilenumbrüchen dargestellt, nie über
  `{@html}` – es gibt (noch) keinen sanitisierenden Markdown-Renderer.
- `index.html` setzt eine `Content-Security-Policy` ohne Fremdskripte/CDNs/Analytics,
  `connect-src` ist vorerst auf `'self'` beschränkt.
- Metaphern-Vokabular bleibt auf die fünf Begriffe aus CONCEPT.md Abschnitt 1 beschränkt
  (Hafen, Kurs, Logbuch, vor Anker, Hafeneinfahrt); alles andere heißt schlicht, wie es
  heißt (Suche, Einstellungen, Tresor/Zugänge).

## Scripts

```bash
npm install
npm run dev      # Vite-Dev-Server
npm run build    # Produktions-Build nach dist/
npm run check    # svelte-check (Typen/Templates)
npm test         # vitest (Unit-Tests für brief.ts, jsdom/happy-dom-Umgebung)
```
