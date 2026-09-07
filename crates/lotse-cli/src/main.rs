//! `lotse` – Kommandozeile.
//!
//! Schnellerfassung aus dem Terminal, Projektverwaltung, Tresor, Erkennung, Export.
//! Teilt sich Datenordner und Gerät mit der Desktop-App.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use lotse_core::brief;
use lotse_core::crypto::{self, KdfParams, Key32, RecoveryCode};
use lotse_core::detect;
use lotse_core::export;
use lotse_core::konto;
use lotse_core::model::*;
use lotse_core::now_ms;
use lotse_core::store::Store;
use lotse_core::sync::client::{self as sync_client, Client, Geraet as SyncGeraet};
use lotse_core::vault::{TresorEintrag, VaultKeys};
use lotse_core::watcher::{self, Beobachter};
use ulid::Ulid;
use zeroize::Zeroizing;

#[derive(Parser)]
#[command(
    name = "lotse",
    version,
    about = "Lotse – das Logbuch für alle deine Vorhaben"
)]
struct Cli {
    /// Datenordner (Standard: $LOTSE_HOME oder das Nutzer-Datenverzeichnis)
    #[arg(long, global = true, env = "LOTSE_HOME")]
    home: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Konto und Datenbank auf diesem Gerät einrichten
    Init {
        /// Name dieses Geräts
        #[arg(long, default_value = "Dieser Rechner")]
        geraet: String,
        /// Bestätigung des Wiederherstellungscodes überspringen (nur für Skripte;
        /// alternativ Umgebungsvariable LOTSE_SKIP_CONFIRM)
        #[arg(long)]
        ohne_bestaetigung: bool,
        /// Sofort beim Sync-Dienst registrieren (zusammen mit --email)
        #[arg(long, requires = "email")]
        sync_url: Option<String>,
        #[arg(long)]
        email: Option<String>,
    },
    /// Startseite: alle Projekte mit Auffälligkeit
    Hafen,
    /// Eintrag ins Logbuch. Projekt aus Arbeitsverzeichnis, `@projekt` oder --projekt.
    Log {
        text: Vec<String>,
        #[arg(long, short)]
        projekt: Option<String>,
        /// Als offenen Faden anlegen
        #[arg(long)]
        offen: bool,
        /// Als Entscheidung anlegen (mit Vorlage)
        #[arg(long)]
        entscheidung: bool,
    },
    /// Offene Fäden, projektübergreifend
    Offen {
        #[arg(long, short)]
        projekt: Option<String>,
    },
    /// Faden als erledigt markieren
    Erledigt { notiz_id: String },
    /// Volltextsuche
    Suche { text: Vec<String> },
    #[command(subcommand)]
    Projekt(ProjektCmd),
    #[command(subcommand)]
    Ref(RefCmd),
    #[command(subcommand)]
    Tresor(TresorCmd),
    /// Ordner nach Projekten durchsuchen (Hafeneinfahrt)
    Scan {
        wurzeln: Vec<PathBuf>,
        /// Alle Kandidaten sofort als Projekte anlegen
        #[arg(long)]
        uebernehmen: bool,
    },
    /// Einen Kandidaten als Projekt anlegen
    Uebernehmen { pfad: PathBuf },
    #[command(subcommand)]
    Export(ExportCmd),
    #[command(subcommand)]
    Sync(SyncCmd),
    /// MCP-Server über stdin/stdout für KI-Assistenten (z. B. `claude mcp add lotse -- lotse mcp`)
    Mcp,
    /// Nachsehen, ob es eine neuere Version gibt (lädt nichts herunter)
    Update,
    /// Anstehende Termine aus abonnierten Kalendern (.ics) zeigen
    Termine {
        /// Nur dieses Projekt; ohne Angabe alle mit Kalender-Referenz
        projekt: Option<String>,
        /// Wie weit nach vorn geschaut wird
        #[arg(long, default_value_t = 90)]
        tage: i64,
    },
    /// Stand auf der Gegenseite holen (GitHub, GitLab) und verdichtet ins Logbuch schreiben
    Gegenseite {
        /// Nur dieses Projekt; ohne Angabe alle mit erkanntem Repo
        projekt: Option<String>,
        /// Nur zeigen, nichts ins Logbuch schreiben
        #[arg(long)]
        trocken: bool,
    },
    /// Wurzelordner beobachten und Änderungen verdichtet ins Logbuch schreiben (läuft bis Strg+C)
    Beobachten {
        /// Wurzelordner; ohne Angabe die gemerkten. Mit --merken dauerhaft speichern.
        wurzeln: Vec<PathBuf>,
        #[arg(long)]
        merken: bool,
        /// Sekunden zwischen zwei Schreibvorgängen ins Logbuch
        #[arg(long, default_value_t = 30)]
        intervall: u64,
    },
}

#[derive(Subcommand)]
enum SyncCmd {
    /// Bestehendes lokales Konto beim Sync-Dienst registrieren (fragt den Wiederherstellungscode ab)
    Register {
        #[arg(long)]
        url: String,
        #[arg(long)]
        email: String,
    },
    /// Neues Gerät an einem bestehenden Konto anmelden und alles herunterladen
    Login {
        #[arg(long)]
        url: String,
        #[arg(long)]
        email: String,
        #[arg(long, default_value = "Dieser Rechner")]
        geraet: String,
    },
    /// Jetzt abgleichen: pushen, dann pullen
    Jetzt,
    /// Lokalen und entfernten Stand anzeigen
    Status,
    /// Geräte des Kontos auflisten
    Geraete,
    /// Ein Gerät widerrufen (seine Sitzungen verfallen)
    Widerrufen { id: String },
}

#[derive(Subcommand)]
enum ProjektCmd {
    Neu {
        titel: String,
        #[arg(long, value_enum, default_value = "generisch")]
        vorlage: VorlageArg,
        #[arg(long)]
        kurs: Option<String>,
    },
    Liste,
    Zeige {
        projekt: String,
    },
    /// Status wechseln. `pausiert` und `wartet` verlangen --notiz.
    Status {
        projekt: String,
        status: String,
        #[arg(long)]
        notiz: Option<String>,
        #[arg(long)]
        wiedervorlage: Option<String>,
    },
    Kurs {
        projekt: String,
        kurs: String,
    },
    Loeschen {
        projekt: String,
    },
}

