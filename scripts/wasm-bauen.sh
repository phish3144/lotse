#!/usr/bin/env bash
# Baut die WASM-Anbindung (crates/lotse-wasm) und legt das Ergebnis nach
# apps/web/src/lib/wasm/ – von dort nimmt es Vite auf.
#
# Braucht `wasm-bindgen` in der Fassung, die auch im Cargo.lock steht. Passen die
# Fassungen nicht, meldet wasm-bindgen das selbst und bricht ab; genau deshalb wird die
# Fassung hier aus dem Lock gelesen und nicht geraten.
set -euo pipefail

wurzel="$(cd "$(dirname "$0")/.." && pwd)"
cd "$wurzel"

ziel="apps/web/src/lib/wasm"
profil="${1:-release}"

erwartet="$(
  awk '/^name = "wasm-bindgen"$/ { gefunden = 1; next }
       gefunden && /^version = / { gsub(/[",]/, "", $3); print $3; exit }' Cargo.lock
)"
if [ -z "$erwartet" ]; then
  echo "FEHLER: wasm-bindgen steht nicht im Cargo.lock." >&2
  exit 1
fi

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "FEHLER: wasm-bindgen fehlt. Erwartet wird Fassung $erwartet." >&2
  echo "  cargo install wasm-bindgen-cli --version $erwartet" >&2
  echo "  oder das fertige Binary aus dem Release von wasm-bindgen." >&2
  exit 1
fi
hat="$(wasm-bindgen --version | awk '{print $2}')"
if [ "$hat" != "$erwartet" ]; then
  echo "FEHLER: wasm-bindgen $hat, erwartet $erwartet (aus Cargo.lock)." >&2
  echo "Verschiedene Fassungen erzeugen Anbindungen, die zur Laufzeit nicht passen." >&2
  exit 1
fi

if [ "$profil" = "debug" ]; then
  cargo build -p lotse-wasm --target wasm32-unknown-unknown
  roh="target/wasm32-unknown-unknown/debug/lotse_wasm.wasm"
else
  cargo build -p lotse-wasm --target wasm32-unknown-unknown --release
  roh="target/wasm32-unknown-unknown/release/lotse_wasm.wasm"
fi

rm -rf "$ziel"
mkdir -p "$ziel"
wasm-bindgen --target web --typescript --out-dir "$ziel" "$roh"


groesse="$(du -h "$ziel/lotse_wasm_bg.wasm" | cut -f1)"
echo "WASM gebaut ($profil, $groesse): $ziel"
