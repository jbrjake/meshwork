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
//! Registry context reaches single-repo verbs only through
//! `MESHWORK_PORTFOLIO` until M2 wires proper discovery.

use crate::lint::{Finding, Severity};
use crate::parse::ParsedTask;
use crate::store::RepoStore;
use serde::Deserialize;
use std::collections::BTreeMap;
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

/// Registry-aware findings for one loaded store: namespace collisions in
/// the registry itself, ID-alias-prefix collisions across registered
/// repos, and renamed-repo refs in this store's files.
#[must_use]
pub fn registry_findings(registry: &Registry, store: &RepoStore) -> Vec<Finding> {
    let mut out = registry.override_findings.clone();
    namespace_collisions(registry, &mut out);
    id_alias_collisions(registry, &mut out);
    renamed_refs(registry, store, &mut out);
    out
}

fn push(out: &mut Vec<Finding>, severity: Severity, code: &str, subject: &str, message: String) {
    out.push(Finding {
        severity,
        code: code.to_string(),
        subject: subject.to_string(),
        message,
    });
}

/// Two entries claiming one name in the name+aliases namespace make
/// resolution ambiguous — registry damage, an error.
fn namespace_collisions(registry: &Registry, out: &mut Vec<Finding>) {
    let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
    for entry in &registry.entries {
        for name in
            std::iter::once(entry.name.as_str()).chain(entry.aliases.iter().map(String::as_str))
        {
            match seen.get(name) {
                Some(first) if *first != entry.name => push(
                    out,
                    Severity::Error,
                    "registry-collision",
                    name,
                    format!(
                        "`{name}` is claimed by both `{first}` and `{}` in repos.toml — \
                         resolution is ambiguous",
                        entry.name
                    ),
                ),
                Some(_) => {}
                None => {
                    seen.insert(name, entry.name.as_str());
                }
            }
        }
    }
}

/// Two locally-present registered repos minting the same ID alias prefix
/// (config.toml `alias`) make bare-ID lookup ambiguous — an error. Absent
/// or unreadable repos are skipped, never guessed (MW-G5's spirit).
fn id_alias_collisions(registry: &Registry, out: &mut Vec<Finding>) {
    let mut by_alias: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for entry in &registry.entries {
        let Some(path) = &entry.path else { continue };
        let config = path.join("docs").join("meshwork").join("config.toml");
        let Ok(text) = std::fs::read_to_string(config) else {
            continue;
        };
        let Ok(cfg) = toml::from_str::<crate::store::Config>(&text) else {
            continue;
        };
        by_alias.entry(cfg.alias).or_default().push(&entry.name);
    }
    for (alias, repos) in by_alias {
        if repos.len() > 1 {
            push(
                out,
                Severity::Error,
                "alias-collision",
                &alias,
                format!(
                    "ID alias `{alias}-` is minted by {} — bare-ID lookup is ambiguous; \
                     re-alias all but one store",
                    repos.join(" and ")
                ),
            );
        }
    }
}

/// Cross-repo refs written against a former name: still resolve, but warn
/// with the rewrite — never silent, never auto-fixed.
fn renamed_refs(registry: &Registry, store: &RepoStore, out: &mut Vec<Finding>) {
    for entry in &store.entries {
        let ParsedTask::Valid(t) = &entry.parsed else {
            continue;
        };
        let refs = t
            .needs
            .iter()
            .chain(&t.relates)
            .chain(&t.discovered_from)
            .chain(&t.parent);
        for target in refs {
            let Some((repo_part, id_part)) = target.split_once('#') else {
                continue;
            };
            if let Some((resolved, true)) = registry.resolve(repo_part) {
                push(
                    out,
                    Severity::Warning,
                    "renamed-repo",
                    &t.id,
                    format!(
                        "ref `{target}` uses former name `{repo_part}` — repo is now \
                         `{}`; rewrite to `{}#{id_part}`",
                        resolved.name, resolved.name
                    ),
                );
            }
        }
    }
}
