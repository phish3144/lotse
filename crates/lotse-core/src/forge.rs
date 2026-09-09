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

/// Unterstützte Hoster. Beide werden gegen echte, aufgezeichnete Antworten geprüft
/// (`tests/daten/`); geraten wird an keiner Stelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anbieter {
    GitHub,
    GitLab,
}

impl Anbieter {
    pub fn as_str(self) -> &'static str {
        match self {
            Anbieter::GitHub => "GitHub",
            Anbieter::GitLab => "GitLab",
        }
    }

    fn host(self) -> &'static str {
        match self {
            Anbieter::GitHub => "github.com",
            Anbieter::GitLab => "gitlab.com",
        }
    }
}

/// Zeiger auf ein Repository beim Hoster, aus einer Referenz gewonnen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoZeiger {
    pub anbieter: Anbieter,
    /// Bei GitLab kann das eine verschachtelte Gruppe sein (`gruppe/untergruppe`).
    pub owner: String,
    pub repo: String,
}

/// Schneidet Host und Schema ab und sagt, wer die Gegenseite ist. Schema und Host
/// werden ohne Rücksicht auf Groß- und Kleinschreibung verglichen – `GitHub.com` ist
/// dieselbe Adresse wie `github.com`, der Rest des Pfades bleibt, wie er ist.
fn wegweiser(z: &str) -> Option<(Anbieter, &str)> {
    let klein = z.to_ascii_lowercase();
    for anbieter in [Anbieter::GitHub, Anbieter::GitLab] {
        let host = anbieter.host();
        for muster in [
            format!("git@{host}:"),
            format!("https://{host}/"),
            format!("http://{host}/"),
            format!("ssh://git@{host}/"),
            format!("{host}/"),
        ] {
            if klein.starts_with(&muster) {
                return Some((anbieter, &z[muster.len()..]));
            }
        }
    }
    None
}

impl RepoZeiger {
    /// Erkennt `git@github.com:owner/repo.git`, `https://github.com/owner/repo(.git)`,
    /// `ssh://git@github.com/owner/repo` und dieselben Schreibweisen für gitlab.com,
    /// dort auch mit verschachtelten Gruppen. Alles andere ergibt `None` – das ist kein
    /// Fehler, sondern heißt nur: dazu gibt es keine Gegenseite.
    pub fn erkennen(ziel: &str) -> Option<RepoZeiger> {
        let (anbieter, rest) = wegweiser(ziel.trim())?;
        // Aus dem Browser kopierte Adressen tragen oft einen Anker oder Parameter mit;
        // beides gehört nicht zum Repo-Namen.
        let rest = rest.split(['?', '#']).next().unwrap_or(rest);
        // GitLab hängt seine Weboberfläche hinter `/-/` an: `.../repo/-/issues/3`.
        let rest = rest.split("/-/").next().unwrap_or(rest);
        let rest = rest.trim_start_matches('/').trim_end_matches('/');
        let rest = rest.strip_suffix(".git").unwrap_or(rest);
        let teile: Vec<&str> = rest.split('/').filter(|t| !t.is_empty()).collect();
        if teile.len() < 2 {
            return None;
        }
        let (owner, repo) = match anbieter {
            // Alles hinter owner/repo (z. B. /tree/main) gehört nicht zum Namen.
            Anbieter::GitHub => (teile[0].to_string(), teile[1].to_string()),
            // Bei GitLab ist alles bis auf das letzte Stück der Namensraum.
            Anbieter::GitLab => (
                teile[..teile.len() - 1].join("/"),
                teile[teile.len() - 1].to_string(),
            ),
        };
        if owner.is_empty() || repo.is_empty() {
            return None;
        }
        Some(RepoZeiger {
            anbieter,
            owner,
            repo,
        })
    }

    /// Erkennt das Repo eines lokalen Ordners an seinen Git-Remotes, `origin` zuerst.
    /// Damit genügt eine Ordner-Referenz; niemand muss die Adresse von Hand eintragen.
    /// Kein Netz, kein Tresor: nur `git remote -v`.
    pub fn aus_ordner(pfad: &std::path::Path) -> Option<RepoZeiger> {
        crate::git::remotes(pfad)
            .iter()
            .find_map(|u| RepoZeiger::erkennen(u))
    }

    pub fn anzeige(&self) -> String {
        format!("{}/{}", self.owner, self.repo)
    }

