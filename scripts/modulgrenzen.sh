#!/usr/bin/env bash
# Prüft die Regel aus THREAT_MODEL.md Abschnitt 6 und CLAUDE.md: Diese Module dürfen
# keinen Weg zum Tresor haben. Der Tresor gehört der Oberfläche, der CLI und dem
# Bundle-Export – sonst niemandem.
#
# Läuft in CI und lokal: scripts/modulgrenzen.sh
set -euo pipefail

wurzel="$(cd "$(dirname "$0")/.." && pwd)"
kern="$wurzel/crates/lotse-core/src"

module=(ai.rs detect.rs dokument.rs forge.rs git.rs kalender.rs mcp.rs update.rs watcher.rs export/spiegel.rs)

fehler=0
for m in "${module[@]}"; do
  pfad="$kern/$m"
  [ -f "$pfad" ] || continue
  # `crate::vault`, `use ...vault::` oder `super::vault` – jede Form zählt.
  if grep -nE '(crate|super)::vault|use[[:space:]]+.*\bvault::' "$pfad"; then
    echo "FEHLER: $m greift auf den Tresor zu (THREAT_MODEL.md, Abschnitt 6)." >&2
    fehler=1
  fi
done

if [ "$fehler" -ne 0 ]; then
  exit 1
fi
echo "Modulgrenzen eingehalten: ${#module[@]} Module ohne Tresor-Zugriff."
