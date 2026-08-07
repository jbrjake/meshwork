//! Portfolio registry (MW-G2) — the durability slice (mw-mrjccx2):
//! `repos.toml` maps name → remote, plus per-repo `aliases = ["oldname"]`
//! so refs baked into OTHER repos' files survive a rename. Resolution
//! accepts old names; registry-aware lint warns `renamed-repo` with the
//! rewrite, errors `registry-collision` on namespace damage and
//! `alias-collision` when two registered repos claim the same ID prefix
//! (bare-ID lookup is ambiguous the moment that happens). Local paths come
//! from the gitignored `repos.local.toml` `[paths]` table, defaulting to
//! `~/Documents/code/<name>`; full override semantics land with 2.1
//! (mw-5ckb). Registry context reaches single-repo verbs only through
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

    let local: LocalFile = match std::fs::read_to_string(portfolio_dir.join("repos.local.toml")) {
        Ok(text) => toml::from_str(&text)
            .map_err(|e| format!("{}: {e}", portfolio_dir.join("repos.local.toml").display()))?,
        Err(_) => LocalFile::default(),
    };

    let entries = parsed
        .repos
        .into_iter()
        .map(|r| {
            let path = local
                .paths
                .get(&r.name)
                .map(PathBuf::from)
                .or_else(|| default_path(&r.name));
            RepoEntry {
                name: r.name,
                remote: r.remote,
                aliases: r.aliases,
                path,
            }
        })
        .collect();
    Ok(Registry { entries })
}

/// `~/Documents/code/<name>` (MW-G2's default), when HOME resolves.
fn default_path(name: &str) -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| Path::new(&h).join("Documents").join("code").join(name))
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
    let mut out = Vec::new();
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
