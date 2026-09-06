//! Ordner-Beobachter: sammelt Metadaten über Dateiänderungen in registrierten Wurzelordnern
//! und schreibt sie verdichtet ins Logbuch (`docs/CONCEPT.md`, Abschnitt 4).
//!
//! Gesammelt werden ausschließlich Pfade und Zeitpunkte, nie Inhalte. Pro Projekt und Tag
//! entsteht eine Notiz mit Quelle `datei`, die im Laufe des Tages fortgeschrieben wird
//! ("14 Dateien geändert, 2 neu: src/main.rs, …"). Bei Projekten mit Git-Repo kommen die
//! Commits seit dem letzten Blick als Notizen mit Quelle `git` dazu. Neue Projektordner
//! landen als Kandidaten in der Hafeneinfahrt.
//!
//! Dieses Modul importiert **nie** aus `vault`.

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::time::{Duration, Instant};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use ulid::Ulid;

use crate::brief::MS_PRO_TAG;
use crate::detect::{self, AUSSCHLUSS};
use crate::model::{Art, Notiz, Quelle, ReferenzTyp};
use crate::store::Store;
use crate::{git, now_ms, Error, Result};

/// Meta-Schlüssel, unter dem die Wurzelordner dieses Geräts liegen (JSON-Liste).
pub const META_WURZELN: &str = "watch_wurzeln";
const META_GIT_STAND: &str = "watch_git_stand";

#[derive(Debug, Default, Clone)]
struct Aggregat {
    geaendert: BTreeSet<String>,
    neu: usize,
    letzte: String,
    letzter_ts: i64,
}

/// Was ein Schreibvorgang ins Logbuch gebracht hat.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Bilanz {
    pub datei_notizen: usize,
    pub git_notizen: usize,
    pub kandidaten: usize,
}

pub struct Beobachter {
    _watcher: RecommendedWatcher,
    rx: Receiver<notify::Result<Event>>,
    wurzeln: Vec<PathBuf>,
    /// Ordner → Projekt, längster Pfad zuerst.
    zuordnung: Vec<(PathBuf, Ulid)>,
    aggregat: HashMap<(Ulid, i64), Aggregat>,
    notiz_ids: HashMap<(Ulid, i64), Ulid>,
    letzter_scan: Option<Instant>,
}