#[derive(Subcommand)]
enum RefCmd {
    Add {
        projekt: String,
        typ: String,
        ziel: String,
        #[arg(long, default_value = "material")]
        rolle: String,
    },
    Liste {
        projekt: String,
    },
    Pruefen {
        id: String,
    },
}

#[derive(Subcommand)]
enum TresorCmd {
    /// Eintrag anlegen: `lotse tresor add "Fritzbox" --projekt Haus passwort=xyz user=admin`
    Add {
        titel: String,
        #[arg(long, short)]
        projekt: Option<String>,
        #[arg(long)]
        nur_desktop: bool,
        /// Felder als name=wert
        felder: Vec<String>,
    },
    Liste {
        #[arg(long, short)]
        projekt: Option<String>,
    },
    /// Ein Feld anzeigen
    Zeige {
        titel: String,
        feld: String,
    },
    Loeschen {
        titel: String,
    },
}

#[derive(Subcommand)]
enum ExportCmd {
    /// Klartext-Spiegel (Markdown) schreiben – ohne Tresor
    Spiegel { ziel: PathBuf },
    /// Gesamten Bestand als age-verschlüsseltes JSON-Bundle schreiben
    Bundle { ziel: PathBuf },
}

#[derive(Clone, Copy, ValueEnum)]
enum VorlageArg {
    Software,
    Hardware,
    Haus,
    Kreativ,
    Finanzen,
    Lernen,
    Reise,
    Generisch,
}

