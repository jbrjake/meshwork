//! Portfolio registry (MW-G2) — the durability slice (mw-mrjccx2):
//! `repos.toml` maps name → remote, plus per-repo `aliases = ["oldname"]`
//! so refs baked into OTHER repos' files survive a rename. Resolution
//! accepts old names; registry-aware lint warns `renamed-repo` with the
//! rewrite, errors `registry-collision` on namespace damage and
//! `alias-collision` when two registered repos claim the same ID prefix
//! (bare-ID lookup is ambiguous the moment that happens). Local paths come
//! from the gitignored `repos.local.toml` `[paths]` table, defaulting to
//! `~/Documents/code/<name>`. Override semantics (mw-5ckb): absolute
//! values pass through, `~/` expands against HOME (loud when it can't),
//! relative values anchor at the portfolio dir; keys share the name+alias
//! namespace — former names apply but warn, unknown keys warn (the file is
//! gitignored, no other review surface), two keys on one entry error.
//! Discovery (mw-9093): `MESHWORK_PORTFOLIO` overrides, default
//! `~/Documents/code/portfolio` (§15.4) — `portfolio` verbs use the
//! chain; single-repo lint stays env-opt-in (per-repo scope, MW-G1).

use crate::lint::{Finding, Severity};
use crate::parse::ParsedTask;
use crate::store::RepoStore;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// One registered repo.
#[derive(Debug, Clone)]
pub struct RepoEntry {
    /// Canonical name — the `repo#id` namespace (MW-B3/G2).
    pub name: String,
    /// GitHub remote, informational here.
    pub remote: Option<String>,
    /// Former names still accepted in inbound refs (mw-mrjccx2).
    pub aliases: Vec<String>,
    /// Local checkout, when `repos.local.toml` says (or the default holds).
    pub path: Option<PathBuf>,
}

/// A loaded registry.
#[derive(Debug, Clone)]
pub struct Registry {
    /// Entries in file order (the MW-G4 fallback ordering).
    pub entries: Vec<RepoEntry>,
    /// Findings minted while applying `repos.local.toml` overrides —
    /// carried here so every load path reports them, not just lint's.
    pub override_findings: Vec<Finding>,
}

#[derive(Debug, Deserialize)]
struct ReposFile {
    #[serde(default, rename = "repo")]
    repos: Vec<RepoToml>,
}

#[derive(Debug, Deserialize)]
struct RepoToml {
    name: String,
    #[serde(default)]
    remote: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
struct LocalFile {
    #[serde(default)]
    paths: BTreeMap<String, String>,
}

/// Load `repos.toml` (+ optional `repos.local.toml` paths) from a
/// portfolio directory.
///
/// # Errors
/// Unreadable or unparseable registry files — loud, never guessed around.
pub fn load(portfolio_dir: &Path) -> Result<Registry, String> {
    let repos_path = portfolio_dir.join("repos.toml");
    let text = std::fs::read_to_string(&repos_path)
        .map_err(|e| format!("{}: {e}", repos_path.display()))?;
    let parsed: ReposFile =
        toml::from_str(&text).map_err(|e| format!("{}: {e}", repos_path.display()))?;

    // Absent is the normal state; present-but-unreadable is loud — only
    // NotFound may pass silently (mw-5ckb).
    let local_path = portfolio_dir.join("repos.local.toml");
    let local: LocalFile = match std::fs::read_to_string(&local_path) {
        Ok(text) => toml::from_str(&text).map_err(|e| format!("{}: {e}", local_path.display()))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => LocalFile::default(),
        Err(e) => return Err(format!("{}: {e}", local_path.display())),
    };

    let mut override_findings = Vec::new();
    // canonical name → (winning key, resolved path); BTreeMap iteration
    // makes the first-key-wins tiebreak deterministic (the collision is an
    // error regardless — nothing rides on which side wins).
    let mut overrides: BTreeMap<String, (String, PathBuf)> = BTreeMap::new();
    for (key, value) in &local.paths {
        let hit = parsed
            .repos
            .iter()
            .find(|r| r.name == *key)
            .map(|r| (r, false))
            .or_else(|| {
                parsed
                    .repos
                    .iter()
                    .find(|r| r.aliases.iter().any(|a| a == key))
                    .map(|r| (r, true))
            });
        let Some((repo, via_alias)) = hit else {
            push(
                &mut override_findings,
                Severity::Warning,
                "unknown-path-override",
                key,
                format!(
                    "`[paths]` key `{key}` in repos.local.toml matches no registered repo — \
                     a typo, or a repo missing from repos.toml"
                ),
            );
            continue;
        };
        if via_alias {
            push(
                &mut override_findings,
                Severity::Warning,
                "renamed-repo",
                key,
                format!(
                    "`[paths]` key `{key}` uses a former name — repo is now `{}`; \
                     rename the key",
                    repo.name
                ),
            );
        }
        if let Some((first_key, _)) = overrides.get(&repo.name) {
            push(
                &mut override_findings,
                Severity::Error,
                "override-collision",
                &repo.name,
                format!(
                    "`[paths]` keys `{first_key}` and `{key}` both override `{}` — \
                     ambiguous; keep one",
                    repo.name
                ),
            );
            continue;
        }
        let path = expand_override(value, portfolio_dir)
            .map_err(|e| format!("{}: [paths] {key}: {e}", local_path.display()))?;
        overrides.insert(repo.name.clone(), (key.clone(), path));
    }

    let entries = parsed
        .repos
        .into_iter()
        .map(|r| {
            let path = overrides
                .get(&r.name)
                .map(|(_, p)| p.clone())
                .or_else(|| default_path(&r.name));
            RepoEntry {
                name: r.name,
                remote: r.remote,
                aliases: r.aliases,
                path,
            }
        })
        .collect();
    Ok(Registry {
        entries,
        override_findings,
    })
}

/// `~/Documents/code/<name>` (MW-G2's default), when HOME resolves.
fn default_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| Path::new(&h).join("Documents").join("code").join(name))
}

