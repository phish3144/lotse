// Was hält ein Browser aus? Misst an einem selbst erzeugten Bestand, nicht an einem
// echten: dieselbe Versiegelung, beliebig viele Datensätze, wiederholbar.
//
//   node scripts/wasm-mengentest.mjs            # 20 000 Notizen
//   node scripts/wasm-mengentest.mjs 200000     # der Zehn-Jahre-Fall
//
// Die Zahlen tragen die Entscheidungen in docs/WEB_CLIENT.md Abschnitt 4a. Deshalb steht
// hier auch eine harte Grenze: reißt der Umschlag-Aufschlag aus, ist eine Annahme des
// Plans falsch geworden und das soll auffallen.

import { imBrowser, wurzel } from './browserlauf.mjs'
import { join } from 'node:path'

const n = Number(process.argv[2] ?? process.env.LOTSE_MENGE ?? 20000)
if (!Number.isFinite(n) || n < 1) {
  console.error(`FEHLER: »${process.argv[2]}« ist keine Anzahl.`)
  process.exit(1)
}

let antwort
try {
  antwort = await imBrowser({
    seite: 'wasm-mengentest.html',
    argumente: { n: String(n) },
    zusatz: { '/vektoren.json': join(wurzel, 'tests/vektoren/kdf.json') },
    // Der Heap-Wert ist ohne dieses Flag gerundet und damit für große Bestände unbrauchbar.
    // LOTSE_HEAP_MB simuliert ein knapperes Budget, wie es ein Telefon hat: ein Desktop
    // mit 4 GB Heap beweist nichts über das Gerät, auf dem es eng wird.
    flaggen: [
      '--enable-precise-memory-info',
      ...(process.env.LOTSE_HEAP_MB
        ? [`--js-flags=--max-old-space-size=${Number(process.env.LOTSE_HEAP_MB)}`]
        : []),
    ],
    zeitlimit: Number(process.env.LOTSE_BROWSERTEST_TIMEOUT_MS ?? 600_000),
  })
} catch (e) {
  console.error(`FEHLER: ${e.message}`)
  if (process.env.LOTSE_HEAP_MB) {
    console.error(
      `Bei ${process.env.LOTSE_HEAP_MB} MB Heap-Budget und ${n} Sätzen ist der Browser ` +
        'ausgestiegen. Das ist ein Messergebnis, keine Panne: genau diese Grenze trägt die ' +
        'Entscheidung in docs/WEB_CLIENT.md 4a.',
    )
  }
  process.exit(1)
}

if (antwort.fehler) {
  console.error(`FEHLER im Browser: ${antwort.fehler}`)
  process.exit(1)
}
const w = antwort.werte ?? {}
if (!w.n) {
  console.error('FEHLER: Der Browser hat keine Zahlen gemeldet.')
  process.exit(1)
}

// Ein abgebrochener Lauf kann Zahlen halb geliefert haben. Dann soll hier »–« stehen und
// nicht eine Ausnahme fliegen, die wie ein Fehler des Tests aussieht.
const z = (v) => (typeof v === 'number' ? v.toLocaleString('de-DE') : '–')
const mb = (bytes) => (typeof bytes === 'number' ? (bytes / 1048576).toFixed(1) : '–')
console.log(`Bestand: ${z(w.n)} Notizen (${antwort.browser})`)
console.log(`  versiegeln        ${z(w.siegeln_ms)} ms`)
console.log(`  öffnen            ${z(w.oeffnen_ms)} ms  (${z(w.oeffnen_pro_sekunde)}/s)`)
console.log(`  Index bauen       ${z(w.index_ms)} ms`)
console.log(`  Suche linear      ${z(w.suche_ms)} ms  (${z(w.suche_treffer)} Treffer)`)
console.log(`  Umschlag          ${z(w.umschlag_bytes_mittel)} B je Satz, ${mb(w.umschlag_bytes_gesamt)} MB gesamt`)
console.log(`  Klartext          ${z(w.klartext_bytes_je_satz)} B je Satz`)
console.log(`  schlanker Index   ${z(w.index_bytes_je_satz)} B je Satz`)
if (w.heap_mb) console.log(`  JS-Heap           ${z(w.heap_mb)} MB von ${z(w.heap_grenze_mb)} MB`)

// Der Umschlag ist Nonce + Ciphertext + Poly1305-Tag in Base64 plus die Kopffelder. Der
// Aufschlag ist damit begrenzt – aber die Grenze gilt für die Notizgröße dieses Tests
// (~200 B Klartext), weil die Kopffelder konstant sind und bei kleinen Sätzen relativ
// schwer wiegen. Läuft der Wert weg, hat sich an der Versiegelung etwas geändert, das
// docs/WEB_CLIENT.md 4a nicht kennt.
const aufschlag = w.umschlag_bytes_mittel / w.klartext_bytes_je_satz
if (!Number.isFinite(aufschlag)) {
  console.error('FEHLER: Der Umschlag-Aufschlag ließ sich nicht berechnen.')
  process.exit(1)
}
if (aufschlag > 3) {
  console.error(
    `FEHLER: Ein Umschlag ist ${aufschlag.toFixed(1)}× so groß wie sein Klartext ` +
      `(${w.klartext_bytes_je_satz} B). Grenze ist 3× für Sätze dieser Größe.`,
  )
  process.exit(1)
}
console.log(`  Aufschlag         ${aufschlag.toFixed(2)}× (Grenze 3× bei ~200 B Klartext)`)