impl From<VorlageArg> for Vorlage {
    fn from(v: VorlageArg) -> Vorlage {
        match v {
            VorlageArg::Software => Vorlage::Software,
            VorlageArg::Hardware => Vorlage::HardwareMaker,
            VorlageArg::Haus => Vorlage::HausGarten,
            VorlageArg::Kreativ => Vorlage::Kreativ,
            VorlageArg::Finanzen => Vorlage::FinanzenVerwaltung,
            VorlageArg::Lernen => Vorlage::LernenForschung,
            VorlageArg::Reise => Vorlage::ReiseVeranstaltung,
            VorlageArg::Generisch => Vorlage::Generisch,
        }
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Fehler: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let home = cli
        .home
        .or_else(|| dirs::data_dir().map(|d| d.join("lotse")))
        .ok_or_else(|| anyhow!("Kein Datenverzeichnis bestimmbar; --home angeben"))?;

    if let Cmd::Init {
        geraet,
        ohne_bestaetigung,
        sync_url,
        email,
    } = &cli.cmd
    {
        return init(
            &home,
            geraet,
            *ohne_bestaetigung,
            sync_url.as_deref(),
            email.as_deref(),
        );
    }
    if let Cmd::Sync(SyncCmd::Login { url, email, geraet }) = &cli.cmd {
        return sync_login(&home, url, email, geraet);
    }
    // Nachsehen, ob es eine neuere Version gibt, geht ohne Konto und ohne Passwort.
    if let Cmd::Update = &cli.cmd {
        return update_pruefen();
    }

    let konto::Entsperrt {
        konto: konto_daten,
        mut store,
        account_key: ak,
        auth_key,
    } = oeffnen(&home)?;
    match cli.cmd {
        Cmd::Init { .. } | Cmd::Update => unreachable!(),
        Cmd::Hafen => hafen(&store),
        Cmd::Log {
            text,
            projekt,
            offen,
            entscheidung,
        } => log(&mut store, text, projekt, offen, entscheidung),
        Cmd::Offen { projekt } => {
            let pid = projekt
                .map(|p| finde_projekt(&store, &p))
                .transpose()?
                .map(|p| p.id);
            for n in store.offene_faeden(pid)? {
                let p = store
                    .projekt(n.projekt_id)?
                    .map(|p| p.titel)
                    .unwrap_or_default();
                println!("{}  [{}] {}", n.id, p, erste_zeile(&n.text));
            }
            Ok(())
        }
        Cmd::Erledigt { notiz_id } => {
            store.faden_erledigen(parse_ulid(&notiz_id)?)?;
            println!("Erledigt.");
            Ok(())
        }
        Cmd::Suche { text } => {
            for t in store.suche(&text.join(" "))? {
                let p = t
                    .projekt_id
                    .and_then(|id| store.projekt(id).ok().flatten())
                    .map(|p| p.titel)
                    .unwrap_or_default();
                println!(
                    "{:<8} {:<24} {}",
                    t.kind,
                    p,
                    t.ausschnitt.replace('\n', " ")
                );
            }
            Ok(())
        }
        Cmd::Projekt(c) => projekt(&mut store, c),
        Cmd::Ref(c) => referenz(&mut store, c),
        Cmd::Tresor(c) => tresor(&mut store, &ak, konto_daten.geraet_id, c),
        Cmd::Scan {
            wurzeln,
            uebernehmen,
        } => scan(&mut store, wurzeln, uebernehmen),
        Cmd::Uebernehmen { pfad } => {
            let k = detect::erkenne(&pfad)
                .ok_or_else(|| anyhow!("Kein Projekt erkannt in {}", pfad.display()))?;
            let p = detect::uebernehmen(&mut store, &k)?;
            println!("Angelegt: {} ({})", p.titel, p.vorlage.anzeigename());
            Ok(())
        }
        Cmd::Export(c) => exportieren(&store, &ak, konto_daten.geraet_id, c),
        Cmd::Sync(c) => sync(&mut store, &konto_daten, &auth_key, c),
        Cmd::Mcp => {
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            lotse_core::mcp::bedienen(&mut store, stdin.lock(), stdout.lock())?;
            Ok(())
        }
        Cmd::Termine { projekt, tage } => termine(&store, projekt.as_deref(), tage),
        Cmd::Gegenseite { projekt, trocken } => gegenseite(
            &mut store,
            &ak,
            konto_daten.geraet_id,
            projekt.as_deref(),
            trocken,
        ),
        Cmd::Beobachten {
            wurzeln,
            merken,
            intervall,
        } => beobachten(&mut store, wurzeln, merken, intervall),
    }
}

/// Sieht nach, ob es eine neuere Version gibt. Lädt nichts und führt nichts aus –
/// die Installer sind unsigniert (siehe `docs/THREAT_MODEL.md`).
fn update_pruefen() -> Result<()> {
    let laufend = env!("CARGO_PKG_VERSION");
    let repo = lotse_core::update::eigenes_repo();
    // Solange es nur Vorabversionen gibt, wären fertige Versionen eine leere Liste.
    let Some(v) = lotse_core::update::neueste(&repo, true)? else {
        println!("Version {laufend}. Keine Veröffentlichung gefunden.");
        return Ok(());
    };
    if !lotse_core::update::neuer_als(&v.version, laufend) {
        println!("Version {laufend} ist die neueste.");
        return Ok(());
    }
    let vorab = if v.vorab { " (Vorabversion)" } else { "" };
    println!("Version {} ist da{vorab}, hier läuft {laufend}.", v.version);
    if let Some(d) =
        lotse_core::update::passende_datei(&v, std::env::consts::OS, std::env::consts::ARCH)
    {
        println!("  {}  {}", d.name, d.url);
    }
    println!("  Alle Dateien: {}", v.seite);
    Ok(())
}

/// Zeigt, was ansteht: Termine aus den Kalender-Referenzen der Projekte. Lotse
/// schreibt sie nicht ins Logbuch – sie bleiben dort, wo sie gepflegt werden.
fn termine(store: &Store, projekt: Option<&str>, tage: i64) -> Result<()> {
    use lotse_core::kalender;

    let nur = projekt.map(|p| finde_projekt(store, p)).transpose()?;
    let von: String = export::iso(now_ms()).chars().take(10).collect();
    let bis = kalender::tage_spaeter(&von, tage.max(1));

    let mut etwas = false;
    for p in store.projekte()? {
        if nur.as_ref().is_some_and(|n| n.id != p.id) {
            continue;
        }
        let ziele: Vec<String> = store
            .referenzen(p.id)?
            .into_iter()
            .filter(|r| {
                matches!(r.typ, ReferenzTyp::Url | ReferenzTyp::Datei)
                    && kalender::ist_kalender(&r.ziel)
            })
            .map(|r| r.ziel)
            .collect();
        if ziele.is_empty() {
            continue;
        }
        etwas = true;

        let mut alle = Vec::new();
        for ziel in &ziele {
            match kalender::holen(ziel) {
                Ok(ics) => alle.extend(kalender::lesen(&ics)),
                Err(e) => eprintln!("{ziel}: {e}"),
            }
        }
        println!("{}", p.titel);
        let kommende = kalender::kommende(&alle, &von, &bis, 20);
        if kommende.is_empty() {
            println!("  nichts in den nächsten {tage} Tagen");
        }
        for t in kommende {
            let ort = t
                .ort
                .as_deref()
                .map(|o| format!("  ({o})"))
                .unwrap_or_default();
            println!("  {}  {}{ort}", t.anzeige(), t.titel);
        }
    }
    if !etwas {
        bail!(
            "Kein Projekt hat eine Kalender-Referenz. Die Abonnement-Adresse des Kalenders \
             als Referenz vom Typ `url` anlegen (endet auf .ics oder beginnt mit webcal://)."
        );
    }
    Ok(())
}

/// Zeiger auf den Tresor-Eintrag mit dem Token – dieselben Schlüssel wie in der App,
/// damit beide Oberflächen dieselbe Einstellung nutzen. Je Hoster getrennt: ein Token
/// für GitHub wird nie an GitLab geschickt.
const META_FORGE_EINTRAG: &str = "forge_token_eintrag";
const META_FORGE_FELD: &str = "forge_token_feld";
const META_FORGE_EINTRAG_GITLAB: &str = "forge_token_gitlab_eintrag";
const META_FORGE_FELD_GITLAB: &str = "forge_token_gitlab_feld";

/// Holt den Stand von GitHub oder GitLab. Den Token liefert entweder die Umgebung
/// (`LOTSE_FORGE_TOKEN`) oder der in der App hinterlegte Tresor-Eintrag.
fn gegenseite(
    store: &mut Store,
    ak: &Key32,
    geraet_id: Ulid,
    projekt: Option<&str>,
    trocken: bool,
) -> Result<()> {
    use lotse_core::forge;

    let nur = projekt.map(|p| finde_projekt(store, p)).transpose()?;

    let mut ziele = Vec::new();
    for p in store.projekte()? {
        if nur.as_ref().is_some_and(|n| n.id != p.id) {
            continue;
        }
        let refs = store.referenzen(p.id)?;
        if let Some((z, h)) = forge::zeiger_aus_referenzen(&refs, geraet_id) {
            ziele.push((p, z, h));
        }
    }
    if ziele.is_empty() {
        bail!(
            "Kein Projekt zeigt auf ein Repo bei GitHub oder GitLab. Eine Ordner-Referenz auf \
             einen geklonten Arbeitsordner genügt – das Remote liest Lotse selbst."
        );
    }

    // Je Hoster ein eigener Token, aus der Umgebung oder aus dem Tresor.
    let mut token_github = None;
    let mut token_gitlab = None;
    for anbieter in ziele.iter().map(|(_, z, _)| z.anbieter) {
        let ziel = match anbieter {
            forge::Anbieter::GitHub => &mut token_github,
            forge::Anbieter::GitLab => &mut token_gitlab,
        };
        if ziel.is_none() {
            *ziel = Some(forge_token(store, ak, geraet_id, anbieter)?);
        }
    }

    for (p, z, h) in ziele {
        let token = match z.anbieter {
            forge::Anbieter::GitHub => &token_github,
            forge::Anbieter::GitLab => &token_gitlab,
        };
        let token = token.as_ref().and_then(|t| t.as_deref());
        let stand = match forge::abfragen(&z, token) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{}: {e}", z.anzeige());
                continue;
            }
        };
        let woher = match h {
            forge::Herkunft::Adresse => "eingetragen",
            forge::Herkunft::Ordner => "am Ordner erkannt",
        };
        println!(
            "{}  {} ({}, {woher})",
            p.titel,
            z.anzeige(),
            z.anbieter.as_str()
        );
        println!(
            "  {} offen zur Übernahme, {} offene Issues, Prüflauf {} auf {}",
            stand.offene_prs,
            stand.offene_issues,
            stand.ci.as_str(),
            stand.standard_branch
        );
        match forge::notiz(p.id, &z, &stand, now_ms()) {
            Some(n) if !trocken => {
                store.notiz_speichern(&n)?;
                println!("  → Logbuch");
            }
            Some(_) => println!("  (trocken: nichts geschrieben)"),
            None => println!("  nichts zu melden"),
        }
    }
    Ok(())
}