    /// Adresse zum Öffnen im Browser.
    pub fn web_url(&self) -> String {
        format!(
            "https://{}/{}/{}",
            self.anbieter.host(),
            self.owner,
            self.repo
        )
    }
}

/// Woher das Repo bekannt ist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Herkunft {
    /// Aus einer eingetragenen Adresse (Referenz vom Typ URL oder Git-Repo).
    Adresse,
    /// Aus dem Git-Remote eines Ordners, der auf diesem Gerät liegt.
    Ordner,
}

impl Herkunft {
    pub fn as_str(self) -> &'static str {
        match self {
            Herkunft::Adresse => "adresse",
            Herkunft::Ordner => "ordner",
        }
    }
}

/// Findet zu den Referenzen eines Projekts das Repo auf der Gegenseite: erst eine
/// eingetragene Adresse, sonst das Git-Remote eines Ordners, der auf diesem Gerät gilt.
///
/// Ruft `git` auf, hält aber nichts: Desktop und CLI rufen die Funktion außerhalb
/// jeder Sperre auf, damit die Oberfläche nicht ansteht.
pub fn zeiger_aus_referenzen(
    referenzen: &[crate::model::Referenz],
    geraet: Ulid,
) -> Option<(RepoZeiger, Herkunft)> {
    use crate::model::ReferenzTyp;
    if let Some(z) = referenzen
        .iter()
        .find_map(|r| RepoZeiger::erkennen(&r.ziel))
    {
        return Some((z, Herkunft::Adresse));
    }
    referenzen
        .iter()
        .filter(|r| {
            matches!(r.typ, ReferenzTyp::Ordner | ReferenzTyp::GitRepo)
                && r.geraet_id.is_none_or(|g| g == geraet)
        })
        .find_map(|r| RepoZeiger::aus_ordner(std::path::Path::new(&r.ziel)))
        .map(|z| (z, Herkunft::Ordner))
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
    /// Die Liste war voll: es sind mindestens so viele, womöglich mehr. Ohne diese
    /// Markierung stünde eine zu kleine Zahl im Logbuch, die wie eine genaue aussieht.
    pub mehr_prs: bool,
    /// Titel der offenen Pull Requests, für die Notiz.
    pub pr_titel: Vec<String>,
    pub ci: Ci,
    pub standard_branch: String,
}

// --------------------------------------------------------------- Abfrage

/// So viele offene Pull- bzw. Merge-Requests werden geholt. Mehr wäre für eine Zeile im
/// Logbuch sinnlos; dass es mehr sein können, sagt `Stand::mehr_prs`.
const SEITE: usize = 100;

/// Ein Agent für alle Abfragen dieses Moduls.
///
/// `http_status_as_error(false)`: Fehlerantworten kommen als gewöhnliche Antwort zurück,
/// nicht als Fehlerwert. Nur so lassen sich ihre Kopfzeilen lesen – bei GitHub steckt im
/// 403 der Unterschied zwischen „keine Berechtigung" und „Kontingent erschöpft".
///
/// TLS prüft gegen den Wurzelspeicher des Systems (`THREAT_MODEL.md` 6a).
pub(crate) fn agent() -> ureq::Agent {
    crate::netz::agent(std::time::Duration::from_secs(20))
}