/// Locate the portfolio dir (mw-9093): `MESHWORK_PORTFOLIO` overrides
/// (tests, nonstandard machines — §15.6); the default is
/// `~/Documents/code/portfolio` (§15.4).
///
/// # Errors
/// Nothing resolves to a registry — loud, names the fix.
pub fn portfolio_dir() -> Result<PathBuf, String> {
    if let Some(dir) = std::env::var_os("MESHWORK_PORTFOLIO").filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(dir));
    }
    let default = std::env::var_os("HOME").map(|h| {
        Path::new(&h)
            .join("Documents")
            .join("code")
            .join("portfolio")
    });
    match default {
        Some(dir) if dir.join("repos.toml").exists() => Ok(dir),
        Some(dir) => Err(format!(
            "no portfolio registry: {} has no repos.toml — create it (MW-G2), or set \
             MESHWORK_PORTFOLIO=<dir>",
            dir.display()
        )),
        None => Err(
            "no portfolio registry: HOME is unset and MESHWORK_PORTFOLIO is not given".to_string(),
        ),
    }
}

/// A cross-repo target resolved by direct file lookup (DESIGN §5,
/// mw-k7r5): the registry names the repo, the ID-prefixed filename finds
/// the file, one parse yields the fields — no full portfolio load
/// (MW-B3/G5).
#[derive(Debug, Clone)]
pub struct ForeignTask {
    /// The ref exactly as written — edges join on the literal string,
    /// so a rename-alias ref keeps its spelling here.
    pub gid: String,
    /// Canonical repo name (rename aliases resolve, mw-mrjccx2).
    pub repo: String,
    /// Bare task id.
    pub id: String,
    /// Status string as parsed.
    pub status: String,
    /// Title when the file parses.
    pub title: Option<String>,
    /// Absolute path of the resolved file (outside this repo, so never
    /// store-relative like local rows).
    pub path: String,
}

/// Quiet registry discovery for single-repo resolution (mw-k7r5): no
/// registry anywhere is the normal state (`None`, today's conservative
/// behavior); a FOUND but broken one is loud — never guessed around.
///
/// # Errors
/// A discovered registry that fails to load.
pub fn quiet_load() -> Result<Option<Registry>, String> {
    match portfolio_dir() {
        Ok(dir) => load(&dir).map(Some),
        Err(_) => Ok(None),
    }
}

/// Every cross-repo `needs` ref in the store set — the refs that gate
/// readiness (DESIGN §5's blocking predicate).
#[must_use]
pub fn foreign_refs(stores: &[&RepoStore]) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for store in stores {
        for entry in &store.entries {
            if let ParsedTask::Valid(t) = &entry.parsed {
                refs.extend(t.needs.iter().filter(|n| n.contains('#')).cloned());
            }
        }
    }
    refs
}

/// Resolve foreign refs by direct file lookup. Refs into `loaded` repos
/// are those stores' own rows — skipped, or the union would hold the same
/// task twice. Unregistered repos, absent checkouts, missing files,
/// unparseable files: skipped too — NULL stays conservative (MW-G5);
/// nothing here is an error.
#[must_use]
pub fn resolve_foreign(
    registry: &Registry,
    refs: &BTreeSet<String>,
    loaded: &BTreeSet<&str>,
) -> Vec<ForeignTask> {
    let mut out = Vec::new();
    for target in refs {
        let Some((repo_part, id_part)) = target.split_once('#') else {
            continue;
        };
        let Some((entry, _)) = registry.resolve(repo_part) else {
            continue;
        };
        if loaded.contains(entry.name.as_str()) {
            continue;
        }
        let Some(root) = &entry.path else { continue };
        let tasks_dir = root.join("docs").join("meshwork");
        let Some(file) = crate::store::find_task_file(&tasks_dir, id_part) else {
            continue;
        };
        let ParsedTask::Valid(t) = crate::parse::parse_task_file(&file) else {
            continue;
        };
        out.push(ForeignTask {
            gid: target.clone(),
            repo: entry.name.clone(),
            id: t.id.clone(),
            status: t.status.as_str().to_string(),
            title: Some(t.title.clone()),
            path: file.display().to_string(),
        });
    }
    out
}