/// Den Token eines Hosters: erst aus der Umgebung, sonst aus dem Tresor-Eintrag, den
/// die App hinterlegt hat.
fn forge_token(
    store: &Store,
    ak: &Key32,
    geraet_id: Ulid,
    anbieter: lotse_core::forge::Anbieter,
) -> Result<Option<String>> {
    use lotse_core::forge::Anbieter;
    let (umgebung, k_eintrag, k_feld) = match anbieter {
        Anbieter::GitHub => ("LOTSE_GITHUB_TOKEN", META_FORGE_EINTRAG, META_FORGE_FELD),
        Anbieter::GitLab => (
            "LOTSE_GITLAB_TOKEN",
            META_FORGE_EINTRAG_GITLAB,
            META_FORGE_FELD_GITLAB,
        ),
    };
    if let Some(t) = std::env::var(umgebung).ok().filter(|t| !t.is_empty()) {
        return Ok(Some(t));
    }
    let Some(id) = store.meta_get(k_eintrag)?.filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    let Ok(uid) = Ulid::from_string(&id) else {
        return Ok(None);
    };
    let Some(e) = store.tresor_eintrag(uid)? else {
        return Ok(None);
    };
    let feld = store
        .meta_get(k_feld)?
        .unwrap_or_else(|| "token".to_string());
    let keys = vault_keys(ak, geraet_id)?;
    Ok(Some(e.feld_lesen(&keys, &feld)?.to_string()))
}

fn beobachten(
    store: &mut Store,
    wurzeln: Vec<PathBuf>,
    merken: bool,
    intervall: u64,
) -> Result<()> {
    let mut wurzeln: Vec<PathBuf> = wurzeln
        .into_iter()
        .map(|w| w.canonicalize().unwrap_or(w))
        .collect();
    if wurzeln.is_empty() {
        wurzeln = watcher::wurzeln_laden(store)?;
    } else if merken {
        watcher::wurzeln_speichern(store, &wurzeln)?;
    }
    if wurzeln.is_empty() {
        bail!("Keine Wurzelordner. Angeben, z. B. `lotse beobachten ~/Projekte --merken`");
    }
    let mut b = Beobachter::starten(store, wurzeln)?;
    println!(
        "Beobachte {} Wurzelordner, schreibe alle {intervall} s. Beenden mit Strg+C.",
        b.wurzeln().len()
    );
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let s2 = stop.clone();
    ctrlc::set_handler(move || s2.store(true, std::sync::atomic::Ordering::SeqCst)).ok();
    // Erster Durchlauf setzt den Git-Stand und findet Kandidaten.
    let bilanz = b.schreiben(store)?;
    if bilanz.kandidaten > 0 {
        println!(
            "Hafeneinfahrt: {} neue Kandidaten (`lotse hafen`)",
            bilanz.kandidaten
        );
    }
    while !stop.load(std::sync::atomic::Ordering::SeqCst) {
        let n = b.verarbeiten(std::time::Duration::from_secs(intervall.max(1)));
        b.zuordnung_laden(store)?;
        let bilanz = b.schreiben(store)?;
        if n > 0 || bilanz.git_notizen > 0 || bilanz.kandidaten > 0 {
            println!(
                "{}  {n} Ereignisse, {} Datei-Notizen, {} Git-Notizen, {} Kandidaten",
                export::iso(now_ms()),
                bilanz.datei_notizen,
                bilanz.git_notizen,
                bilanz.kandidaten
            );
        }
    }
    b.schreiben(store)?;
    println!("Beobachter beendet.");
    Ok(())
}

// ------------------------------------------------------------------ konto

fn init(
    home: &Path,
    geraet: &str,
    ohne_bestaetigung: bool,
    sync_url: Option<&str>,
    email: Option<&str>,
) -> Result<()> {
    if konto::existiert(home) {
        bail!("In {} ist bereits ein Konto eingerichtet", home.display());
    }
    let pw = passwort_abfragen("Master-Passwort wählen: ")?;
    if pw.len() < 12 {
        bail!("Das Master-Passwort sollte mindestens 12 Zeichen haben");
    }
    let pw2 = passwort_abfragen("Master-Passwort wiederholen: ")?;
    if pw.as_str() != pw2.as_str() {
        bail!("Die Passwörter stimmen nicht überein");
    }
    let kdf = if std::env::var("LOTSE_KDF_SCHNELL").is_ok() {
        KdfParams::schnell_fuer_tests()
    } else {
        KdfParams::default()
    };
    eprintln!("Leite Schlüssel ab …");
    let mut e = konto::einrichten(home, pw.as_bytes(), geraet, kdf)?;
    let im_bund = konto::desktop_key_speichern(e.konto.geraet_id, &e.desktop_key).is_ok();

    println!();
    println!("Wiederherstellungscode (einmalige Anzeige, in Proton Pass ablegen):");
    println!("    {}", e.recovery_code.display().as_str());
    println!();
    println!("Desktop-Schlüssel (einmalige Anzeige, in Proton Pass ablegen; nie auf fremden Rechnern eingeben):");
    println!("    {}", e.desktop_key.display().as_str());
    if im_bund {
        println!("    (zusätzlich im Schlüsselbund dieses Rechners abgelegt)");
    } else {
        println!("    (kein Schlüsselbund verfügbar; für »nur Desktop«-Einträge LOTSE_DESKTOP_KEY setzen)");
    }
    println!();
    let ohne_bestaetigung = ohne_bestaetigung || std::env::var_os("LOTSE_SKIP_CONFIRM").is_some();
    if !ohne_bestaetigung {
        let eingabe = passwort_abfragen("Zur Bestätigung den Wiederherstellungscode eingeben: ")?;
        let ok = RecoveryCode::parse(&eingabe)
            .map(|c| crypto::konto_wiederherstellen(&e.konto.header, &c).is_ok())
            .unwrap_or(false);
        if !ok {
            drop(e);
            konto::verwerfen(home)?;
            bail!("Der eingegebene Code ist falsch. Einrichtung verworfen.");
        }
    }
    println!("Eingerichtet in {}", home.display());
    if let (Some(url), Some(email)) = (sync_url, email) {
        let client = Client::new(url);
        let sg = SyncGeraet {
            id: e.konto.geraet_id,
            name: geraet.to_string(),
            platform: "cli".into(),
        };
        let (account_id, token) = client
            .register(
                email,
                &e.auth_key,
                &e.recovery_auth_key,
                &e.konto.header,
                &sg,
            )
            .context("Registrierung beim Sync-Dienst")?;
        sync_client::verbindung_merken(&e.store, url, email, &account_id, &token)?;
        let r = sync_client::abgleichen(&mut e.store, &client.with_token(&token))?;
        println!(
            "Beim Sync-Dienst registriert ({url}), {} Datensätze gepusht.",
            r.gepusht
        );
    }
    Ok(())
}