/// Ein GET mit den Kopfzeilen des jeweiligen Hosters. `token` trägt die Anmeldung: GitHub
/// nimmt `Authorization: Bearer`, GitLab `PRIVATE-TOKEN`.
pub(crate) fn hole<T: serde::de::DeserializeOwned>(
    agent: &ureq::Agent,
    url: &str,
    anbieter: Anbieter,
    token: Option<&str>,
) -> Result<T> {
    let mut r = agent.get(url).header("Accept", "application/json");
    if anbieter == Anbieter::GitHub {
        r = r
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28");
    }
    if let Some(t) = token {
        r = match anbieter {
            Anbieter::GitHub => r.header("Authorization", &format!("Bearer {t}")),
            Anbieter::GitLab => r.header("PRIVATE-TOKEN", t),
        };
    }
    let name = anbieter.as_str();
    let mut resp = r.call().map_err(|e| crate::netz::fehler(e, name))?;
    let status = resp.status().as_u16();
    if status == 403 {
        // GitHub schickt 403 sowohl bei fehlenden Rechten als auch bei erschöpftem
        // Kontingent. Der Unterschied steht im Header.
        let rest = resp
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        return Err(if rest == "0" {
            Error::Invalid(
                "GitHub-Kontingent erschöpft. Mit Token sind es 5000 Anfragen pro \
                 Stunde statt 60."
                    .into(),
            )
        } else {
            Error::Invalid("Keine Berechtigung für dieses Repo".into())
        });
    }
    match status {
        200..=299 => resp
            .body_mut()
            .read_json::<T>()
            .map_err(|e| Error::Other(e.to_string())),
        401 => Err(Error::Invalid(
            "Der Token wird abgelehnt (401). Ist er abgelaufen?".into(),
        )),
        // GitLab drosselt mit 429 statt mit 403.
        429 => Err(Error::Invalid(format!(
            "{name} drosselt gerade (429). Später noch einmal."
        ))),
        404 => Err(Error::NotFound(
            "Repo nicht gefunden. Bei privaten Repos braucht es einen Token.".into(),
        )),
        s => Err(Error::Netz(format!("{name} antwortete {s}"))),
    }
}

/// GitHub: ein Aufruf für das Repo, einer für die Pull Requests, einer für den Prüflauf.
mod github {
    use super::*;

    #[derive(Deserialize)]
    pub struct RepoWire {
        pub default_branch: String,
        /// Zählt bei GitHub Issues **und** Pull Requests zusammen.
        pub open_issues_count: usize,
    }

    #[derive(Deserialize)]
    pub struct PrWire {
        pub title: String,
        pub draft: Option<bool>,
    }

    #[derive(Deserialize)]
    pub struct StatusWire {
        pub state: String,
        /// Leer, wenn niemand einen Status gemeldet hat.
        pub statuses: Vec<serde_json::Value>,
    }

    /// Setzt die drei Antworten zu einer Momentaufnahme zusammen. Getrennt vom Holen,
    /// damit genau dieser Teil gegen echte Antworten prüfbar ist, ohne Netz.
    pub fn stand_aus(repo: RepoWire, prs: Vec<PrWire>, status: Option<StatusWire>) -> Stand {
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
            mehr_prs: offene_prs >= super::SEITE,
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

    pub fn abfragen(a: &ureq::Agent, z: &RepoZeiger, token: Option<&str>) -> Result<Stand> {
        let basis = format!("https://api.github.com/repos/{}/{}", z.owner, z.repo);
        let anbieter = Anbieter::GitHub;
        let repo: RepoWire = hole(a, &basis, anbieter, token)?;
        let prs: Vec<PrWire> = hole(
            a,
            &format!("{basis}/pulls?state=open&per_page={}", super::SEITE),
            anbieter,
            token,
        )?;
        // Der Prüflauf-Status ist eine Bequemlichkeit, kein Muss: schlägt er fehl, bleibt
        // der Rest trotzdem nützlich.
        let status = hole::<StatusWire>(
            a,
            &format!("{basis}/commits/{}/status", repo.default_branch),
            anbieter,
            token,
        )
        .ok();
        Ok(stand_aus(repo, prs, status))
    }
}

/// GitLab: dieselbe Frage, vier Aufrufe. Die Zahl offener Issues steht nicht im
/// Projekt-Datensatz (jedenfalls nicht ohne Anmeldung), sondern in `issues_statistics`.
mod gitlab {
    use super::*;

    #[derive(Deserialize)]
    pub struct ProjektWire {
        pub default_branch: String,
    }

    #[derive(Deserialize)]
    pub struct MrWire {
        pub title: String,
        pub draft: Option<bool>,
        /// Ältere GitLab-Stände nennen es so.
        pub work_in_progress: Option<bool>,
    }

    #[derive(Deserialize)]
    pub struct IssuesWire {
        pub statistics: IssuesStatistik,
    }

    #[derive(Deserialize)]
    pub struct IssuesStatistik {
        pub counts: IssuesZahlen,
    }

    #[derive(Deserialize)]
    pub struct IssuesZahlen {
        pub opened: usize,
    }

    #[derive(Deserialize)]
    pub struct PipelineWire {
        pub status: String,
    }

    /// Pfad mit Namensraum, wie GitLab ihn in der API erwartet: `gruppe%2Frepo`.
    pub fn pfad(z: &RepoZeiger) -> String {
        format!("{}/{}", z.owner, z.repo).replace('/', "%2F")
    }

