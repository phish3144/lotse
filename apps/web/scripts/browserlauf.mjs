// Eine Seite in einem echten Browser laufen lassen und ihr Ergebnis abholen.
//
// Absichtlich ohne Fremdabhängigkeit: ein http-Server aus der Node-Standardbibliothek,
// ein Chromium im Kopflosmodus, das Ergebnis kommt als POST auf /ergebnis zurück.
// Playwright wäre der bequeme Weg gewesen und hätte einen Browsertreiber samt Baum in den
// Werkzeugbaum geholt – für etwas, das eine Seite lädt und Zahlen zurückgibt.
//
// Benutzt von wasm-browsertest.mjs (Vektoren) und wasm-mengentest.mjs (Menge).

import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { spawn } from 'node:child_process'
import { existsSync, mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const hier = dirname(fileURLToPath(import.meta.url))
export const web = resolve(hier, '..')
export const wurzel = resolve(web, '../..')
export const wasmDir = join(web, 'src/lib/wasm')

/** Ein Browser, der schon da ist. Erster Treffer gewinnt – der ist der vorhersehbarste. */
export function browserFinden() {
  const kandidaten = [
    process.env.LOTSE_BROWSER,
    process.env.CHROME_BIN,
    ...(process.env.PLAYWRIGHT_BROWSERS_PATH
      ? [
          `${process.env.PLAYWRIGHT_BROWSERS_PATH}/chromium-1194/chrome-linux/chrome`,
          `${process.env.PLAYWRIGHT_BROWSERS_PATH}/chromium_headless_shell-1194/chrome-linux/headless_shell`,
        ]
      : []),
    '/usr/bin/google-chrome',
    '/usr/bin/google-chrome-stable',
    '/usr/bin/chromium',
    '/usr/bin/chromium-browser',
    '/opt/google/chrome/chrome',
  ].filter(Boolean)
  for (const k of kandidaten) if (existsSync(k)) return k
  return null
}

const typen = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
  '.json': 'application/json; charset=utf-8',
}

/**
 * Lädt `seite` (Dateiname neben diesem Skript) im Browser und liefert, was sie an
 * /ergebnis geschickt hat. Wirft, wenn die WASM-Anbindung fehlt, kein Browser da ist
 * oder innerhalb des Zeitlimits nichts kommt – Stille ist hier nicht von Erfolg zu
 * unterscheiden.
 *
 * `zusatz` bindet weitere Pfade an Dateien, z. B. { '/vektoren.json': '/abs/pfad' }.
 * `argumente` wird als Query an die Seite gehängt.
 * `flaggen` sind zusätzliche Kommandozeilenflaggen für den Browser.
 */
export async function imBrowser({
  seite,
  zusatz = {},
  argumente = {},
  flaggen = [],
  zeitlimit = Number(process.env.LOTSE_BROWSERTEST_TIMEOUT_MS ?? 180_000),
} = {}) {
  if (!existsSync(join(wasmDir, 'lotse_wasm.js'))) {
    throw new Error('Die WASM-Anbindung fehlt. Erst scripts/wasm-bauen.sh laufen lassen.')
  }
  const browser = browserFinden()
  if (!browser) {
    throw new Error(
      'Kein Chromium oder Chrome gefunden. Pfad über LOTSE_BROWSER oder CHROME_BIN setzen.',
    )
  }

  const inhalt = await readFile(join(hier, seite), 'utf8')
  let fertig
  const ergebnis = new Promise((r) => (fertig = r))

  const server = createServer(async (req, res) => {
    const pfad = new URL(req.url, 'http://127.0.0.1').pathname
    if (req.method === 'POST' && pfad === '/ergebnis') {
      let rumpf = ''
      for await (const teil of req) rumpf += teil
      res.writeHead(204).end()
      try {
        fertig(JSON.parse(rumpf))
      } catch (e) {
        fertig({ fehler: `Antwort war kein JSON: ${e.message}` })
      }
      return
    }
    if (pfad === '/' || pfad === '/index.html') {
      res.writeHead(200, { 'content-type': typen['.html'] }).end(inhalt)
      return
    }
    if (zusatz[pfad]) {
      const datei = zusatz[pfad]
      const endung = datei.slice(datei.lastIndexOf('.'))
      res
        .writeHead(200, { 'content-type': typen[endung] ?? 'application/octet-stream' })
        .end(await readFile(datei))
      return
    }
    if (pfad.startsWith('/wasm/')) {
      const datei = join(wasmDir, pfad.slice('/wasm/'.length))
      if (!datei.startsWith(wasmDir)) {
        res.writeHead(403).end()
        return
      }
      try {
        const roh = await readFile(datei)
        const endung = datei.slice(datei.lastIndexOf('.'))
        res.writeHead(200, { 'content-type': typen[endung] ?? 'application/octet-stream' }).end(roh)
      } catch {
        res.writeHead(404).end()
      }
      return
    }
    res.writeHead(404).end()
  })

  await new Promise((r) => server.listen(0, '127.0.0.1', r))
  const abfrage = new URLSearchParams(argumente).toString()
  const adresse = `http://127.0.0.1:${server.address().port}/${abfrage ? `?${abfrage}` : ''}`

  const profil = mkdtempSync(join(tmpdir(), 'lotse-browserlauf-'))
  const kind = spawn(
    browser,
    [
      '--headless=new',
      '--no-sandbox',
      '--disable-gpu',
      '--disable-dev-shm-usage',
      `--user-data-dir=${profil}`,
      ...flaggen,
      adresse,
    ],
    { stdio: ['ignore', 'pipe', 'pipe'] },
  )
  let ausgabe = ''
  kind.stdout.on('data', (d) => (ausgabe += d))
  kind.stderr.on('data', (d) => (ausgabe += d))

  // Der Wecker muss wieder abgestellt werden. Ohne das clearTimeout hält ein offener Timer
  // die Ereignisschleife am Leben: der Lauf ist längst fertig und sieht trotzdem hängend
  // aus. Genau das ist beim ersten Mengentest passiert.
  let wecker
  const abbruch = new Promise((r) => (wecker = setTimeout(() => r(null), zeitlimit)))
  const antwort = await Promise.race([ergebnis, abbruch])
  clearTimeout(wecker)

  kind.kill('SIGKILL')
  server.close()
  rmSync(profil, { recursive: true, force: true })

  if (!antwort) {
    throw new Error(
      `Der Browser hat innerhalb von ${zeitlimit} ms nichts gemeldet.${
        ausgabe.trim() ? `\n${ausgabe.trim()}` : ''
      }`,
    )
  }
  return { ...antwort, browser: antwort.browser ?? 'unbekannt' }
}