fn oeffnen(home: &Path) -> Result<konto::Entsperrt> {
    if !konto::existiert(home) {
        bail!(
            "Kein Konto in {}. Zuerst `lotse init` oder `lotse sync login`.",
            home.display()
        );
    }
    let pw = passwort_abfragen("Master-Passwort: ")?;
    konto::entsperren(home, pw.as_bytes()).map_err(|e| match e {
        lotse_core::Error::Decrypt => anyhow!("Falsches Master-Passwort"),
        e => anyhow!(e),
    })
}

/// Neues Gerät: Passwort → Prelogin → Login → Konto-Datei → Speicher → Abgleich.
fn sync_login(home: &Path, url: &str, email: &str, geraet: &str) -> Result<()> {
    if konto::existiert(home) {
        bail!(
            "In {} ist bereits ein Konto eingerichtet; für ein bestehendes Konto `lotse sync register`",
            home.display()
        );
    }
    let client = Client::new(url);
    let pre = client.prelogin(email).context("Prelogin")?;
    let pw = passwort_abfragen("Master-Passwort des Kontos: ")?;
    eprintln!("Leite Schlüssel ab …");
    let stretched = crypto::derive_stretched(pw.as_bytes(), &pre.salt, &pre.kdf)?;
    let pk = crypto::split_password_keys(&stretched);
    let geraet_id = Ulid::new();
    let sg = SyncGeraet {
        id: geraet_id,
        name: geraet.to_string(),
        platform: "cli".into(),
    };
    let login = client
        .login(email, &pk.auth, &sg)
        .map_err(|e| anyhow!("Anmeldung fehlgeschlagen: {e}"))?;
    let (ak, _) = crypto::konto_entsperren(&login.header, pw.as_bytes()).map_err(|_| {
        anyhow!("Der Dienst lieferte einen Konto-Header, der sich mit diesem Passwort nicht öffnen lässt")
    })?;
    let (_, mut store) = konto::aus_login(home, login.header, geraet_id, geraet, &ak)?;
    sync_client::verbindung_merken(&store, url, email, &login.account_id, &login.session_token)?;
    let r = sync_client::abgleichen(
        &mut store,
        &Client::new(url).with_token(&login.session_token),
    )?;
    println!(
        "Angemeldet als {email} auf Gerät »{geraet}«. {} Datensätze übernommen, {} Projekte lokal.",
        r.uebernommen,
        store.projekte()?.len()
    );
    Ok(())
}

fn passwort_abfragen(prompt: &str) -> Result<Zeroizing<String>> {
    if let Ok(pw) = std::env::var("LOTSE_PASSWORD") {
        return Ok(Zeroizing::new(pw));
    }
    let pw = rpassword::prompt_password(prompt).context("Passwort-Eingabe")?;
    Ok(Zeroizing::new(pw))
}

/// Tresor-Schlüssel: mit Desktop-Schlüssel aus Schlüsselbund oder Umgebung, sonst ohne.
fn vault_keys(ak: &Key32, geraet_id: Ulid) -> Result<VaultKeys> {
    let dk = konto::desktop_key_aus_umgebung()?.or(konto::desktop_key_laden(geraet_id)?);
    Ok(match dk {
        Some(dk) => VaultKeys::with_desktop_key(ak, &dk),
        None => VaultKeys::from_account_key(ak),
    })
}

