// Prüft die WASM-Anbindung in einem echten Browser. Das ist das Abschlusskriterium der
// Phase W1 aus docs/WEB_CLIENT.md: derselbe Vektor, dieselbe Schlüsselableitung, einmal
// nativ in Rust (crates/lotse-core/tests/kdf_vektoren.rs) und einmal hier.
//
// Ohne Fremdabhängigkeit: ein kleiner http-Server aus der Node-Standardbibliothek, ein
// Chromium im Kopflosmodus, und das Ergebnis kommt als POST zurück. Absichtlich kein
// Playwright – ein Browsertreiber als Abhängigkeit wäre mehr Gewicht als der Test wert,
// und ein Browser liegt in CI ohnehin bereit.
//
// Aufruf: node scripts/wasm-browsertest.mjs   (vorher scripts/wasm-bauen.sh)

import { createServer } from 'node:http'
import { readFile } from 'node:fs/promises'
import { spawn } from 'node:child_process'
import { existsSync, mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const hier = dirname(fileURLToPath(import.meta.url))
const web = resolve(hier, '..')
const wurzel = resolve(web, '../..')
const wasmDir = join(web, 'src/lib/wasm')
const vektorDatei = join(wurzel, 'tests/vektoren/kdf.json')

if (!existsSync(join(wasmDir, 'lotse_wasm.js'))) {
  console.error('FEHLER: Die WASM-Anbindung fehlt. Erst scripts/wasm-bauen.sh laufen lassen.')
  process.exit(1)
}

// Ein Browser, der schon da ist. In dieser Reihenfolge, weil der erste Treffer der
// vorhersehbarste ist.
function browserFinden() {
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

const browser = browserFinden()
if (!browser) {
  console.error(
    'FEHLER: Kein Chromium oder Chrome gefunden. Pfad über LOTSE_BROWSER oder CHROME_BIN setzen.',
  )
  process.exit(1)
}

const typen = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
  '.json': 'application/json; charset=utf-8',
}

const seite = await readFile(join(hier, 'wasm-browsertest.html'), 'utf8')

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
      fertig({ fehler: `Antwort war kein JSON: ${e.message}`, faelle: [] })
    }
    return
  }
  if (pfad === '/' || pfad === '/index.html') {
    res.writeHead(200, { 'content-type': typen['.html'] }).end(seite)
    return
  }
  if (pfad === '/vektoren.json') {
    res.writeHead(200, { 'content-type': typen['.json'] }).end(await readFile(vektorDatei))
    return
  }
  if (pfad.startsWith('/wasm/')) {
    const datei = join(wasmDir, pfad.slice('/wasm/'.length))
    if (!datei.startsWith(wasmDir)) {
      res.writeHead(403).end()
      return
    }
    try {
      const inhalt = await readFile(datei)
      const endung = datei.slice(datei.lastIndexOf('.'))
      res.writeHead(200, { 'content-type': typen[endung] ?? 'application/octet-stream' }).end(inhalt)
    } catch {
      res.writeHead(404).end()
    }
    return
  }
  res.writeHead(404).end()
})

await new Promise((r) => server.listen(0, '127.0.0.1', r))
const adresse = `http://127.0.0.1:${server.address().port}/`

const profil = mkdtempSync(join(tmpdir(), 'lotse-browsertest-'))
const kind = spawn(
  browser,
  [
    '--headless=new',
    '--no-sandbox',
    '--disable-gpu',
    '--disable-dev-shm-usage',
    `--user-data-dir=${profil}`,
    adresse,
  ],
  { stdio: ['ignore', 'pipe', 'pipe'] },
)
let browserAusgabe = ''
kind.stdout.on('data', (d) => (browserAusgabe += d))
kind.stderr.on('data', (d) => (browserAusgabe += d))

// Argon2id mit 64 MiB dauert im Browser Sekunden – die Grenze ist großzügig, aber
// endlich: ein hängender Browser darf CI nicht blockieren.
const zeitlimit = Number(process.env.LOTSE_BROWSERTEST_TIMEOUT_MS ?? 180_000)
const abbruch = new Promise((r) => setTimeout(() => r(null), zeitlimit))

const antwort = await Promise.race([ergebnis, abbruch])

kind.kill('SIGKILL')
server.close()
rmSync(profil, { recursive: true, force: true })

if (!antwort) {
  console.error(`FEHLER: Der Browser hat innerhalb von ${zeitlimit} ms nichts gemeldet.`)
  if (browserAusgabe.trim()) console.error(browserAusgabe.trim())
  process.exit(1)
}

for (const f of antwort.faelle ?? []) {
  console.log(`${f.ok ? 'ok  ' : 'FEHL'} ${f.name}${f.ok ? '' : ` – ${f.grund}`}`)
}
if (antwort.fehler) {
  console.error(`FEHLER im Browser: ${antwort.fehler}`)
  process.exit(1)
}
const fehl = (antwort.faelle ?? []).filter((f) => !f.ok).length
const gesamt = (antwort.faelle ?? []).length
if (gesamt === 0) {
  console.error('FEHLER: Der Browser hat keinen einzigen Fall gemeldet.')
  process.exit(1)
}
console.log(
  `${gesamt} Prüfungen im Browser (${antwort.browser ?? 'unbekannt'}), ${fehl} davon fehlgeschlagen.`,
)
process.exit(fehl === 0 ? 0 : 1)