    pub fn stand_aus(
        projekt: ProjektWire,
        mrs: Vec<MrWire>,
        issues: Option<IssuesWire>,
        pipelines: &[PipelineWire],
    ) -> Stand {
        let ci = match pipelines.first().map(|p| p.status.as_str()) {
            Some("success") => Ci::Gruen,
            Some("failed") => Ci::Rot,
            Some(
                "running"
                | "pending"
                | "created"
                | "preparing"
                | "scheduled"
                | "waiting_for_resource",
            ) => Ci::Laeuft,
            // canceled, skipped, manual und alles Unbekannte: kein Urteil.
            _ => Ci::Unbekannt,
        };
        Stand {
            // GitLab zählt Merge Requests **nicht** bei den Issues mit; kein Abzug.
            offene_issues: issues.map(|i| i.statistics.counts.opened).unwrap_or(0),
            offene_prs: mrs.len(),
            mehr_prs: mrs.len() >= super::SEITE,
            pr_titel: mrs
                .into_iter()
                .map(|m| {
                    if m.draft.or(m.work_in_progress).unwrap_or(false) {
                        format!("{} (Entwurf)", m.title)
                    } else {
                        m.title
                    }
                })
                .collect(),
            ci,
            standard_branch: projekt.default_branch,
        }
    }

