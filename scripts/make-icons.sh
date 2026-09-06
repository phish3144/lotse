#!/usr/bin/env bash
# Erzeugt die Tauri-Icons aus icons/icon.svg. Braucht ein headless Chromium und Python 3.
set -euo pipefail
# Die Headless-Shell nimmt --window-size als exakte Viewport-Größe; der neue Headless-Modus zieht Fensterrahmen ab.
CH="${CHROME:-/opt/pw-browsers/chromium_headless_shell-1194/chrome-linux/headless_shell}"
DIR="$(cd "$(dirname "$0")/.." && pwd)/apps/desktop/src-tauri/icons"
TMP="$(mktemp -d)"
for s in 16 32 48 64 128 256 512 1024; do
  { printf '<!doctype html><meta charset="utf-8"><style>html,body{margin:0;background:transparent;overflow:hidden}svg{display:block;width:%spx;height:%spx}</style>' "$s" "$s"; cat "$DIR/icon.svg"; } > "$TMP/i.html"
  "$CH" --headless --no-sandbox --disable-gpu --hide-scrollbars --default-background-color=00000000 --virtual-time-budget=1500 \
    --window-size=$s,$s --screenshot="$TMP/$s.png" "file://$TMP/i.html" >/dev/null 2>&1
done
cp "$TMP/32.png" "$DIR/32x32.png"; cp "$TMP/128.png" "$DIR/128x128.png"; cp "$TMP/256.png" "$DIR/128x128@2x.png"; cp "$TMP/512.png" "$DIR/icon.png"
python3 "$(dirname "$0")/pack-icons.py" "$TMP" "$DIR"
rm -rf "$TMP"
ls -la "$DIR"
