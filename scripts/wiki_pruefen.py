#!/usr/bin/env python3
"""Prüft, ob das Wiki noch zur Anwendung passt.

Der Sinn: »Das Wiki wird bei jedem Update erweitert« ist als Vorsatz wertlos. Diese
Prüfung liest die tatsächliche Oberfläche aus dem Quelltext – Kommandos, Felder,
Zustände, Schalter, Fehler, Umgebungsvariablen – und verlangt, dass jedes Stück davon
im Wiki vorkommt. Wer ein Feld hinzufügt und das Wiki nicht anfasst, bekommt hier eine
rote CI, nicht ein halbes Jahr später eine Frage im Postfach.

Absichtlich stumpf: sie prüft, dass die Zeichenkette irgendwo auf der zuständigen Seite
steht. Ob der Satz drumherum stimmt, kann keine Maschine wissen. Was sie kann, ist das
Vergessen unmöglich machen.

Aufruf: scripts/wiki_pruefen.sh   (oder python3 scripts/wiki_pruefen.py)
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

WURZEL = Path(__file__).resolve().parent.parent
WIKI = WURZEL / "docs" / "wiki"


# --------------------------------------------------------------- Werkzeug

def lies(*teile: str) -> str:
    return (WURZEL.joinpath(*teile)).read_text(encoding="utf-8")


def seite(name: str) -> str:
    p = WIKI / f"{name}.md"
    if not p.exists():
        raise SystemExit(f"Wiki-Seite fehlt: docs/wiki/{name}.md")
    return p.read_text(encoding="utf-8")


def alles() -> str:
    return "\n".join(p.read_text(encoding="utf-8") for p in sorted(WIKI.glob("*.md")))


class Befund:
    def __init__(self) -> None:
        self.fehler: list[str] = []
        self.geprueft = 0

    def verlange(self, nadel: str, heu: str, wo: str, was: str) -> None:
        self.geprueft += 1
        if nadel not in heu:
            self.fehler.append(f"{wo}: {was} »{nadel}« ist nirgends beschrieben.")

    def verlange_alle(self, nadeln, heu: str, wo: str, was: str) -> None:
        for n in sorted(set(nadeln)):
            self.verlange(n, heu, wo, was)


# --------------------------------------------------------------- Ablesen

def cli_kommandos() -> list[str]:
    """Kommandobaum aus den clap-Enums in main.rs.

    Nicht aus `--help`: das verlangte einen Bau, und die Prüfung soll auch ohne
    Rust-Werkzeugkette laufen.
    """
    text = lies("crates", "lotse-cli", "src", "main.rs")
    unter = {
        "Cmd": "",
        "SyncCmd": "sync",
        "ProjektCmd": "projekt",
        "RefCmd": "ref",
        "TresorCmd": "tresor",
        "ExportCmd": "export",
    }
    # Welches Unterkommando trägt welches Enum? `#[command(subcommand)] Sync(SyncCmd)`
    out: list[str] = []
    for enum_name, praefix in unter.items():
        m = re.search(rf"enum {enum_name} \{{(.*?)\n\}}", text, re.S)
        if not m:
            raise SystemExit(f"clap-Enum {enum_name} nicht gefunden – Prüfung veraltet.")
        koerper = m.group(1)
        # Varianten: Zeilenanfang, zwei Leerzeichen, Großbuchstabe.
        for v in re.findall(r"^    ([A-ZÄÖÜ][A-Za-z0-9]*)", koerper, re.M):
            name = kebab(v)
            out.append(f"{praefix} {name}".strip())
    return out


def kebab(variante: str) -> str:
    """clap macht aus `Uebernehmen` → `uebernehmen`, aus `SyncJetzt` → `sync-jetzt`."""
    s = re.sub(r"(?<!^)(?=[A-Z])", "-", variante).lower()
    return s


def modell_werte() -> dict[str, list[str]]:
    text = lies("crates", "lotse-core", "src", "model.rs")
    werte: dict[str, list[str]] = {}
    for m in re.finditer(r'([A-ZÄÖÜ][A-Za-z]*)::([A-Za-z]+) => "([a-z_]+)"', text):
        werte.setdefault(m.group(1), []).append(m.group(3))
    return werte


def struktur_felder() -> dict[str, list[str]]:
    text = lies("crates", "lotse-core", "src", "model.rs")
    out: dict[str, list[str]] = {}
    for m in re.finditer(r"pub struct ([A-ZÄÖÜ][A-Za-z]*) \{(.*?)\n\}", text, re.S):
        name, koerper = m.group(1), m.group(2)
        felder = re.findall(r"^\s+pub ([a-z_0-9]+):", koerper, re.M)
        out[name] = felder
    return out


def tauri_kommandos() -> list[str]:
    text = lies("apps", "desktop", "src-tauri", "src", "lib.rs")
    return re.findall(r"#\[tauri::command\]\s*(?:pub )?(?:async )?fn ([a-z_0-9]+)", text)


def mcp_werkzeuge() -> list[str]:
    text = lies("crates", "lotse-core", "src", "mcp", "mod.rs")
    m = re.search(r"fn werkzeuge\(\).*?\n\}", text, re.S)
    return re.findall(r'"name": "([a-z_]+)"', m.group(0) if m else "")


def umgebungsvariablen() -> list[str]:
    namen: set[str] = set()
    for p in list((WURZEL / "crates").rglob("*.rs")) + list(
        (WURZEL / "apps" / "desktop" / "src-tauri" / "src").rglob("*.rs")
    ):
        namen.update(re.findall(r"\bLOTSE_[A-Z_]+", p.read_text(encoding="utf-8")))
    # Nur für Tests gedacht und nirgends dokumentiert nötig.
    namen.discard("LOTSE_KDF_SCHNELL")
    return sorted(namen)


def meta_schluessel() -> list[str]:
    namen: set[str] = set()
    for p in list((WURZEL / "crates").rglob("*.rs")) + list(
        (WURZEL / "apps" / "desktop" / "src-tauri" / "src").rglob("*.rs")
    ):
        namen.update(
            re.findall(r'META_[A-Z_]+: &str = "([a-z_.]+)"', p.read_text(encoding="utf-8"))
        )
    return sorted(namen)


def fehlervarianten() -> list[str]:
    text = lies("crates", "lotse-core", "src", "error.rs")
    return re.findall(r'#\[error\(\s*\n?\s*"([^"]+)"', text)


def einstellungs_reiter() -> list[str]:
    text = lies("apps", "web", "src", "routes", "Einstellungen.svelte")
    m = re.search(r"const REITER = \[(.*?)\] as const", text, re.S)
    if not m:
        raise SystemExit("REITER in Einstellungen.svelte nicht gefunden – Prüfung veraltet.")
    return re.findall(r"name: '([^']+)'", m.group(1))


def sync_endpunkte() -> list[str]:
    pfade: set[str] = set()
    for p in (WURZEL / "services" / "sync-worker" / "src").rglob("*.ts"):
        pfade.update(re.findall(r'"(/v1/[a-z0-9/:_-]+)"', p.read_text(encoding="utf-8")))
    return sorted(pfade)


def erkennungsmarken() -> list[str]:
    text = lies("crates", "lotse-core", "src", "detect.rs")
    m = re.search(r"pub fn erkenne\(.*?\n\}", text, re.S)
    körper = m.group(0) if m else ""
    marken = set(re.findall(r'"(\.[a-z0-9]+|[a-z][a-z0-9._]*\.[a-z]+|gemfile)"', körper))
    return sorted(marken)


def vorlagen_intervalle() -> dict[str, int]:
    text = lies("crates", "lotse-core", "src", "model.rs")
    m = re.search(r"fn erwartungsintervall_tage\(self\) -> u32 \{(.*?)\n    \}", text, re.S)
    return {v: int(t) for v, t in re.findall(r"Vorlage::([A-Za-z]+) => (\d+)", m.group(1))}


def version() -> str:
    return re.search(r'^version = "([^"]+)"', lies("Cargo.toml"), re.M).group(1)


def verweise() -> list[tuple[str, str]]:
    raus: list[tuple[str, str]] = []
    for p in sorted(WIKI.glob("*.md")):
        for m in re.finditer(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]", p.read_text(encoding="utf-8")):
            raus.append((p.name, (m.group(2) or m.group(1)).strip()))
    return raus


# --------------------------------------------------------------- Prüfen

def main() -> int:
    b = Befund()
    ganz = alles()

    b.verlange_alle(cli_kommandos(), seite("Kommandozeile-Referenz"), "Kommandozeile-Referenz", "Kommando")
    b.verlange_alle(tauri_kommandos(), ganz, "irgendeine Seite", "App-Kommando")
    b.verlange_alle(mcp_werkzeuge(), seite("Assistenten-MCP"), "Assistenten-MCP", "MCP-Werkzeug")
    b.verlange_alle(umgebungsvariablen(), seite("Umgebungsvariablen"), "Umgebungsvariablen", "Variable")
    b.verlange_alle(meta_schluessel(), seite("Datenablage"), "Datenablage", "Einstellungsschlüssel")
    b.verlange_alle(einstellungs_reiter(), seite("Einstellungen"), "Einstellungen", "Reiter")
    b.verlange_alle(sync_endpunkte(), seite("Sync-Protokoll"), "Sync-Protokoll", "Endpunkt")
    b.verlange_alle(erkennungsmarken(), seite("Erkennungsregeln"), "Erkennungsregeln", "Erkennungsmarke")

    datenmodell = seite("Datenmodell")
    for typ, werte in modell_werte().items():
        b.verlange_alle(werte, datenmodell, "Datenmodell", f"{typ}-Wert")
    for strukt, felder in struktur_felder().items():
        b.verlange(strukt, datenmodell, "Datenmodell", "Struktur")
        b.verlange_alle(felder, datenmodell, "Datenmodell", f"{strukt}-Feld")

    fehler_seite = seite("Fehlermeldungen")
    for text in fehlervarianten():
        # Platzhalter heraus: geprüft wird der feste Teil des Satzes.
        fest = max(re.split(r"\{[^}]*\}", text), key=len).strip(" :,.()")
        if len(fest) >= 12:
            b.verlange(fest, fehler_seite, "Fehlermeldungen", "Fehlertext")

    vorlagen = seite("Vorhaben")
    for v, tage in vorlagen_intervalle().items():
        b.verlange(f"{tage} Tage", vorlagen, "Vorhaben", f"Erwartungsintervall von {v}")

    b.verlange(version(), seite("Aenderungen"), "Aenderungen", "laufende Fassung")

    seiten = {p.stem for p in WIKI.glob("*.md")}
    for datei, ziel in verweise():
        b.geprueft += 1
        if ziel not in seiten:
            b.fehler.append(f"{datei}: Verweis [[{ziel}]] zeigt auf keine Seite.")

    print(f"{len(seiten)} Wiki-Seiten, {b.geprueft} Prüfungen.")
    if b.fehler:
        print()
        for f in b.fehler:
            print(f"  ✗ {f}")
        print(f"\n{len(b.fehler)} Lücke(n). Das Wiki hinkt der Anwendung nach –")
        print("ergänze die genannten Stellen in docs/wiki/ und lauf die Prüfung erneut.")
        return 1
    print("Das Wiki deckt die gesamte abgelesene Oberfläche ab.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