/// Parse `sequence.md` (MW-G4, mw-jpbv): `- repo#id` bullets in file
/// order; tranche headings are cosmetic. An absent file is the normal
/// state (empty overlay); a present-but-unreadable one is loud. Bullets
/// without a `repo#id` shape are prose — ignored.
///
/// # Errors
/// A present but unreadable sequence file.
pub fn load_sequence(portfolio_dir: &Path) -> Result<Vec<String>, String> {
    let path = portfolio_dir.join("sequence.md");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    Ok(text
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix("- "))
        .map(str::trim)
        .filter(|s| s.contains('#'))
        .map(ToString::to_string)
        .collect())
}

/// One registered repo the union could not load (MW-G5: reported, never
/// an error, never guessed around).
#[derive(Debug, Clone)]
pub struct SkippedRepo {
    /// Canonical registry name.
    pub repo: String,
    /// Stable machine token: `no-path` | `no-checkout` | `no-store`.
    pub reason: &'static str,
    /// Human detail — may contain machine-local paths, so it belongs in
    /// stderr reports, never in golden-compared output.
    pub detail: String,
}

/// Load every registered repo's store for the union (MW-G3). The store's
/// `repo` becomes the registry name — the `repo#id` namespace is the
/// registry's, not the checkout dirname's. Absent things skip + report
/// (MW-G5); a present-but-broken store is a loud error, never a silent
/// hole in the union.
///
/// # Errors
/// A checkout whose store exists but fails to load — its tasks silently
/// missing from the union would misreport the portfolio.
pub fn load_stores(registry: &Registry) -> Result<(Vec<RepoStore>, Vec<SkippedRepo>), String> {
    let mut stores = Vec::new();
    let mut skipped = Vec::new();
    for entry in &registry.entries {
        let Some(path) = &entry.path else {
            skipped.push(SkippedRepo {
                repo: entry.name.clone(),
                reason: "no-path",
                detail: "no local path (HOME unset and no repos.local.toml override)".into(),
            });
            continue;
        };
        if !path.exists() {
            skipped.push(SkippedRepo {
                repo: entry.name.clone(),
                reason: "no-checkout",
                detail: format!("no checkout at {}", path.display()),
            });
            continue;
        }
        match crate::store::load_repo(path) {
            Ok(mut store) => {
                store.repo.clone_from(&entry.name);
                stores.push(store);
            }
            Err(crate::store::StoreError::NotAStore(p)) => skipped.push(SkippedRepo {
                repo: entry.name.clone(),
                reason: "no-store",
                detail: format!("{} is not a meshwork store", p.display()),
            }),
            Err(e) => return Err(format!("{}: {e}", entry.name)),
        }
    }
    Ok((stores, skipped))
}

/// Absolute values pass through; `~`/`~/…` expand against HOME; anything
/// else anchors at the portfolio dir — the only deterministic anchor for a
/// per-machine file (cwd varies per invocation).
///
/// # Errors
/// `~` that cannot resolve (HOME unset) and `~user` forms — loud, an
/// explicit override is never guessed around.
fn expand_override(value: &str, portfolio_dir: &Path) -> Result<PathBuf, String> {
    let tilde_rest = match value {
        "~" => Some(""),
        _ => value.strip_prefix("~/"),
    };
    if let Some(rest) = tilde_rest {
        let home = std::env::var_os("HOME").ok_or_else(|| {
            format!("`{value}`: HOME is unset, `~` cannot resolve — spell the path out")
        })?;
        let home = Path::new(&home);
        return Ok(if rest.is_empty() {
            home.to_path_buf()
        } else {
            home.join(rest)
        });
    }
    if value.starts_with('~') {
        return Err(format!(
            "`{value}`: `~user` expansion is not supported — spell the path out"
        ));
    }
    let p = Path::new(value);
    Ok(if p.is_absolute() {
        p.to_path_buf()
    } else {
        portfolio_dir.join(p)
    })
}

impl Registry {
    /// Resolve a repo name, canonical or via a rename alias; the bool says
    /// an alias matched (the caller's cue to suggest the rewrite).
    #[must_use]
    pub fn resolve(&self, name: &str) -> Option<(&RepoEntry, bool)> {
        self.entries
            .iter()
            .find(|e| e.name == name)
            .map(|e| (e, false))
            .or_else(|| {
                self.entries
                    .iter()
                    .find(|e| e.aliases.iter().any(|a| a == name))
                    .map(|e| (e, true))
            })
    }
}

/// Append one finding — shared with `registry_hygiene`, where the
/// registry-aware findings themselves live (mw-e6qsjq0).
pub(crate) fn push(
    out: &mut Vec<Finding>,
    severity: Severity,
    code: &str,
    subject: &str,
    message: String,
) {
    out.push(Finding {
        severity,
        code: code.to_string(),
        subject: subject.to_string(),
        message,
    });
}