fn sync(store: &mut Store, konto: &konto::Konto, auth_key: &Key32, c: SyncCmd) -> Result<()> {
    match c {
        SyncCmd::Login { .. } => unreachable!("wird vor dem Öffnen behandelt"),
        SyncCmd::Register { url, email } => {
            let code = std::env::var("LOTSE_RECOVERY_CODE")
                .map(Zeroizing::new)
                .or_else(|_| {
                    passwort_abfragen(
                        "Wiederherstellungscode (für die Konto-Wiederherstellung beim Dienst): ",
                    )
                })?;
            let code = RecoveryCode::parse(&code).context("Code nicht lesbar")?;
            let salt: [u8; crypto::SALT_LEN] =
                konto.header.salt.as_slice().try_into().context("Salt")?;
            let rk = code.derive_key(&salt, &konto.header.kdf)?;
            let recovery_auth = crypto::recovery_auth_key(&rk);
            let sg = SyncGeraet {
                id: konto.geraet_id,
                name: konto.geraet_name.clone(),
                platform: "cli".into(),
            };
            let client = Client::new(&url);
            let (account_id, token) = client
                .register(&email, auth_key, &recovery_auth, &konto.header, &sg)
                .context("Registrierung beim Sync-Dienst")?;
            sync_client::verbindung_merken(store, &url, &email, &account_id, &token)?;
            let r = sync_client::abgleichen(store, &client.with_token(&token))?;
            println!(
                "Registriert als {email} bei {url}. {} Datensätze gepusht.",
                r.gepusht
            );
        }
        SyncCmd::Jetzt => {
            let client = sync_client::client_aus_store(store)?;
            let r = sync_client::abgleichen(store, &client)?;
            println!(
                "Gepusht {}, übernommen {}, verworfen {}, Konflikte {}. Server-Sequenz {}.",
                r.gepusht, r.uebernommen, r.verworfen, r.konflikte, r.server_seq
            );
            if r.abgelehnt > 0 {
                println!("Abgelehnt: {} (siehe Dienst-Antwort)", r.abgelehnt);
            }
        }
        SyncCmd::Status => {
            let s = store.sync_state()?;
            println!("Ausstehende Änderungen: {}", store.ausstehend()?);
            println!(
                "Zuletzt gepusht: lokale Sequenz {}",
                s.last_pushed_local_seq
            );
            println!("Zuletzt gepullt: Server-Sequenz {}", s.last_server_seq);
            match sync_client::client_aus_store(store) {
                Ok(client) => match client.status() {
                    Ok(st) => println!(
                        "Dienst {}: Sequenz {}, {} Datensätze, {} Byte Anhänge",
                        client.base_url(),
                        st.server_seq,
                        st.record_count,
                        st.blob_bytes
                    ),
                    Err(e) => println!("Dienst nicht erreichbar: {e}"),
                },
                Err(_) => println!(
                    "Kein Sync eingerichtet (`lotse sync register` oder `lotse sync login`)."
                ),
            }
        }
        SyncCmd::Geraete => {
            let client = sync_client::client_aus_store(store)?;
            for g in client.geraete()? {
                println!(
                    "{}  {:<8} {:<24} zuletzt {}",
                    g.id,
                    g.platform,
                    g.name,
                    export::iso(g.last_seen_at)
                );
            }
        }
        SyncCmd::Widerrufen { id } => {
            let client = sync_client::client_aus_store(store)?;
            client.geraet_widerrufen(&id)?;
            println!("Gerät {id} widerrufen.");
        }
    }
    Ok(())
}

// ---------------------------------------------------------------- befehle

fn hafen(store: &Store) -> Result<()> {
    let jetzt = now_ms();
    let karten = store.hafen(jetzt)?;
    let kandidaten = store.kandidaten()?;
    if !kandidaten.is_empty() {
        println!("Hafeneinfahrt: {} erkannte Projekte warten auf Bestätigung (`lotse uebernehmen <pfad>`)", kandidaten.len());
        for k in kandidaten.iter().take(5) {
            println!("   {} → {}  ({})", k.name, k.vorlage.anzeigename(), k.pfad);
        }
        println!();
    }
    let gruppe = |titel: &str, filter: &dyn Fn(&Projekt) -> bool| {
        let sel: Vec<_> = karten.iter().filter(|k| filter(&k.projekt)).collect();
        if sel.is_empty() {
            return;
        }
        println!("{titel}");
        for k in sel {
            let marker = match k.auffaelligkeit {
                brief::Auffaelligkeit::Ueberfaellig => "!!",
                brief::Auffaelligkeit::Auffaellig => " !",
                brief::Auffaelligkeit::Ruhig => "  ",
            };
            let letzte = k
                .letzte_notiz
                .as_ref()
                .map(|n| erste_zeile(&n.text))
                .unwrap_or_else(|| "–".into());
            println!(
                "{marker} {:<28} {:>4} T  {:>2} offen  {}",
                kuerzen(&k.projekt.titel, 28),
                k.tage_seit,
                k.offene_faeden,
                kuerzen(&letzte, 60)
            );
        }
        println!();
    };
    gruppe("Auf See", &|p| p.status == Status::Aktiv);
    gruppe("Vor Anker", &|p| {
        matches!(p.status, Status::Pausiert | Status::Wartet)
    });
    gruppe("Ideen", &|p| p.status == Status::Idee);
    gruppe("Abgeschlossen / eingemottet", &|p| {
        matches!(p.status, Status::Abgeschlossen | Status::Eingemottet)
    });
    Ok(())
}

fn log(
    store: &mut Store,
    text: Vec<String>,
    projekt: Option<String>,
    offen: bool,
    entscheidung: bool,
) -> Result<()> {
    let mut text = text.join(" ");
    let mut ziel: Option<Projekt> = None;
    if let Some(p) = projekt {
        ziel = Some(finde_projekt(store, &p)?);
    } else if let Some(rest) = text.strip_prefix('@') {
        let (name, body) = rest.split_once(' ').unwrap_or((rest, ""));
        if let Ok(p) = finde_projekt(store, name) {
            ziel = Some(p);
            text = body.to_string();
        }
    }
    if ziel.is_none() {
        ziel = projekt_aus_arbeitsverzeichnis(store)?;
    }
    let ziel = match ziel {
        Some(p) => p,
        None => {
            let p = store.postkorb()?;
            eprintln!("Kein Projekt erkannt – landet im Postkorb.");
            p
        }
    };
    if text.trim().is_empty() {
        bail!("Kein Text");
    }
    let art = if offen {
        Art::Offen
    } else if entscheidung {
        text = format!("{}\n\n{}", Notiz::entscheidungs_vorlage(), text);
        Art::Entscheidung
    } else {
        Art::Log
    };
    let n = Notiz::neu(ziel.id, Quelle::Cli, art, text, now_ms());
    store.notiz_speichern(&n)?;
    println!("→ {} ({})", ziel.titel, art.as_str());
    Ok(())
}

fn projekt_aus_arbeitsverzeichnis(store: &Store) -> Result<Option<Projekt>> {
    let mut dir = std::env::current_dir()?;
    loop {
        if let Some(id) = detect::marker_lesen(&dir) {
            return Ok(store.projekt(id)?);
        }
        if !dir.pop() {
            return Ok(None);
        }
    }
}

