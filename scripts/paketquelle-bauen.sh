#!/usr/bin/env bash
# Baut aus den Paketen einer Veröffentlichung eine APT- und eine RPM-Quelle.
#
# Warum überhaupt: auf Linux ist »automatisch aktualisieren« gleichbedeutend mit »die
# Paketverwaltung tut es«. Ein eingebauter Updater, der eine Datei herunterlädt, die man
# dann anklickt, ist Handarbeit mit Fortschrittsbalken. Mit einer Quelle aktualisiert
# `apt upgrade` Lotse zusammen mit allem anderen, und der Updater wird dort überflüssig.
#
# In der Quelle steht immer nur die **neueste** Fassung. Für das Aktualisieren genügt das;
# eine ältere Fassung gezielt zu installieren geht darüber nicht. Das ist der Preis dafür,
# dass nichts Binäres im Repository landet: die Dateien liegen nur im ausgelieferten
# Seiten-Bündel, nicht in der Git-Geschichte.
#
#   scripts/paketquelle-bauen.sh <pakete> <ausgabe> <basis-url>
#
# Signiert wird, wenn GPG_KEY_ID gesetzt ist. Ohne Signatur entsteht eine Quelle, die apt
# ablehnt – das ist zum Ausprobieren gedacht und wird deutlich gesagt.
set -euo pipefail

pakete="${1:?Verzeichnis mit .deb und .rpm}"
aus="${2:?Ausgabeverzeichnis}"
basis="${3:?Basis-URL, z. B. https://lotse.sanctora.eu}"
schluessel="${GPG_KEY_ID:-}"

# Signieren immer über dieselbe Aufrufform. Eine Passphrase gehört auf die Kommandozeile
# mit `--pinentry-mode loopback`; in die gpg.conf geschrieben tut sie nichts, und man
# sucht den Fehler dann bei der Signatur statt bei der Einrichtung.
gpg_signieren() {
  local args=(--batch --yes --local-user "$schluessel")
  if [ -n "${GPG_PASSPHRASE:-}" ]; then
    args+=(--pinentry-mode loopback --passphrase "$GPG_PASSPHRASE")
  fi
  gpg "${args[@]}" "$@"
}

deb="$(find "$pakete" -maxdepth 2 -name '*.deb' | head -1)"
rpm="$(find "$pakete" -maxdepth 2 -name '*.rpm' | head -1)"

meld() { printf '%s\n' "$*" >&2; }

# ------------------------------------------------------------------------- APT
if [ -n "$deb" ]; then
  meld "APT-Quelle aus $(basename "$deb")"
  mkdir -p "$aus/apt/pool/main/l/lotse" "$aus/apt/dists/stable/main/binary-amd64"
  cp "$deb" "$aus/apt/pool/main/l/lotse/"

  # `Filename` muss relativ zur Wurzel der Quelle stehen – deshalb aus $aus/apt heraus.
  ( cd "$aus/apt" && dpkg-scanpackages --multiversion pool /dev/null ) \
    > "$aus/apt/dists/stable/main/binary-amd64/Packages"
  gzip -9nc "$aus/apt/dists/stable/main/binary-amd64/Packages" \
    > "$aus/apt/dists/stable/main/binary-amd64/Packages.gz"

  # Das Release-Verzeichnis von Hand: apt-ftparchive wäre eine Abhängigkeit mehr für
  # eine Datei mit acht Feldern und zwei Prüfsummen.
  {
    echo "Origin: Lotse"
    echo "Label: Lotse"
    echo "Suite: stable"
    echo "Codename: stable"
    echo "Architectures: amd64"
    echo "Components: main"
    echo "Description: Lotse – das Logbuch für alle deine Vorhaben"
    echo "Date: $(LC_ALL=C date -u '+%a, %d %b %Y %H:%M:%S UTC')"
    echo "SHA256:"
    ( cd "$aus/apt/dists/stable" && \
      find main -type f | LC_ALL=C sort | while read -r f; do
        printf ' %s %16d %s\n' "$(sha256sum "$f" | cut -d' ' -f1)" "$(stat -c%s "$f")" "$f"
      done )
  } > "$aus/apt/dists/stable/Release"

  if [ -n "$schluessel" ]; then
    gpg_signieren --armor --detach-sign \
      -o "$aus/apt/dists/stable/Release.gpg" "$aus/apt/dists/stable/Release"
    gpg_signieren --clearsign \
      -o "$aus/apt/dists/stable/InRelease" "$aus/apt/dists/stable/Release"
  else
    meld "WARNUNG: ohne GPG_KEY_ID entsteht eine unsignierte Quelle. apt lehnt sie ab."
  fi
else
  meld "Kein .deb gefunden – APT-Quelle übersprungen."
fi

# ------------------------------------------------------------------------- RPM
if [ -n "$rpm" ]; then
  if command -v createrepo_c >/dev/null 2>&1; then
    meld "RPM-Quelle aus $(basename "$rpm")"
    mkdir -p "$aus/rpm"
    cp "$rpm" "$aus/rpm/"
    createrepo_c --quiet "$aus/rpm"
    if [ -n "$schluessel" ]; then
      gpg_signieren --armor --detach-sign \
        -o "$aus/rpm/repodata/repomd.xml.asc" "$aus/rpm/repodata/repomd.xml"
    fi
  else
    meld "createrepo_c fehlt – RPM-Quelle übersprungen."
  fi
else
  meld "Kein .rpm gefunden – RPM-Quelle übersprungen."
fi

# ------------------------------------------------------- Einrichtung zum Mitnehmen
if [ -n "$schluessel" ]; then
  gpg --batch --yes --armor --export "$schluessel" > "$aus/lotse-archiv.asc"
fi

# Die Einrichtungsschnipsel entstehen auch dann, wenn ein Paket fehlte – wer sie liest,
# soll sehen, wie es gemeint ist, statt auf eine fehlende Datei zu stoßen.
mkdir -p "$aus/apt" "$aus/rpm"

cat > "$aus/apt/lotse.list" <<EOF
deb [signed-by=/usr/share/keyrings/lotse-archiv.gpg] $basis/apt stable main
EOF

cat > "$aus/rpm/lotse.repo" <<EOF
[lotse]
name=Lotse
baseurl=$basis/rpm
enabled=1
repo_gpgcheck=1
gpgcheck=0
gpgkey=$basis/lotse-archiv.asc
EOF

meld "Fertig: $aus"
