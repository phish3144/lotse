//! Lokaler Git-Log-Reader: `git log` als Subprozess. Kein libgit2, kein Auth, kein Netz.
//!
//! Liefert Commits als verdichtete Logbuch-Notizen (eine pro Tag), damit ein Logbuch nicht
//! von 200 Commit-Zeilen geflutet wird.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use ulid::Ulid;

use crate::model::{Art, Notiz, Quelle};
use crate::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    pub hash: String,
    pub ts_ms: i64,
    pub betreff: String,
}

/// Liest Commits eines Repos, optional nur seit einem Zeitpunkt. `limit` begrenzt die Menge.
pub fn log(repo: &Path, seit_ms: Option<i64>, limit: usize) -> Result<Vec<Commit>> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(repo)
        .arg("log")
        .arg("--no-merges")
        .arg("--format=%H%x1f%ct%x1f%s")
        .arg(format!("-n{limit}"));
    if let Some(ms) = seit_ms {
        cmd.arg(format!("--since={}", ms / 1000));
    }
    let out = match cmd.output() {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()), // kein git installiert: still, kein Fehler
    };
    if !out.status.success() {
        return Ok(Vec::new());
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut commits = Vec::new();
    for line in text.lines() {
        let mut teile = line.split('\u{1f}');
        let (Some(hash), Some(ts), Some(betreff)) = (teile.next(), teile.next(), teile.next())
        else {
            continue;
        };
        let Ok(ts): std::result::Result<i64, _> = ts.parse() else {
            continue;
        };
        commits.push(Commit {
            hash: hash.to_string(),
            ts_ms: ts * 1000,
            betreff: betreff.to_string(),
        });
    }
    Ok(commits)
}

/// Verdichtet Commits zu einer Notiz pro Tag (UTC), rückdatiert auf den letzten Commit
/// des Tages.
pub fn verdichten(projekt_id: Ulid, commits: &[Commit], quelle: Quelle) -> Vec<Notiz> {
    let mut nach_tag: BTreeMap<i64, Vec<&Commit>> = BTreeMap::new();
    for c in commits {
        nach_tag
            .entry(c.ts_ms / crate::brief::MS_PRO_TAG)
            .or_default()
            .push(c);
    }
    let mut out = Vec::new();
    for (_, mut cs) in nach_tag {
        cs.sort_by_key(|c| c.ts_ms);
        let letzter = cs.last().map(|c| c.ts_ms).unwrap_or(0);
        let text = if cs.len() == 1 {
            format!("Commit: {}", cs[0].betreff)
        } else {
            let mut s = format!("{} Commits:\n", cs.len());
            for c in cs.iter().take(8) {
                s.push_str("- ");
                s.push_str(&c.betreff);
                s.push('\n');
            }
            if cs.len() > 8 {
                s.push_str(&format!("- … und {} weitere\n", cs.len() - 8));
            }
            s.trim_end().to_string()
        };
        out.push(Notiz::neu(projekt_id, quelle, Art::Log, text, letzter));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdichtet_pro_tag() {
        let tag = crate::brief::MS_PRO_TAG;
        let cs = vec![
            Commit {
                hash: "a".into(),
                ts_ms: tag + 10,
                betreff: "eins".into(),
            },
            Commit {
                hash: "b".into(),
                ts_ms: tag + 20,
                betreff: "zwei".into(),
            },
            Commit {
                hash: "c".into(),
                ts_ms: 3 * tag,
                betreff: "drei".into(),
            },
        ];
        let n = verdichten(Ulid::new(), &cs, Quelle::Import);
        assert_eq!(n.len(), 2);
        assert!(n[0].text.starts_with("2 Commits"));
        assert_eq!(n[0].ts, tag + 20);
        assert_eq!(n[1].text, "Commit: drei");
    }

    #[test]
    fn log_in_echtem_repo() {
        let t = tempfile::tempdir().unwrap();
        let run = |args: &[&str]| {
            Command::new("git")
                .arg("-C")
                .arg(t.path())
                .args(args)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@t")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@t")
                .output()
                .unwrap()
        };
        if !run(&["init", "-q"]).status.success() {
            return; // kein git in dieser Umgebung
        }
        std::fs::write(t.path().join("a"), "1").unwrap();
        run(&["add", "."]);
        run(&["commit", "-q", "-m", "erster Wurf"]);
        let cs = log(t.path(), None, 50).unwrap();
        assert_eq!(cs.len(), 1);
        assert_eq!(cs[0].betreff, "erster Wurf");
        assert!(log(t.path(), Some(crate::now_ms() + 60_000), 50)
            .unwrap()
            .is_empty());
    }
}