fn projekt(store: &mut Store, c: ProjektCmd) -> Result<()> {
    match c {
        ProjektCmd::Neu {
            titel,
            vorlage,
            kurs,
        } => {
            let mut p = Projekt::neu(titel, vorlage.into(), now_ms());
            if let Some(k) = kurs {
                p.kurs = k;
            }
            store.projekt_speichern(&p)?;
            println!("Angelegt: {} ({})", p.titel, p.id);
        }
        ProjektCmd::Liste => {
            for p in store.projekte()? {
                println!(
                    "{}  {:<12} {:<22} {}",
                    p.id,
                    p.status.as_str(),
                    p.vorlage.anzeigename(),
                    p.titel
                );
            }
        }
        ProjektCmd::Zeige { projekt } => {
            let p = finde_projekt(store, &projekt)?;
            let jetzt = now_ms();
            let b = store.brief(p.id, jetzt)?;
            println!(
                "{}  [{}]  {}",
                p.titel,
                p.status.as_str(),
                p.vorlage.anzeigename()
            );
            if !p.kurs.is_empty() {
                println!("Kurs: {}", p.kurs);
            }
            if brief::brief_faellig(&p, b.offene_faeden.first().map(|n| n.ts), jetzt)
                || b.tage_seit > p.erwartungsintervall_tage as i64
            {
                println!();
                println!("Wo war ich?  Zuletzt vor {} Tagen.", b.tage_seit);
                if let Some(u) = &b.letzte_uebergabe {
                    println!("Übergabe: {u}");
                } else if let Some(l) = &b.letzte_notiz {
                    println!("Letzte Notiz: {}", erste_zeile(l));
                }
                if !b.aktivitaet_seit_letztem_besuch.is_empty() {
                    let teile: Vec<String> = b
                        .aktivitaet_seit_letztem_besuch
                        .iter()
                        .map(|(q, n)| format!("{n} × {q}"))
                        .collect();
                    println!("Seitdem: {}", teile.join(", "));
                }
            }
            if !b.offene_faeden.is_empty() {
                println!();
                println!("Offen:");
                for n in &b.offene_faeden {
                    println!("  - {}", erste_zeile(&n.text));
                }
            }
            let refs = store.referenzen(p.id)?;
            if !refs.is_empty() {
                println!();
                println!("Referenzen:");
                for r in refs {
                    println!(
                        "  {:<15} {:<15} {}",
                        r.typ.as_str(),
                        r.pruefstatus.as_str(),
                        r.ziel
                    );
                }
            }
            println!();
            println!("Logbuch:");
            for n in store.notizen(p.id)?.iter().rev().take(15) {
                println!(
                    "  {}  {:<12} {:<7} {}",
                    export::iso(n.ts),
                    n.art.as_str(),
                    n.quelle.as_str(),
                    erste_zeile(&n.text)
                );
            }
        }
        ProjektCmd::Status {
            projekt,
            status,
            notiz,
            wiedervorlage,
        } => {
            let p = finde_projekt(store, &projekt)?;
            let st = Status::parse(&status).ok_or_else(|| anyhow!("Unbekannter Status. Erlaubt: idee, aktiv, pausiert, wartet, abgeschlossen, eingemottet"))?;
            store.status_setzen(
                p.id,
                st,
                notiz.as_deref(),
                wiedervorlage.as_deref(),
                Quelle::Cli,
            )?;
            println!("{} → {}", p.titel, st.as_str());
        }
        ProjektCmd::Kurs { projekt, kurs } => {
            let mut p = finde_projekt(store, &projekt)?;
            p.kurs = kurs;
            store.projekt_speichern(&p)?;
            println!("Kurs gesetzt.");
        }
        ProjektCmd::Loeschen { projekt } => {
            let p = finde_projekt(store, &projekt)?;
            store.projekt_loeschen(p.id)?;
            println!("Gelöscht: {}", p.titel);
        }
    }
    Ok(())
}

fn referenz(store: &mut Store, c: RefCmd) -> Result<()> {
    match c {
        RefCmd::Add {
            projekt,
            typ,
            ziel,
            rolle,
        } => {
            let p = finde_projekt(store, &projekt)?;
            let typ = ReferenzTyp::parse(&typ).ok_or_else(|| {
                anyhow!("Typ: ordner, git_repo, url, datei, physisch, geraet, passwortmanager")
            })?;
            let rolle =
                Rolle::parse(&rolle).ok_or_else(|| anyhow!("Rolle: material, ergebnis, doku"))?;
            let r = Referenz::neu(p.id, typ, ziel, rolle);
            store.referenz_speichern(&r)?;
            println!("Referenz {} angelegt.", r.id);
        }
        RefCmd::Liste { projekt } => {
            let p = finde_projekt(store, &projekt)?;
            for r in store.referenzen(p.id)? {
                println!(
                    "{}  {:<15} {:<15} {}",
                    r.id,
                    r.typ.as_str(),
                    r.pruefstatus.as_str(),
                    r.ziel
                );
            }
        }
        RefCmd::Pruefen { id } => {
            let st = store.referenz_pruefen(parse_ulid(&id)?)?;
            println!("{}", st.as_str());
        }
    }
    Ok(())
}

