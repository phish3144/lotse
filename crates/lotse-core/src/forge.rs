//! Fernabfrage eines Git-Hosters, lesend.
//!
//! Ergänzt `git`, das nur den lokalen Log liest, um das, was auf der Gegenseite steht:
//! offene Pull Requests, Anzahl offener Issues, Zustand der Prüfläufe. Das beantwortet
//! „wo stehe ich" für ein Software-Projekt oft besser als der letzte Commit.
//!
//! Zwei Regeln, die dieses Modul einhält:
//!
//! 1. **Kein Tresor-Zugriff.** Wie `detect`, `git` und `watcher` importiert dieses Modul
//!    nie aus `vault`. Den Token reicht der Aufrufer herein; er entscheidet, woher er
//!    kommt (siehe `CLAUDE.md`).
//! 2. **Keine Aufgabenliste.** Issues werden gezählt und ein paar Titel genannt, aber
//!    nicht als offene Fäden angelegt. Tickets und Backlogs stehen in `NON_GOALS.md`
//!    dauerhaft auf der Nicht-Liste; eine verdichtete Zeile pro Tag ist das Muster,
//!    das sich beim Ordner-Beobachter bewährt hat.

use serde::Deserialize;
use ulid::Ulid;

use crate::model::{Art, Notiz, Quelle};
use crate::{Error, Result};

/// Unterstützte Hoster. GitLab folgt demselben Muster, ist aber noch nicht gebaut –
/// ungetesteter Code gegen eine Schnittstelle, die hier niemand ausprobieren kann,
/// wäre schlechter als keiner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anbieter {
    GitHub,
}

/// Zeiger auf ein Repository beim Hoster, aus einer Referenz gewonnen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoZeiger {
    pub anbieter: Anbieter,
    pub owner: String,
    pub repo: String,
}

