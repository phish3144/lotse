#!/usr/bin/env bash
# Prüft die Regel aus THREAT_MODEL.md Abschnitt 6 und CLAUDE.md: Diese Module dürfen
# keinen Weg zum Tresor haben. Der Tresor gehört der Oberfläche, der CLI und dem
# Bundle-Export – sonst niemandem.
#
# Läuft in CI und lokal: scripts/modulgrenzen.sh
set -euo pipefail

wurzel="$(cd "$(dirname "$0")/.." && pwd)"
kern="$wurzel/crates/lotse-core/src"

# Eigenständige Module. Namen, keine Dateinamen: ein Modul kann `x.rs` sein oder ein
# Ordner `x/` mit mehreren Dateien. Beides wird geprüft, sonst rutscht eine Regel beim
# Aufteilen eines Moduls unbemerkt durch.
module=(ai detect dokument forge git kalender mcp netz update watcher)

# Module, die in einer größeren Datei stehen, als `Datei:Modulname`. `export::spiegel`
# darf den Tresor nicht sehen, `export::bundle` in derselben Datei schon – deshalb wird
# hier nur der Block des genannten Moduls geprüft.
eingebettet=("export.rs:spiegel")

# `crate::vault`, `use ...vault::` oder `super::vault` – jede Form zählt.
muster='(crate|super)::vault|use[[:space:]]+.*\bvault::'

fehler=0
gezaehlt=0

# Liest den Rumpf eines eingebetteten Moduls: ab `pub mod <name> {` bis die Klammer
# wieder zugeht. Ohne das würde die ganze Datei geprüft und `bundle` falsch anschlagen.
block() {
  awk -v modul="$2" '
    !drin && $0 ~ "^[[:space:]]*(pub )?mod[[:space:]]+" modul "[[:space:]]*\\{" { drin = 1 }
    drin {
      print
      n = gsub(/\{/, "{"); tiefe += n
      n = gsub(/\}/, "}"); tiefe -= n
      if (tiefe <= 0) exit
    }
  ' "$1"
}

pruefe() { # $1 = Beschriftung, Text auf stdin
  if grep -nE "$muster"; then
    echo "FEHLER: $1 greift auf den Tresor zu (THREAT_MODEL.md, Abschnitt 6)." >&2
    return 1
  fi
  return 0
}

for m in "${module[@]}"; do
  dateien=()
  [ -f "$kern/$m.rs" ] && dateien+=("$kern/$m.rs")
  if [ -d "$kern/$m" ]; then
    while IFS= read -r d; do dateien+=("$d"); done < <(find "$kern/$m" -name '*.rs' | sort)
  fi
  # Ein Modul aus der Liste, das es nicht gibt, ist ein Fehler in dieser Liste – kein
  # Grund, still weiterzugehen. Genau das hat das Aufteilen von `mcp` verdeckt.
  if [ "${#dateien[@]}" -eq 0 ]; then
    echo "FEHLER: Modul $m gibt es nicht mehr. Liste in $0 nachziehen." >&2
    fehler=1
    continue
  fi
  for pfad in "${dateien[@]}"; do
    gezaehlt=$((gezaehlt + 1))
    pruefe "${pfad#"$kern/"}" < "$pfad" || fehler=1
  done
done

for e in "${eingebettet[@]}"; do
  datei="$kern/${e%%:*}"
  name="${e##*:}"
  if [ ! -f "$datei" ]; then
    echo "FEHLER: $datei gibt es nicht mehr. Liste in $0 nachziehen." >&2
    fehler=1
    continue
  fi
  rumpf="$(block "$datei" "$name")"
  if [ -z "$rumpf" ]; then
    echo "FEHLER: Modul $name steht nicht mehr in ${e%%:*}. Liste in $0 nachziehen." >&2
    fehler=1
    continue
  fi
  gezaehlt=$((gezaehlt + 1))
  printf '%s\n' "$rumpf" | pruefe "${e%%:*} :: $name" || fehler=1
done

if [ "$fehler" -ne 0 ]; then
  exit 1
fi
echo "Modulgrenzen eingehalten: $gezaehlt Dateien und Blöcke ohne Tresor-Zugriff."