impl Beobachter {
    /// Startet die Beobachtung der Wurzelordner. Die Zuordnung zu Projekten kommt aus den
    /// Ordner- und Repo-Referenzen dieses Geräts.
    pub fn starten(store: &Store, wurzeln: Vec<PathBuf>) -> Result<Beobachter> {
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |ev| {
            let _ = tx.send(ev);
        })
        .map_err(|e| Error::Other(format!("Beobachter: {e}")))?;
        let mut aktive = Vec::new();
        for w in &wurzeln {
            if w.is_dir() {
                watcher
                    .watch(w, RecursiveMode::Recursive)
                    .map_err(|e| Error::Other(format!("Beobachter {}: {e}", w.display())))?;
                aktive.push(w.clone());
            }
        }
        let mut b = Beobachter {
            _watcher: watcher,
            rx,
            wurzeln: aktive,
            zuordnung: Vec::new(),
            aggregat: HashMap::new(),
            notiz_ids: HashMap::new(),
            letzter_scan: None,
        };
        b.zuordnung_laden(store)?;
        Ok(b)
    }

    pub fn wurzeln(&self) -> &[PathBuf] {
        &self.wurzeln
    }

    /// Liest die Ordner-Referenzen aller Projekte neu ein.
    pub fn zuordnung_laden(&mut self, store: &Store) -> Result<()> {
        let mut z: Vec<(PathBuf, Ulid)> = store
            .ordner_referenzen()?
            .into_iter()
            .map(|r| (PathBuf::from(r.ziel), r.projekt_id))
            .collect();
        z.sort_by_key(|(p, _)| std::cmp::Reverse(p.as_os_str().len()));
        self.zuordnung = z;
        Ok(())
    }

    fn projekt_fuer(&self, pfad: &Path) -> Option<Ulid> {
        self.zuordnung
            .iter()
            .find(|(root, _)| pfad.starts_with(root))
            .map(|(_, id)| *id)
    }

    fn relevant(pfad: &Path) -> bool {
        let name = pfad.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.ends_with('~')
            || name.ends_with(".tmp-lotse")
            || name.ends_with(".swp")
            || name.starts_with(".#")
        {
            return false;
        }
        for teil in pfad.components() {
            let t = teil.as_os_str().to_string_lossy();
            if AUSSCHLUSS.iter().any(|a| *a == t) || (t.starts_with('.') && t != "." && t != "..") {
                return false;
            }
        }
        !detect::NIE_LESEN.iter().any(|m| passt(m, name))
    }

    /// Nimmt Ereignisse für höchstens `dauer` entgegen und verdichtet sie im Speicher.
    /// Liefert die Zahl verarbeiteter Ereignisse.
    pub fn verarbeiten(&mut self, dauer: Duration) -> usize {
        let ende = Instant::now() + dauer;
        let mut n = 0;
        loop {
            let rest = ende.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(rest) {
                Ok(Ok(ev)) => {
                    n += 1;
                    self.ereignis(ev);
                }
                Ok(Err(_)) => {}
                Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => break,
            }
            if Instant::now() >= ende {
                break;
            }
        }
        n
    }

    fn ereignis(&mut self, ev: Event) {
        let neu = matches!(ev.kind, EventKind::Create(_));
        if !matches!(
            ev.kind,
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
        ) {
            return;
        }
        let ts = now_ms();
        for pfad in ev.paths {
            let Some(pid) = self.projekt_fuer(&pfad) else {
                continue;
            };
            let root = self
                .zuordnung
                .iter()
                .find(|(r, id)| *id == pid && pfad.starts_with(r))
                .map(|(r, _)| r.clone())
                .unwrap_or_default();
            let rel_pfad = pfad.strip_prefix(&root).unwrap_or(&pfad).to_path_buf();
            // Nur der Teil unterhalb des Projektordners zählt; darüber liegende versteckte
            // Ordner (z. B. ein Temp-Verzeichnis) sind egal.
            if !Self::relevant(&rel_pfad) {
                continue;
            }
            let rel = rel_pfad.to_string_lossy().to_string();
            let agg = self.aggregat.entry((pid, ts / MS_PRO_TAG)).or_default();
            if neu && agg.geaendert.insert(rel.clone()) {
                agg.neu += 1;
            } else {
                agg.geaendert.insert(rel.clone());
            }
            agg.letzte = rel;
            agg.letzter_ts = ts;
        }
    }

    /// Schreibt die Verdichtung ins Logbuch, holt Commits und sucht neue Kandidaten.
    pub fn schreiben(&mut self, store: &mut Store) -> Result<Bilanz> {
        let mut bilanz = Bilanz::default();

        // 1) Dateiänderungen: eine Notiz pro Projekt und Tag, fortgeschrieben.
        let schluessel: Vec<(Ulid, i64)> = self.aggregat.keys().copied().collect();
        for k in schluessel {
            let agg = self.aggregat.get(&k).cloned().unwrap_or_default();
            if agg.geaendert.is_empty() {
                continue;
            }
            let id = *self.notiz_ids.entry(k).or_insert_with(Ulid::new);
            let text = text_fuer(&agg);
            let notiz = match store.notiz(id)? {
                Some(mut n) => {
                    n.text = text;
                    n.ts = agg.letzter_ts;
                    n
                }
                None => {
                    let mut n = Notiz::neu(k.0, Quelle::Datei, Art::Log, text, agg.letzter_ts);
                    n.id = id;
                    n
                }
            };
            if store.projekt(k.0)?.is_some() {
                store.notiz_speichern(&notiz)?;
                bilanz.datei_notizen += 1;
            }
        }
        // Verdichtungen vergangener Tage sind abgeschlossen und können weg.
        let heute = now_ms() / MS_PRO_TAG;
        self.aggregat.retain(|(_, tag), _| *tag == heute);
        self.notiz_ids.retain(|(_, tag), _| *tag == heute);

        // 2) Git-Commits seit dem letzten Blick, pro Repo-Referenz.
        let mut stand: HashMap<String, i64> = store
            .meta_get(META_GIT_STAND)?
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        for r in store.ordner_referenzen()? {
            if r.typ != ReferenzTyp::GitRepo {
                continue;
            }
            let seit = stand.get(&r.ziel).copied();
            let jetzt = now_ms();
            let commits = git::log(Path::new(&r.ziel), seit, 200)?;
            let commits: Vec<_> = commits
                .into_iter()
                .filter(|c| seit.is_none_or(|s| c.ts_ms > s))
                .collect();
            if seit.is_some() && !commits.is_empty() {
                for n in git::verdichten(r.projekt_id, &commits, Quelle::Git) {
                    store.notiz_speichern(&n)?;
                    bilanz.git_notizen += 1;
                }
            }
            stand.insert(r.ziel.clone(), jetzt);
        }
        store.meta_set(META_GIT_STAND, &serde_json::to_string(&stand)?)?;

        // 3) Neue Projektordner unter den Wurzeln, höchstens alle zehn Minuten.
        let faellig = self
            .letzter_scan
            .is_none_or(|t| t.elapsed() > Duration::from_secs(600));
        if faellig && !self.wurzeln.is_empty() {
            let bekannt: std::collections::HashSet<PathBuf> =
                self.zuordnung.iter().map(|(p, _)| p.clone()).collect();
            let kandidaten: Vec<_> = detect::scan(&self.wurzeln, &detect::ScanOptionen::default())?
                .into_iter()
                .filter(|k| !bekannt.contains(Path::new(&k.pfad)))
                .filter(|k| {
                    k.bekannte_id
                        .and_then(|id| store.projekt(id).ok().flatten())
                        .is_none()
                })
                .collect();
            bilanz.kandidaten = kandidaten.len();
            if !kandidaten.is_empty() {
                store.kandidaten_merken(&kandidaten)?;
            }
            self.letzter_scan = Some(Instant::now());
        }
        Ok(bilanz)
    }
}

