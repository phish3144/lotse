#!/usr/bin/env bash
# Prüft, ob docs/wiki/ noch zur Anwendung passt. Siehe scripts/wiki_pruefen.py.
set -euo pipefail
exec python3 "$(dirname "$0")/wiki_pruefen.py" "$@"
