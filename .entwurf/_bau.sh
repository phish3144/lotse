#!/usr/bin/env bash
# Setzt Rumpf + Seiten-CSS + Basis-CSS zu einem .dc.html zusammen.
# Logik und data-props kommen aus _<name>.props / _<name>.logic.js, falls vorhanden.
set -euo pipefail
name="$1"
{
  echo '<!doctype html>'
  echo '<html>'
  echo '<head>'
  echo '  <meta charset="utf-8">'
  echo '  <script src="./support.js"></script>'
  echo '</head>'
  echo '<body>'
  echo '<x-dc>'
  echo '<helmet>'
  echo '  <style>'
  cat _basis.css
  [ -f "_${name}.css" ] && cat "_${name}.css"
  echo '  </style>'
  echo '</helmet>'
  cat "_${name}.body.html"
  echo '</x-dc>'
  if [ -f "_${name}.props" ]; then
    printf '<script data-dc-script data-props='"'"'%s'"'"'>\n' "$(cat "_${name}.props")"
    cat "_${name}.logic.js"
    echo '</script>'
  fi
  echo '</body>'
  echo '</html>'
} > "${name}.dc.html"
echo "${name}.dc.html: $(wc -l < "${name}.dc.html") Zeilen"