fn text_fuer(agg: &Aggregat) -> String {
    let n = agg.geaendert.len();
    let mut s = if n == 1 {
        "1 Datei geändert".to_string()
    } else {
        format!("{n} Dateien geändert")
    };
    if agg.neu > 0 {
        s.push_str(&format!(", {} neu", agg.neu));
    }
    s.push_str(": ");
    let mut liste: Vec<&String> = agg.geaendert.iter().collect();
    liste.sort();
    let gezeigt: Vec<&str> = liste.iter().take(6).map(|p| p.as_str()).collect();
    s.push_str(&gezeigt.join(", "));
    if liste.len() > 6 {
        s.push_str(&format!(" … (+{})", liste.len() - 6));
    }
    s.push_str(&format!(". Zuletzt {}.", agg.letzte));
    s
}

/// Einfaches Muster: `*.ext`, `prefix*` oder exakter Name.
fn passt(muster: &str, name: &str) -> bool {
    if let Some(ext) = muster.strip_prefix('*') {
        name.ends_with(ext)
    } else if let Some(prefix) = muster.strip_suffix('*') {
        name.starts_with(prefix)
    } else {
        muster == name
    }
}

/// Wurzelordner dieses Geräts aus dem Speicher.
pub fn wurzeln_laden(store: &Store) -> Result<Vec<PathBuf>> {
    Ok(store
        .meta_get(META_WURZELN)?
        .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
        .unwrap_or_default()
        .into_iter()
        .map(PathBuf::from)
        .collect())
}