fn tresor(store: &mut Store, ak: &Key32, geraet_id: Ulid, c: TresorCmd) -> Result<()> {
    let keys = vault_keys(ak, geraet_id)?;
    match c {
        TresorCmd::Add {
            titel,
            projekt,
            nur_desktop,
            felder,
        } => {
            let stufe = if nur_desktop {
                Stufe::NurDesktop
            } else {
                Stufe::Ueberall
            };
            if stufe == Stufe::NurDesktop && !keys.kann_nur_desktop() {
                bail!("Für »nur Desktop« wird der Desktop-Schlüssel gebraucht (LOTSE_DESKTOP_KEY)");
            }
            let pids = projekt
                .map(|p| finde_projekt(store, &p))
                .transpose()?
                .map(|p| vec![p.id])
                .unwrap_or_default();
            let paare: Vec<(String, String)> = felder
                .iter()
                .map(|f| {
                    f.split_once('=')
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                        .ok_or_else(|| anyhow!("Feld muss name=wert sein: {f}"))
                })
                .collect::<Result<_>>()?;
            let refs: Vec<(&str, &str)> = paare
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            let e = TresorEintrag::neu(&keys, titel, pids, stufe, &refs, now_ms())?;
            store.tresor_speichern(&e)?;
            println!(
                "Tresor-Eintrag »{}« angelegt ({}).",
                e.titel,
                e.stufe.as_str()
            );
        }
        TresorCmd::Liste { projekt } => {
            let pid = projekt
                .map(|p| finde_projekt(store, &p))
                .transpose()?
                .map(|p| p.id);
            for e in store.tresor_eintraege(pid)? {
                let felder: Vec<&str> = e.felder.iter().map(|f| f.name.as_str()).collect();
                let lesbar = if e.lesbar_mit(&keys) {
                    ""
                } else {
                    "  (hier nicht lesbar)"
                };
                println!(
                    "{:<12} {:<30} {}{}",
                    e.stufe.as_str(),
                    e.titel,
                    felder.join(", "),
                    lesbar
                );
            }
        }
        TresorCmd::Zeige { titel, feld } => {
            let e = store
                .tresor_eintraege(None)?
                .into_iter()
                .find(|e| e.titel.eq_ignore_ascii_case(&titel))
                .ok_or_else(|| anyhow!("Kein Eintrag »{titel}«"))?;
            let wert = e.feld_lesen(&keys, &feld)?;
            println!("{}", wert.as_str());
        }
        TresorCmd::Loeschen { titel } => {
            let e = store
                .tresor_eintraege(None)?
                .into_iter()
                .find(|e| e.titel.eq_ignore_ascii_case(&titel))
                .ok_or_else(|| anyhow!("Kein Eintrag »{titel}«"))?;
            store.tresor_loeschen(e.id)?;
            println!("Gelöscht und Schlüssel vernichtet.");
        }
    }
    Ok(())
}

fn scan(store: &mut Store, wurzeln: Vec<PathBuf>, uebernehmen: bool) -> Result<()> {
    if wurzeln.is_empty() {
        bail!("Mindestens einen Wurzelordner angeben");
    }
    let wurzeln: Vec<PathBuf> = wurzeln
        .into_iter()
        .map(|w| w.canonicalize().unwrap_or(w))
        .collect();
    let alle = detect::scan(&wurzeln, &detect::ScanOptionen::default())?;
    // Bereits bekannte Projekte (Marker) sind keine Kandidaten mehr.
    let neu: Vec<Kandidat> = alle
        .into_iter()
        .filter(|k| {
            k.bekannte_id
                .and_then(|id| store.projekt(id).ok().flatten())
                .is_none()
        })
        .collect();
    if neu.is_empty() {
        println!("Keine neuen Projekte gefunden.");
        return Ok(());
    }
    if uebernehmen {
        for k in &neu {
            let p = detect::uebernehmen(store, k)?;
            println!("Angelegt: {} ({})", p.titel, p.vorlage.anzeigename());
        }
    } else {
        store.kandidaten_merken(&neu)?;
        println!("{} Kandidaten in der Hafeneinfahrt:", neu.len());
        for k in &neu {
            println!(
                "  {:<24} {:<22} {}",
                kuerzen(&k.name, 24),
                k.vorlage.anzeigename(),
                k.pfad
            );
        }
        println!(
            "Übernehmen mit `lotse uebernehmen <pfad>` oder alle mit `lotse scan --uebernehmen …`."
        );
    }
    Ok(())
}

fn exportieren(store: &Store, ak: &Key32, geraet_id: Ulid, c: ExportCmd) -> Result<()> {
    match c {
        ExportCmd::Spiegel { ziel } => {
            let n = export::spiegel::schreiben(store, &ziel)?;
            println!(
                "{n} Projekte nach {} gespiegelt (Klartext, ohne Tresor).",
                ziel.display()
            );
        }
        ExportCmd::Bundle { ziel } => {
            let keys = vault_keys(ak, geraet_id)?;
            let pass = std::env::var("LOTSE_EXPORT_PASSPHRASE")
                .map(Zeroizing::new)
                .or_else(|_| passwort_abfragen("Passphrase für das Bundle: "))?;
            let b = export::bundle::schreiben(store, Some(&keys), &pass, &ziel)?;
            println!(
                "{} Projekte, {} Notizen, {} Tresor-Einträge nach {} geschrieben.",
                b.projekte.len(),
                b.notizen.len(),
                b.tresor.len(),
                ziel.display()
            );
            if !b.tresor_nicht_lesbar.is_empty() {
                println!(
                    "Nicht enthalten (hier nicht lesbar): {}",
                    b.tresor_nicht_lesbar.join(", ")
                );
            }
            println!(
                "Entschlüsseln ohne Lotse: age -d -o bundle.json {}",
                ziel.display()
            );
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ helfer

fn finde_projekt(store: &Store, s: &str) -> Result<Projekt> {
    if let Ok(id) = Ulid::from_string(s) {
        if let Some(p) = store.projekt(id)? {
            return Ok(p);
        }
    }
    if let Some(p) = store.projekt_nach_titel(s)? {
        return Ok(p);
    }
    let s_l = s.to_lowercase();
    let treffer: Vec<Projekt> = store
        .projekte()?
        .into_iter()
        .filter(|p| p.titel.to_lowercase().starts_with(&s_l))
        .collect();
    match treffer.len() {
        1 => Ok(treffer.into_iter().next().unwrap()),
        0 => bail!("Kein Projekt »{s}«"),
        _ => bail!(
            "Mehrdeutig: {}",
            treffer
                .iter()
                .map(|p| p.titel.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn parse_ulid(s: &str) -> Result<Ulid> {
    Ulid::from_string(s).map_err(|_| anyhow!("Keine gültige ID: {s}"))
}

fn erste_zeile(s: &str) -> String {
    s.lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .to_string()
}

fn kuerzen(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n - 1).collect();
        out.push('…');
        out
    }
}
