#!/usr/bin/env bash
# Startet den Sync-Dienst lokal (wrangler dev) und lässt den End-to-End-Test des Kerns
# dagegen laufen. Braucht Node, die Abhängigkeiten in services/sync-worker und Rust.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PORT="${PORT:-8787}"
LOG="$(mktemp)"

cd "$ROOT/services/sync-worker"
npx wrangler d1 migrations apply lotse --local >/dev/null
npx wrangler dev --port "$PORT" --local >"$LOG" 2>&1 &
WRANGLER=$!
trap 'kill $WRANGLER 2>/dev/null || true; rm -f "$LOG"' EXIT

for _ in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:$PORT/v1/auth/prelogin?email=x" >/dev/null 2>&1 \
     || curl -sS -o /dev/null -w '%{http_code}' "http://127.0.0.1:$PORT/v1/auth/prelogin?email=x" 2>/dev/null | grep -q '^4'; then
    break
  fi
  sleep 1
done

cd "$ROOT"
LOTSE_SYNC_URL="http://127.0.0.1:$PORT" cargo test -p lotse-core --test sync_e2e -- --nocapture