    pub fn abfragen(a: &ureq::Agent, z: &RepoZeiger, token: Option<&str>) -> Result<Stand> {
        let basis = format!("https://gitlab.com/api/v4/projects/{}", pfad(z));
        let anbieter = Anbieter::GitLab;
        let projekt: ProjektWire = hole(a, &basis, anbieter, token)?;
        let mrs: Vec<MrWire> = hole(
            a,
            &format!(
                "{basis}/merge_requests?state=opened&per_page={}",
                super::SEITE
            ),
            anbieter,
            token,
        )?;
        // Issue-Zahl und Prüflauf sind Beiwerk: fehlen sie, bleibt der Rest nützlich.
        let issues =
            hole::<IssuesWire>(a, &format!("{basis}/issues_statistics"), anbieter, token).ok();
        let pipelines = hole::<Vec<PipelineWire>>(
            a,
            &format!(
                "{basis}/pipelines?ref={}&per_page=1",
                projekt.default_branch
            ),
            anbieter,
            token,
        )
        .unwrap_or_default();
        Ok(stand_aus(projekt, mrs, issues, &pipelines))
    }
}

/// Fragt den Stand ab. Ohne Token gilt das kleine, IP-weite Kontingent des Hosters;
/// für private Repos ist er Pflicht.
pub fn abfragen(zeiger: &RepoZeiger, token: Option<&str>) -> Result<Stand> {
    let a = agent();
    match zeiger.anbieter {
        Anbieter::GitHub => github::abfragen(&a, zeiger, token),
        Anbieter::GitLab => gitlab::abfragen(&a, zeiger, token),
    }
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
        // GitLab nennt sie Merge Requests. Wer beides nutzt, will die eigene Sprache lesen.
        let art = match zeiger.anbieter {
            Anbieter::GitHub => "Pull Request",
            Anbieter::GitLab => "Merge Request",
        };
        zeilen.push(if stand.offene_prs == 1 {
            format!("- 1 offener {art}:")
        } else if stand.mehr_prs {
            format!("- mindestens {} offene {art}s:", stand.offene_prs)
        } else {
            format!("- {} offene {art}s:", stand.offene_prs)
        });
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

    use super::github::{PrWire, RepoWire, StatusWire};
    use super::gitlab::{
        IssuesStatistik, IssuesWire, IssuesZahlen, MrWire, PipelineWire, ProjektWire,
    };

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
            "https://bitbucket.org/du/lotse",
            "Keller, Regal 3",
            "https://github.com/",
            "https://github.com/nurowner",
            "https://gitlab.com/nurgruppe",
            "",
        ] {
            assert!(RepoZeiger::erkennen(ziel).is_none(), "{ziel}");
        }
    }

    #[test]
    fn erkennt_gitlab_mit_verschachtelten_gruppen() {
        for (ziel, owner, repo) in [
            ("https://gitlab.com/du/lotse", "du", "lotse"),
            (
                "git@gitlab.com:gitlab-org/gitlab-runner.git",
                "gitlab-org",
                "gitlab-runner",
            ),
            (
                "https://gitlab.com/gruppe/untergruppe/lotse",
                "gruppe/untergruppe",
                "lotse",
            ),
            // Die Weboberfläche hängt hinter `/-/` an; das gehört nicht zum Namen.
            (
                "https://gitlab.com/gruppe/lotse/-/merge_requests/7",
                "gruppe",
                "lotse",
            ),
        ] {
            let z = RepoZeiger::erkennen(ziel).unwrap_or_else(|| panic!("{ziel}"));
            assert_eq!(z.anbieter, Anbieter::GitLab, "{ziel}");
            assert_eq!(z.owner, owner, "{ziel}");
            assert_eq!(z.repo, repo, "{ziel}");
        }
        assert_eq!(
            RepoZeiger::erkennen("https://gitlab.com/gruppe/untergruppe/lotse")
                .unwrap()
                .web_url(),
            "https://gitlab.com/gruppe/untergruppe/lotse"
        );
    }

    #[test]
    fn erkennt_das_repo_am_ordner() {
        let t = tempfile::tempdir().unwrap();
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(t.path())
                .args(args)
                .output()
                .unwrap()
        };
        if !run(&["init", "-q"]).status.success() {
            return; // kein git in dieser Umgebung
        }
        // Ohne Remote gibt es keine Gegenseite – und das ist kein Fehler.
        assert!(RepoZeiger::aus_ordner(t.path()).is_none());

        run(&[
            "remote",
            "add",
            "origin",
            "https://github.com/phish3144/lotse.git",
        ]);
        let z = RepoZeiger::aus_ordner(t.path()).expect("origin");
        assert_eq!(z.anzeige(), "phish3144/lotse");
    }

    #[test]
    fn ueberspringt_remotes_ohne_gegenseite() {
        let t = tempfile::tempdir().unwrap();
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(t.path())
                .args(args)
                .output()
                .unwrap()
        };
        if !run(&["init", "-q"]).status.success() {
            return;
        }
        // origin zeigt auf einen eigenen Server, ein weiteres Remote auf GitHub:
        // dann zählt das GitHub-Remote, nicht die Reihenfolge.
        run(&[
            "remote",
            "add",
            "origin",
            "https://git.example.invalid/lotse.git",
        ]);
        run(&[
            "remote",
            "add",
            "gh",
            "https://github.com/phish3144/lotse.git",
        ]);
        assert_eq!(
            RepoZeiger::aus_ordner(t.path()).map(|z| z.anzeige()),
            Some("phish3144/lotse".to_string())
        );
    }

    #[test]
    fn adresse_schlaegt_ordner() {
        use crate::model::{Referenz, ReferenzTyp, Rolle};
        let projekt = Ulid::new();
        let geraet = Ulid::new();
        let refs = vec![
            Referenz::neu(
                projekt,
                ReferenzTyp::Ordner,
                "/pfad/ohne/repo",
                Rolle::Material,
            ),
            Referenz::neu(
                projekt,
                ReferenzTyp::Url,
                "https://github.com/phish3144/lotse",
                Rolle::Doku,
            ),
        ];
        let (z, h) = zeiger_aus_referenzen(&refs, geraet).unwrap();
        assert_eq!(z.anzeige(), "phish3144/lotse");
        assert_eq!(h, Herkunft::Adresse);

        // Ordner eines fremden Geräts werden nicht angefasst: der Pfad gilt hier nicht.
        let mut fremd = Referenz::neu(
            projekt,
            ReferenzTyp::Ordner,
            "/pfad/vom/anderen/gerät",
            Rolle::Material,
        );
        fremd.geraet_id = Some(Ulid::new());
        assert!(zeiger_aus_referenzen(&[fremd], geraet).is_none());
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
        let stand = github::stand_aus(repo, prs, Some(status));
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
        let stand = github::stand_aus(repo, prs, None);
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
        let stand = github::stand_aus(repo, prs, None);
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
        assert_eq!(github::stand_aus(repo, prs, None).offene_issues, 0);
    }

    /// `gl_projekt.json`, `gl_merge_requests.json`, `gl_issues_statistics.json` und
    /// `gl_pipelines.json` sind Antworten von gitlab.com für gitlab-org/gitlab-runner,
    /// nur die Liste der Merge Requests ist auf zwei Einträge gekürzt.
    #[test]
    fn deutet_echte_gitlab_antworten() {
        let projekt: ProjektWire = serde_json::from_str(&daten("gl_projekt.json")).unwrap();
        let mrs: Vec<MrWire> = serde_json::from_str(&daten("gl_merge_requests.json")).unwrap();
        let issues: IssuesWire = serde_json::from_str(&daten("gl_issues_statistics.json")).unwrap();
        let pipelines: Vec<PipelineWire> =
            serde_json::from_str(&daten("gl_pipelines.json")).unwrap();

        assert_eq!(projekt.default_branch, "main");
        let stand = gitlab::stand_aus(projekt, mrs, Some(issues), &pipelines);
        assert_eq!(stand.offene_prs, 2);
        // Bei GitLab zählen Merge Requests nicht als Issues: die Zahl bleibt, wie sie ist.
        assert_eq!(stand.offene_issues, 2605);
        assert_eq!(stand.ci, Ci::Laeuft);
        assert!(stand.pr_titel[0].starts_with("Fix after_script"));
    }

    #[test]
    fn deutet_gitlab_pipeline_zustaende() {
        let fall = |status: &str| {
            gitlab::stand_aus(
                ProjektWire {
                    default_branch: "main".into(),
                },
                vec![],
                None,
                &[PipelineWire {
                    status: status.into(),
                }],
            )
            .ci
        };
        assert_eq!(fall("success"), Ci::Gruen);
        assert_eq!(fall("failed"), Ci::Rot);
        assert_eq!(fall("running"), Ci::Laeuft);
        assert_eq!(fall("canceled"), Ci::Unbekannt);
        // Ohne Pipeline gibt es kein Urteil, keinen Fehler.
        let ohne = gitlab::stand_aus(
            ProjektWire {
                default_branch: "main".into(),
            },
            vec![],
            None,
            &[],
        );
        assert_eq!(ohne.ci, Ci::Unbekannt);
        assert_eq!(ohne.offene_issues, 0);
    }

    #[test]
    fn kennzeichnet_gitlab_entwuerfe_in_beiden_schreibweisen() {
        let stand = gitlab::stand_aus(
            ProjektWire {
                default_branch: "main".into(),
            },
            vec![
                MrWire {
                    title: "Neu".into(),
                    draft: Some(true),
                    work_in_progress: None,
                },
                MrWire {
                    title: "Alt".into(),
                    draft: None,
                    work_in_progress: Some(true),
                },
                MrWire {
                    title: "Fertig".into(),
                    draft: Some(false),
                    work_in_progress: Some(false),
                },
            ],
            Some(IssuesWire {
                statistics: IssuesStatistik {
                    counts: IssuesZahlen { opened: 1 },
                },
            }),
            &[],
        );
        assert_eq!(stand.pr_titel[0], "Neu (Entwurf)");
        assert_eq!(stand.pr_titel[1], "Alt (Entwurf)");
        assert_eq!(stand.pr_titel[2], "Fertig");
    }

    #[test]
    fn kodiert_den_gitlab_pfad() {
        let z = RepoZeiger::erkennen("https://gitlab.com/gruppe/untergruppe/lotse").unwrap();
        assert_eq!(gitlab::pfad(&z), "gruppe%2Funtergruppe%2Flotse");
    }

    #[test]
    fn erkennt_adressen_unabhaengig_von_der_schreibweise() {
        for ziel in [
            "https://GitHub.com/phish3144/lotse",
            "HTTPS://GITHUB.COM/phish3144/lotse",
            "git@GitHub.com:phish3144/lotse.git",
        ] {
            let z = RepoZeiger::erkennen(ziel).unwrap_or_else(|| panic!("{ziel}"));
            // Der Host ist unempfindlich, der Repo-Name nicht: GitHub unterscheidet dort
            // Groß- und Kleinschreibung nicht, zeigt sie aber so an, wie sie steht.
            assert_eq!(z.anzeige(), "phish3144/lotse", "{ziel}");
        }
    }

    #[test]
    fn wirft_anker_und_parameter_weg() {
        for ziel in [
            "https://github.com/phish3144/lotse#readme",
            "https://github.com/phish3144/lotse?utm_source=x",
            "https://github.com/phish3144/lotse.git?ref=1",
            "https://github.com/phish3144/lotse/tree/main",
        ] {
            assert_eq!(
                RepoZeiger::erkennen(ziel).map(|z| z.anzeige()).as_deref(),
                Some("phish3144/lotse"),
                "{ziel}"
            );
        }
        assert_eq!(
            RepoZeiger::erkennen("https://gitlab.com/gruppe/lotse?ref=1")
                .map(|z| z.anzeige())
                .as_deref(),
            Some("gruppe/lotse")
        );
    }

    #[test]
    fn sagt_mindestens_wenn_die_liste_voll_war() {
        let repo = RepoWire {
            default_branch: "main".into(),
            open_issues_count: 200,
        };
        let prs: Vec<PrWire> = (0..100)
            .map(|i| PrWire {
                title: format!("PR {i}"),
                draft: None,
            })
            .collect();
        let stand = github::stand_aus(repo, prs, None);
        assert!(stand.mehr_prs);
        let n = notiz(Ulid::new(), &zeiger(), &stand, 1).unwrap();
        assert!(
            n.text.contains("mindestens 100 offene Pull Requests"),
            "{}",
            n.text
        );
    }

    #[test]
    fn schweigt_wenn_es_nichts_zu_sagen_gibt() {
        let stand = Stand {
            offene_issues: 0,
            offene_prs: 0,
            mehr_prs: false,
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
            mehr_prs: false,
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
            mehr_prs: false,
            pr_titel: (1..=7).map(|i| format!("PR {i}")).collect(),
            ci: Ci::Gruen,
            standard_branch: "main".into(),
        };
        let n = notiz(Ulid::new(), &zeiger(), &stand, 1).unwrap();
        assert!(n.text.contains("… und 2 weitere"));
        assert!(!n.text.contains("PR 6"));
    }

    #[test]
    fn nennt_bei_gitlab_merge_requests() {
        let z = RepoZeiger::erkennen("https://gitlab.com/gruppe/lotse").unwrap();
        let stand = Stand {
            offene_issues: 0,
            offene_prs: 2,
            mehr_prs: false,
            pr_titel: vec!["Eins".into(), "Zwei".into()],
            ci: Ci::Gruen,
            standard_branch: "main".into(),
        };
        let n = notiz(Ulid::new(), &z, &stand, 1).unwrap();
        assert!(n.text.contains("2 offene Merge Requests"), "{}", n.text);
        assert!(!n.text.contains("Pull"), "{}", n.text);
    }

    #[test]
    fn einzahl_und_mehrzahl() {
        let stand = Stand {
            offene_issues: 1,
            offene_prs: 1,
            mehr_prs: false,
            pr_titel: vec!["Eins".into()],
            ci: Ci::Unbekannt,
            standard_branch: "main".into(),
        };
        let n = notiz(Ulid::new(), &zeiger(), &stand, 1).unwrap();
        assert!(n.text.contains("1 offener Pull Request"), "{}", n.text);
        assert!(n.text.contains("1 offene Issue"));
    }
}

