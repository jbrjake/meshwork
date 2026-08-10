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
use crate::parse::{ParsedTask, Status};
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

/// One live task in another registered repo whose `needs` a drop is about
/// to satisfy (mw-kkvs8zq).
#[derive(Debug)]
pub struct InboundNeed {
    /// `repo#id` of the task holding the need, canonical repo name.
    pub src_gid: String,
    /// The ref exactly as written there (may spell a former name).
    pub target: String,
}

/// Result of the pre-drop scan: this repo's canonical registry name, the
/// live inbound cross-repo needs on the dropped id, and the repos the
/// scan could not check.
#[derive(Debug)]
pub struct InboundScan {
    /// This repo as the registry names it.
    pub self_name: String,
    /// Live (non-terminal) tasks elsewhere needing `<self>#<id>`.
    pub hits: Vec<InboundNeed>,
    /// `(repo, why)` for every registered repo that could not be scanned —
    /// reported, because silence would read as "all clear" (MW-G5).
    pub unscanned: Vec<(String, &'static str)>,
}

/// Scan registered repos for live inbound cross-repo `needs` on
/// `<self>#<id>` (mw-kkvs8zq): only done/dropped satisfies a dependency,
/// and a drop clearing someone else's need means the needed work never
/// happened — worth a warning on both sides of the boundary. `None` when
/// `self_root` is not a registered repo (no cross-repo namespace reaches
/// it). Broken or absent sibling repos are never an error here — the scan
/// is advisory and must not block a local lifecycle verb — they land in
/// `unscanned` instead.
#[must_use]
pub fn inbound_needs(registry: &Registry, self_root: &Path, id: &str) -> Option<InboundScan> {
    let self_canon = self_root.canonicalize().ok()?;
    let self_entry = registry.entries.iter().find(|e| {
        e.path
            .as_ref()
            .and_then(|p| p.canonicalize().ok())
            .is_some_and(|p| p == self_canon)
    })?;
    let self_names: BTreeSet<&str> = std::iter::once(self_entry.name.as_str())
        .chain(self_entry.aliases.iter().map(String::as_str))
        .collect();

    let mut hits = Vec::new();
    let mut unscanned = Vec::new();
    for entry in &registry.entries {
        if entry.name == self_entry.name {
            continue;
        }
        let Some(path) = &entry.path else {
            unscanned.push((entry.name.clone(), "no local path"));
            continue;
        };
        if !path.exists() {
            unscanned.push((entry.name.clone(), "no checkout"));
            continue;
        }
        let store = match crate::store::load_repo(path) {
            Ok(store) => store,
            Err(crate::store::StoreError::NotAStore(_)) => {
                unscanned.push((entry.name.clone(), "not a meshwork store"));
                continue;
            }
            Err(_) => {
                unscanned.push((entry.name.clone(), "store failed to load"));
                continue;
            }
        };
        for se in &store.entries {
            let ParsedTask::Valid(t) = &se.parsed else {
                continue;
            };
            if matches!(t.status, Status::Done | Status::Dropped) {
                continue;
            }
            for n in &t.needs {
                let hit = n
                    .split_once('#')
                    .is_some_and(|(rp, ip)| ip == id && self_names.contains(rp));
                if hit {
                    hits.push(InboundNeed {
                        src_gid: format!("{}#{}", entry.name, t.id),
                        target: n.clone(),
                    });
                }
            }
        }
    }
    Some(InboundScan {
        self_name: self_entry.name.clone(),
        hits,
        unscanned,
    })
}

/// Dangling `sequence.md` entries (mw-2nmsys2): the file is hand-written,
/// denormalized, cross-repo state, so a typo'd or deleted id is the same
/// dangling-edge class lint catches inside a repo — and the overlay
/// otherwise skips it silently ("first ready one wins"). The finding fires
/// only when the entry resolves nowhere it could: an unregistered repo
/// name, or a registered, PRESENT store without the id. A repo absent from
/// disk is unresolvable — the skipped-repo notice's business, never
/// guessed (MW-G5) — and an entry resolving to done/dropped is satisfied:
/// prune's business, not damage. Warning, not error: ordering degrades,
/// readiness semantics don't.
#[must_use]
pub fn sequence_findings(registry: &Registry, sequence: &[String]) -> Vec<Finding> {
    let mut out = Vec::new();
    for target in sequence {
        let Some((repo_part, id_part)) = target.split_once('#') else {
            continue;
        };
        let Some((entry, _)) = registry.resolve(repo_part) else {
            push(
                &mut out,
                Severity::Warning,
                "dangling-sequence",
                target,
                format!(
                    "sequence.md entry `{target}` names no registered repo — a typo, \
                     or a repo missing from repos.toml"
                ),
            );
            continue;
        };
        let Some(root) = &entry.path else { continue };
        let tasks_dir = root.join("docs").join("meshwork");
        if !tasks_dir.join("config.toml").exists() {
            continue; // no checkout / no store — unresolvable, not dangling
        }
        if crate::store::find_task_file(&tasks_dir, id_part).is_none() {
            push(
                &mut out,
                Severity::Warning,
                "dangling-sequence",
                target,
                format!(
                    "sequence.md entry `{target}` resolves to no task in `{}` — a \
                     typo'd or deleted id; edit sequence.md",
                    entry.name
                ),
            );
        }
    }
    out
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
