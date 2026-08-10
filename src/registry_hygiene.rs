//! Registry-aware hygiene (split from `registry` at the 750 ceiling,
//! mw-e6qsjq0): the findings and scans that keep hand-maintained
//! cross-repo state honest — registry namespace damage, renamed refs,
//! dangling `sequence.md` entries, and the pre-drop inbound-needs scan.
//! Loading and resolution stay in `registry`; hygiene *reporting* is
//! portfolio work (DESIGN §9), surfaced through the env-opt-in lint pass
//! and the lifecycle verbs that consult it.

use crate::lint::{Finding, Severity};
use crate::parse::{ParsedTask, Status};
use crate::registry::{push, Registry};
use crate::store::RepoStore;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

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

/// One `sequence.md` entry autoprune removed, with the terminal status
/// that satisfied it.
#[derive(Debug)]
pub struct PrunedEntry {
    /// The entry exactly as written (may spell a former repo name).
    pub target: String,
    /// `done` or `dropped` — why it no longer belongs in the overlay.
    pub status: String,
}

/// Autoprune `sequence.md` (mw-chcqk6g, owner-ruled 2026-08-10: no
/// --prune flag — running any portfolio verb autoprunes): remove entries
/// whose task is done/dropped in a loaded, present repo — satisfied state
/// is dead weight the overlay would otherwise accumulate forever, the
/// clutter archive/ already solves for task files. Only bullet lines that
/// resolve to a terminal task go; headings, prose, live, dangling, and
/// unresolvable entries survive byte-for-byte (an absent checkout is not
/// evidence of death, MW-G5; dangling is lint's finding, not prune's).
/// The file is versioned in the portfolio repo — git diff is the review
/// surface and the undo.
///
/// # Errors
/// A present sequence.md that cannot be read or written back — the prune
/// half-done or silently skipped would both lie about the overlay.
pub fn autoprune_sequence(
    portfolio_dir: &Path,
    registry: &Registry,
    stores: &[RepoStore],
) -> Result<Vec<PrunedEntry>, String> {
    let path = portfolio_dir.join("sequence.md");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(format!("{}: {e}", path.display())),
    };
    let by_name: BTreeMap<&str, &RepoStore> = stores.iter().map(|s| (s.repo.as_str(), s)).collect();
    let satisfied = |line: &str| -> Option<PrunedEntry> {
        let target = line.trim_start().strip_prefix("- ")?.trim();
        let (repo_part, id_part) = target.split_once('#')?;
        let (entry, _) = registry.resolve(repo_part)?;
        let store = by_name.get(entry.name.as_str())?;
        store.entries.iter().find_map(|se| match &se.parsed {
            ParsedTask::Valid(t)
                if t.id == id_part && matches!(t.status, Status::Done | Status::Dropped) =>
            {
                Some(PrunedEntry {
                    target: target.to_string(),
                    status: t.status.as_str().to_string(),
                })
            }
            _ => None,
        })
    };

    let mut pruned = Vec::new();
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| match satisfied(line) {
            Some(entry) => {
                pruned.push(entry);
                false
            }
            None => true,
        })
        .collect();
    if !pruned.is_empty() {
        let mut out = kept.join("\n");
        if text.ends_with('\n') {
            out.push('\n');
        }
        std::fs::write(&path, out).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(pruned)
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
