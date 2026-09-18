// Prüft die WASM-Anbindung in einem echten Browser. Das ist das Abschlusskriterium der
// Phase W1 aus docs/WEB_CLIENT.md: derselbe Vektor, dieselbe Schlüsselableitung, einmal
// nativ in Rust (crates/lotse-core/tests/kdf_vektoren.rs) und einmal hier.
//
// Aufruf: node scripts/wasm-browsertest.mjs   (vorher scripts/wasm-bauen.sh)

import { imBrowser, wurzel } from './browserlauf.mjs'
import { join } from 'node:path'

let antwort
try {
  antwort = await imBrowser({
    seite: 'wasm-browsertest.html',
    zusatz: { '/vektoren.json': join(wurzel, 'tests/vektoren/kdf.json') },
  })
} catch (e) {
  console.error(`FEHLER: ${e.message}`)
  process.exit(1)
}

for (const f of antwort.faelle ?? []) {
  console.log(`${f.ok ? 'ok  ' : 'FEHL'} ${f.name}${f.ok ? '' : ` – ${f.grund}`}`)
}
if (antwort.fehler) {
  console.error(`FEHLER im Browser: ${antwort.fehler}`)
  process.exit(1)
}
const faelle = antwort.faelle ?? []
if (faelle.length === 0) {
  console.error('FEHLER: Der Browser hat keinen einzigen Fall gemeldet.')
  process.exit(1)
}
const fehl = faelle.filter((f) => !f.ok).length
console.log(`${faelle.length} Prüfungen im Browser (${antwort.browser}), ${fehl} davon fehlgeschlagen.`)
process.exit(fehl === 0 ? 0 : 1)