pub fn wurzeln_speichern(store: &Store, wurzeln: &[PathBuf]) -> Result<()> {
    let liste: Vec<String> = wurzeln
        .iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    store.meta_set(META_WURZELN, &serde_json::to_string(&liste)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Key32;
    use crate::model::{Projekt, Referenz, Rolle, Vorlage};

    #[test]
    fn verdichtet_aenderungen_zu_einer_tagesnotiz() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().canonicalize().unwrap();
        let projektdir = root.join("werkstatt");
        std::fs::create_dir_all(projektdir.join("src")).unwrap();
        std::fs::create_dir_all(projektdir.join("node_modules")).unwrap();

        let ak = Key32::random().unwrap();
        let mut s = Store::open_in_memory(&ak, Ulid::new()).unwrap();
        let p = Projekt::neu("Werkstatt", Vorlage::Software, now_ms());
        s.projekt_speichern(&p).unwrap();
        s.referenz_speichern(&Referenz::neu(
            p.id,
            ReferenzTyp::Ordner,
            projektdir.to_string_lossy(),
            Rolle::Material,
        ))
        .unwrap();

        let mut b = Beobachter::starten(&s, vec![root.clone()]).unwrap();
        std::thread::sleep(Duration::from_millis(200));
        std::fs::write(projektdir.join("src").join("main.rs"), "fn main(){}").unwrap();
        std::fs::write(projektdir.join("notizen.md"), "x").unwrap();
        std::fs::write(projektdir.join("node_modules").join("ignoriert.js"), "x").unwrap();
        std::fs::write(projektdir.join(".env"), "SECRET=1").unwrap();
        std::fs::write(root.join("fremd.txt"), "x").unwrap();
        b.verarbeiten(Duration::from_millis(600));

        let bilanz = b.schreiben(&mut s).unwrap();
        assert_eq!(bilanz.datei_notizen, 1);
        let notizen = s.notizen(p.id).unwrap();
        let datei: Vec<_> = notizen
            .iter()
            .filter(|n| n.quelle == Quelle::Datei)
            .collect();
        assert_eq!(datei.len(), 1);
        let text = &datei[0].text;
        assert!(text.contains("src/main.rs"), "{text}");
        assert!(text.contains("notizen.md"), "{text}");
        assert!(!text.contains("ignoriert"), "{text}");
        assert!(!text.contains(".env"), "{text}");
        assert!(!text.contains("fremd"), "{text}");

        // Weitere Änderung am selben Tag schreibt dieselbe Notiz fort.
        std::fs::write(projektdir.join("src").join("lib.rs"), "").unwrap();
        b.verarbeiten(Duration::from_millis(600));
        b.schreiben(&mut s).unwrap();
        let datei: Vec<_> = s
            .notizen(p.id)
            .unwrap()
            .into_iter()
            .filter(|n| n.quelle == Quelle::Datei)
            .collect();
        assert_eq!(datei.len(), 1);
        assert!(datei[0].text.contains("lib.rs"));
        assert!(datei[0].text.starts_with("3 Dateien"), "{}", datei[0].text);
    }

    #[test]
    fn text_und_muster() {
        let mut a = Aggregat::default();
        for f in ["a", "b", "c", "d", "e", "f", "g", "h"] {
            a.geaendert.insert(f.into());
        }
        a.neu = 2;
        a.letzte = "h".into();
        let t = text_fuer(&a);
        assert!(
            t.starts_with("8 Dateien geändert, 2 neu: a, b, c, d, e, f … (+2). Zuletzt h."),
            "{t}"
        );
        assert!(passt("*.pem", "server.pem"));
        assert!(passt("id_rsa*", "id_rsa.pub"));
        assert!(passt(".env", ".env"));
        assert!(!passt(".env", ".envrc"));
        assert!(!Beobachter::relevant(Path::new("target/debug/a")));
        assert!(!Beobachter::relevant(Path::new(".git/HEAD")));
        assert!(Beobachter::relevant(Path::new("src/main.rs")));
        assert!(!Beobachter::relevant(Path::new(".env")));
    }
}