impl RepoZeiger {
    /// Erkennt `git@github.com:owner/repo.git`, `https://github.com/owner/repo(.git)`
    /// und `ssh://git@github.com/owner/repo`. Alles andere ergibt `None` – das ist kein
    /// Fehler, sondern heißt nur: dazu gibt es keine Gegenseite.
    pub fn erkennen(ziel: &str) -> Option<RepoZeiger> {
        let z = ziel.trim();
        let rest = if let Some(r) = z.strip_prefix("git@github.com:") {
            r
        } else if let Some(r) = z.strip_prefix("https://github.com/") {
            r
        } else if let Some(r) = z.strip_prefix("http://github.com/") {
            r
        } else if let Some(r) = z.strip_prefix("ssh://git@github.com/") {
            r
        } else if let Some(r) = z.strip_prefix("github.com/") {
            r
        } else {
            return None;
        };
        let rest = rest.trim_start_matches('/').trim_end_matches('/');
        let rest = rest.strip_suffix(".git").unwrap_or(rest);
        let mut teile = rest.split('/');
        let owner = teile.next()?.trim();
        let repo = teile.next()?.trim();
        // Alles dahinter (z. B. /tree/main) gehört nicht zum Repo-Namen, stört aber nicht.
        if owner.is_empty() || repo.is_empty() {
            return None;
        }
        Some(RepoZeiger {
            anbieter: Anbieter::GitHub,
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    pub fn anzeige(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }
}

/// Zustand der Prüfläufe auf dem Standard-Branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ci {
    Gruen,
    Rot,
    Laeuft,
    /// Kein Prüflauf eingerichtet oder kein Ergebnis – ein ehrlicher Zustand, kein Fehler.
    Unbekannt,
}

impl Ci {
    pub fn as_str(self) -> &'static str {
        match self {
            Ci::Gruen => "grün",
            Ci::Rot => "rot",
            Ci::Laeuft => "läuft",
            Ci::Unbekannt => "unbekannt",
        }
    }
}

/// Momentaufnahme der Gegenseite.
#[derive(Debug, Clone)]
pub struct Stand {
    pub offene_issues: usize,
    pub offene_prs: usize,
    /// Titel der offenen Pull Requests, für die Notiz.
    pub pr_titel: Vec<String>,
    pub ci: Ci,
    pub standard_branch: String,
}

// --------------------------------------------------------------- Abfrage

#[derive(Deserialize)]
struct RepoWire {
    default_branch: String,
    /// Zählt bei GitHub Issues **und** Pull Requests zusammen.
    open_issues_count: usize,
}

#[derive(Deserialize)]
struct PrWire {
    title: String,
    draft: Option<bool>,
}

#[derive(Deserialize)]
struct StatusWire {
    state: String,
    /// Leer, wenn niemand einen Status gemeldet hat.
    statuses: Vec<serde_json::Value>,
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent(concat!("lotse/", env!("CARGO_PKG_VERSION")))
        .build()
}

fn hole<T: serde::de::DeserializeOwned>(
    agent: &ureq::Agent,
    url: &str,
    token: Option<&str>,
) -> Result<T> {
    let mut r = agent
        .get(url)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28");
    if let Some(t) = token {
        r = r.set("Authorization", &format!("Bearer {t}"));
    }
    match r.call() {
        Ok(resp) => resp
            .into_json::<T>()
            .map_err(|e| Error::Other(e.to_string())),
        Err(ureq::Error::Status(401, _)) => Err(Error::Invalid(
            "Der Token wird abgelehnt (401). Ist er abgelaufen?".into(),
        )),
        Err(ureq::Error::Status(403, resp)) => {
            // GitHub schickt 403 sowohl bei fehlenden Rechten als auch bei erschöpftem
            // Kontingent. Der Unterschied steht im Header.
            let rest = resp.header("x-ratelimit-remaining").unwrap_or("");
            if rest == "0" {
                Err(Error::Invalid(
                    "GitHub-Kontingent erschöpft. Mit Token sind es 5000 Anfragen pro \
                     Stunde statt 60."
                        .into(),
                ))
            } else {
                Err(Error::Invalid("Keine Berechtigung für dieses Repo".into()))
            }
        }
        Err(ureq::Error::Status(404, _)) => Err(Error::NotFound(
            "Repo nicht gefunden. Bei privaten Repos braucht es einen Token.".into(),
        )),
        Err(ureq::Error::Status(s, _)) => Err(Error::Netz(format!("GitHub antwortete {s}"))),
        Err(e) => Err(Error::Netz(e.to_string())),
    }
}

/// Setzt die drei Antworten zu einer Momentaufnahme zusammen. Getrennt vom Holen,
/// damit genau dieser Teil gegen echte GitHub-Antworten prüfbar ist, ohne Netz.
fn stand_aus(repo: RepoWire, prs: Vec<PrWire>, status: Option<StatusWire>) -> Stand {
    let ci = match status {
        // Kein gemeldeter Status heißt „nicht eingerichtet", nicht „läuft". GitHub
        // antwortet in dem Fall mit state = "pending" und leerer Liste.
        Some(s) if s.statuses.is_empty() => Ci::Unbekannt,
        Some(s) => match s.state.as_str() {
            "success" => Ci::Gruen,
            "failure" | "error" => Ci::Rot,
            "pending" => Ci::Laeuft,
            _ => Ci::Unbekannt,
        },
        None => Ci::Unbekannt,
    };

    let offene_prs = prs.len();
    Stand {
        // GitHub zählt Pull Requests bei den Issues mit; hier sollen sie getrennt stehen.
        offene_issues: repo.open_issues_count.saturating_sub(offene_prs),
        offene_prs,
        pr_titel: prs
            .into_iter()
            .map(|p| {
                if p.draft.unwrap_or(false) {
                    format!("{} (Entwurf)", p.title)
                } else {
                    p.title
                }
            })
            .collect(),
        ci,
        standard_branch: repo.default_branch,
    }
}

/// Fragt den Stand ab. Ohne Token gilt das kleine, IP-weite Kontingent von GitHub;
/// für private Repos ist er Pflicht.
pub fn abfragen(zeiger: &RepoZeiger, token: Option<&str>) -> Result<Stand> {
    let a = agent();
    let basis = format!(
        "https://api.github.com/repos/{}/{}",
        zeiger.owner, zeiger.repo
    );

    let repo: RepoWire = hole(&a, &basis, token)?;
    let prs: Vec<PrWire> = hole(&a, &format!("{basis}/pulls?state=open&per_page=20"), token)?;
    // Der Prüflauf-Status ist eine Bequemlichkeit, kein Muss: schlägt er fehl, bleibt
    // der Rest trotzdem nützlich.
    let status = hole::<StatusWire>(
        &a,
        &format!("{basis}/commits/{}/status", repo.default_branch),
        token,
    )
    .ok();

    Ok(stand_aus(repo, prs, status))
}

/// Eine verdichtete Zeile für das Logbuch. `None`, wenn es nichts zu sagen gibt –
/// eine Notiz „nichts offen, CI unbekannt" jeden Tag wäre Lärm.
pub fn notiz(projekt_id: Ulid, zeiger: &RepoZeiger, stand: &Stand, jetzt_ms: i64) -> Option<Notiz> {
    let erwaehnenswert = stand.offene_prs > 0 || stand.ci == Ci::Rot || stand.offene_issues > 0;
    if !erwaehnenswert {
        return None;
    }

    let mut zeilen = vec![format!("{}:", zeiger.anzeige())];
    if stand.ci == Ci::Rot {
        zeilen.push(format!("- Prüflauf auf {} ist rot.", stand.standard_branch));
    }
    if stand.offene_prs > 0 {
        zeilen.push(format!(
            "- {} offene Pull Request{}:",
            stand.offene_prs,
            if stand.offene_prs == 1 { "" } else { "s" }
        ));
        for t in stand.pr_titel.iter().take(5) {
            zeilen.push(format!("  - {t}"));
        }
        if stand.pr_titel.len() > 5 {
            zeilen.push(format!("  - … und {} weitere", stand.pr_titel.len() - 5));
        }
    }
    if stand.offene_issues > 0 {
        zeilen.push(format!(
            "- {} offene Issue{}.",
            stand.offene_issues,
            if stand.offene_issues == 1 { "" } else { "s" }
        ));
    }

    Some(Notiz::neu(
        projekt_id,
        Quelle::Git,
        Art::Log,
        zeilen.join("\n"),
        jetzt_ms,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erkennt_die_ueblichen_schreibweisen() {
        let erwartet = RepoZeiger {
            anbieter: Anbieter::GitHub,
            owner: "phish3144".into(),
            repo: "lotse".into(),
        };
        for ziel in [
            "git@github.com:phish3144/lotse.git",
            "https://github.com/phish3144/lotse",
            "https://github.com/phish3144/lotse.git",
            "https://github.com/phish3144/lotse/",
            "ssh://git@github.com/phish3144/lotse.git",
            "github.com/phish3144/lotse",
        ] {
            assert_eq!(
                RepoZeiger::erkennen(ziel).as_ref(),
                Some(&erwartet),
                "{ziel}"
            );
        }
    }

    #[test]
    fn erkennt_nichts_bei_fremden_zielen() {
        for ziel in [
            "/home/du/Projekte/lotse",
            "https://gitlab.com/du/lotse",
            "Keller, Regal 3",
            "https://github.com/",
            "https://github.com/nurowner",
            "",
        ] {
            assert!(RepoZeiger::erkennen(ziel).is_none(), "{ziel}");
        }
    }

    fn zeiger() -> RepoZeiger {
        RepoZeiger::erkennen("https://github.com/phish3144/lotse").unwrap()
    }

    fn daten(name: &str) -> String {
        std::fs::read_to_string(format!("{}/tests/daten/{name}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("{name}: {e}"))
    }

    /// `gh_repo.json` und `gh_status.json` sind unveränderte Antworten von
    /// api.github.com für phish3144/lotse. Sie belegen, dass die Feldnamen stimmen und
    /// dass unbekannte Felder – davon gibt es dutzende – das Deuten nicht stören.
    #[test]
    fn deutet_echte_github_antworten() {
        let repo: RepoWire = serde_json::from_str(&daten("gh_repo.json")).unwrap();
        let prs: Vec<PrWire> = serde_json::from_str(&daten("gh_pulls.json")).unwrap();
        let status: StatusWire = serde_json::from_str(&daten("gh_status.json")).unwrap();

        assert!(!repo.default_branch.is_empty());
        let stand = stand_aus(repo, prs, Some(status));
        // Zum Zeitpunkt der Aufnahme: keine offenen PRs, keine Issues, kein gemeldeter
        // Prüflauf-Status. Letzteres ist der Fall, den GitHub als "pending" mit leerer
        // Liste ausdrückt – „unbekannt“, nicht „läuft“.
        assert_eq!(stand.offene_prs, 0);
        assert_eq!(stand.ci, Ci::Unbekannt);
    }

    /// Der Fall mit offenen Pull Requests, den das eigene Repo gerade nicht hergibt.
    #[test]
    fn deutet_offene_pull_requests() {
        let repo: RepoWire = serde_json::from_str(&daten("gh_repo.json")).unwrap();
        let prs: Vec<PrWire> = serde_json::from_str(&daten("gh_pulls_beispiel.json")).unwrap();
        let stand = stand_aus(repo, prs, None);
        assert_eq!(stand.offene_prs, 2);
        assert_eq!(
            stand.pr_titel[0],
            "Sync-Client: Wiederverbinden nach Netzwechsel"
        );
        // Entwürfe werden als solche gekennzeichnet, sonst sieht man sie für fertig an.
        assert_eq!(stand.pr_titel[1], "Landing Page überarbeiten (Entwurf)");
        assert_eq!(stand.ci, Ci::Unbekannt);
    }

    #[test]
    fn zieht_pull_requests_von_der_issue_zahl_ab() {
        // GitHub zählt PRs bei open_issues_count mit. Ohne Abzug stünde im Logbuch eine
        // Issue-Zahl, die es so nicht gibt.
        let repo = RepoWire {
            default_branch: "main".into(),
            open_issues_count: 5,
        };
        let prs = vec![
            PrWire {
                title: "a".into(),
                draft: None,
            },
            PrWire {
                title: "b".into(),
                draft: None,
            },
        ];
        let stand = stand_aus(repo, prs, None);
        assert_eq!(stand.offene_issues, 3);
        assert_eq!(stand.offene_prs, 2);
    }

    #[test]
    fn mehr_prs_als_issues_ergibt_keine_negativzahl() {
        let repo = RepoWire {
            default_branch: "main".into(),
            open_issues_count: 1,
        };
        let prs = vec![
            PrWire {
                title: "a".into(),
                draft: None,
            },
            PrWire {
                title: "b".into(),
                draft: None,
            },
        ];
        assert_eq!(stand_aus(repo, prs, None).offene_issues, 0);
    }

    #[test]
    fn schweigt_wenn_es_nichts_zu_sagen_gibt() {
        let stand = Stand {
            offene_issues: 0,
            offene_prs: 0,
            pr_titel: vec![],
            ci: Ci::Gruen,
            standard_branch: "main".into(),
        };
        assert!(notiz(Ulid::new(), &zeiger(), &stand, 1).is_none());
    }

    #[test]
    fn nennt_roten_prueflauf_und_pull_requests() {
        let stand = Stand {
            offene_issues: 3,
            offene_prs: 2,
            pr_titel: vec!["Sync-Client".into(), "Landing Page (Entwurf)".into()],
            ci: Ci::Rot,
            standard_branch: "main".into(),
        };
        let n = notiz(Ulid::new(), &zeiger(), &stand, 42).unwrap();
        assert_eq!(n.quelle, Quelle::Git);
        assert_eq!(n.art, Art::Log);
        assert!(n.text.contains("phish3144/lotse"));
        assert!(n.text.contains("rot"));
        assert!(n.text.contains("2 offene Pull Requests"));
        assert!(n.text.contains("Sync-Client"));
        assert!(n.text.contains("3 offene Issues"));
    }

    #[test]
    fn kuerzt_lange_pr_listen() {
        let stand = Stand {
            offene_issues: 0,
            offene_prs: 7,
            pr_titel: (1..=7).map(|i| format!("PR {i}")).collect(),
            ci: Ci::Gruen,
            standard_branch: "main".into(),
        };
        let n = notiz(Ulid::new(), &zeiger(), &stand, 1).unwrap();
        assert!(n.text.contains("… und 2 weitere"));
        assert!(!n.text.contains("PR 6"));
    }

    #[test]
    fn einzahl_und_mehrzahl() {
        let stand = Stand {
            offene_issues: 1,
            offene_prs: 1,
            pr_titel: vec!["Eins".into()],
            ci: Ci::Unbekannt,
            standard_branch: "main".into(),
        };
        let n = notiz(Ulid::new(), &zeiger(), &stand, 1).unwrap();
        assert!(
            n.text.contains("1 offener Pull Request") || n.text.contains("1 offene Pull Request")
        );
        assert!(n.text.contains("1 offene Issue"));
    }
}

/// Läuft nicht in CI: braucht Netz und verbraucht vom GitHub-Kontingent.
/// Von Hand: `cargo test -p lotse-core forge_live -- --ignored --nocapture`
#[cfg(test)]
mod live {
    use super::*;

    #[test]
    #[ignore]
    fn forge_live_gegen_echtes_repo() {
        let zeiger = RepoZeiger::erkennen("https://github.com/phish3144/lotse").unwrap();
        let token = std::env::var("GITHUB_TOKEN").ok();
        let stand = abfragen(&zeiger, token.as_deref()).expect("Abfrage");
        println!(
            "{}: {} Issues, {} PRs, Prüflauf {} auf {}",
            zeiger.anzeige(),
            stand.offene_issues,
            stand.offene_prs,
            stand.ci.as_str(),
            stand.standard_branch
        );
        assert!(!stand.standard_branch.is_empty());
    }
}