/// Läuft nicht in CI: braucht Netz und verbraucht vom GitHub-Kontingent.
/// Von Hand: `cargo test -p lotse-core forge_live -- --ignored --nocapture`
#[cfg(test)]
mod live {
    use super::*;

    fn zeigen(ziel: &str, token: Option<String>) {
        let zeiger = RepoZeiger::erkennen(ziel).unwrap();
        let stand = abfragen(&zeiger, token.as_deref()).expect("Abfrage");
        println!(
            "{} ({}): {} Issues, {} offen zur Übernahme, Prüflauf {} auf {}",
            zeiger.anzeige(),
            zeiger.anbieter.as_str(),
            stand.offene_issues,
            stand.offene_prs,
            stand.ci.as_str(),
            stand.standard_branch
        );
        assert!(!stand.standard_branch.is_empty());
    }

    #[test]
    #[ignore]
    fn forge_live_gegen_echtes_repo() {
        zeigen(
            "https://github.com/phish3144/lotse",
            std::env::var("GITHUB_TOKEN").ok(),
        );
    }

    #[test]
    #[ignore]
    fn forge_live_gegen_gitlab() {
        zeigen(
            "https://gitlab.com/gitlab-org/gitlab-runner",
            std::env::var("GITLAB_TOKEN").ok(),
        );
    }
}
