//! Lint engine (PLAN 0.9): every structural check the spec names — schema
//! warnings (MW-A6), cycles (MW-B2), cross-repo parents (MW-B3), blocked
//! without reason (MW-E1), duplicate IDs (MW-A4), union-merge damage and
//! dangling edges (MW-I2), byte budgets (MW-A5/K3), parent rollup (MW-B7).
//! Pure analysis: repairs live in the CLI's `--fix`.

use crate::parse::{ParsedTask, Status, Task};
use crate::store::RepoStore;
use std::collections::BTreeMap;

/// Finding severity: errors fail lint (exit 1), warnings don't.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Must be repaired (by `--fix` or a human).
    Error,
    /// Should be looked at; never blocks.
    Warning,
}

impl Severity {
    /// Lowercase label for output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

/// One lint finding, sortable into a stable report order.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    /// Error or warning.
    pub severity: Severity,
    /// Stable kebab-case code (`cycle-needs`, `duplicate-id`, …).
    pub code: String,
    /// Task id or file name the finding is about.
    pub subject: String,
    /// Human-readable detail.
    pub message: String,
}

fn finding(severity: Severity, code: &str, subject: &str, message: String) -> Finding {
    Finding {
        severity,
        code: code.to_string(),
        subject: subject.to_string(),
        message,
    }
}

/// Description byte budget (MW-A5: ~2KB, bytes — never lines).
const DESCRIPTION_BUDGET: usize = 2048;
/// Whole-file growth signal (§15.5): usually means the task should split.
const FILE_BUDGET: u64 = 64 * 1024;
/// Attachment excerpt-first threshold (MW-K3).
const ATTACHMENT_BUDGET: u64 = 1024 * 1024;

/// Run every check over a loaded store; findings come back fully sorted
/// (errors first, then code, then subject) for stable output and goldens.
#[must_use]
pub fn lint_store(store: &RepoStore) -> Vec<Finding> {
    let mut out = Vec::new();
    let valid: Vec<&Task> = store
        .entries
        .iter()
        .filter_map(|e| match &e.parsed {
            ParsedTask::Valid(t) => Some(t.as_ref()),
            ParsedTask::Invalid(_) => None,
        })
        .collect();
    let ids: Vec<&str> = valid.iter().map(|t| t.id.as_str()).collect();

    check_files(store, &mut out);
    check_duplicate_ids(&valid, &mut out);
    check_edges(&valid, &ids, &mut out);
    check_cycles(&valid, &mut out);
    check_lifecycle(&valid, &mut out);
    check_budgets(store, &valid, &mut out);
    check_misplaced(store, &mut out);

    out.sort();
    out.dedup();
    out
}

/// Invalid rows (parse / union damage) + parse warnings (schema, MW-A6).
fn check_files(store: &RepoStore, out: &mut Vec<Finding>) {
    for entry in &store.entries {
        match &entry.parsed {
            ParsedTask::Invalid(inv) => {
                let code = if inv.error.contains("duplicate frontmatter key") {
                    "duplicate-key"
                } else {
                    "parse"
                };
                out.push(finding(Severity::Error, code, &inv.id, inv.error.clone()));
            }
            ParsedTask::Valid(t) => {
                for w in &t.warnings {
                    let code = if w.contains("unknown frontmatter key") {
                        "unknown-key"
                    } else {
                        "schema"
                    };
                    out.push(finding(Severity::Warning, code, &t.id, w.clone()));
                }
            }
        }
    }
}

/// Parallel clones CAN mint the same ID (MW-A4); post-merge lint owns it.
fn check_duplicate_ids(valid: &[&Task], out: &mut Vec<Finding>) {
    let mut by_id: BTreeMap<&str, usize> = BTreeMap::new();
    for t in valid {
        *by_id.entry(t.id.as_str()).or_default() += 1;
    }
    for (id, n) in by_id {
        if n > 1 {
            out.push(finding(
                Severity::Error,
                "duplicate-id",
                id,
                format!("{n} files claim this id — lint --fix re-slugs one (MW-A4)"),
            ));
        }
    }
}

/// Cross-repo parents (MW-B3) and dangling same-repo refs (MW-I2).
/// Cross-repo needs/relates are the registry's business (M2), not dangling.
fn check_edges(valid: &[&Task], ids: &[&str], out: &mut Vec<Finding>) {
    let exists = |target: &str| ids.contains(&target);
    for t in valid {
        if let Some(parent) = &t.parent {
            if parent.contains('#') {
                out.push(finding(
                    Severity::Error,
                    "parent-crossrepo",
                    &t.id,
                    format!(
                        "parent `{parent}` crosses repos — hierarchy is per-repo (MW-B3); \
                         use sequence.md tranches"
                    ),
                ));
            } else if !exists(parent) {
                out.push(finding(
                    Severity::Error,
                    "dangling",
                    &t.id,
                    format!("parent `{parent}` does not exist here"),
                ));
            }
        }
        for n in &t.needs {
            if !n.contains('#') && !exists(n) {
                out.push(finding(
                    Severity::Error,
                    "dangling",
                    &t.id,
                    format!("needs `{n}` does not exist here"),
                ));
            }
        }
        for (kind, targets) in [
            ("relates", &t.relates),
            (
                "discovered-from",
                &t.discovered_from.clone().into_iter().collect::<Vec<_>>(),
            ),
        ] {
            for target in targets {
                if !target.contains('#') && !exists(target) {
                    out.push(finding(
                        Severity::Warning,
                        "dangling",
                        &t.id,
                        format!("{kind} `{target}` does not exist here"),
                    ));
                }
            }
        }
    }
}

/// DFS cycle detection on needs + parent, same-repo edges (MW-B2).
fn check_cycles(valid: &[&Task], out: &mut Vec<Finding>) {
    for (code, edges) in [
        ("cycle-needs", same_repo_edges(valid, |t| t.needs.clone())),
        (
            "cycle-parent",
            same_repo_edges(valid, |t| t.parent.clone().into_iter().collect()),
        ),
    ] {
        if let Some(path) = find_cycle(&edges) {
            out.push(finding(
                Severity::Error,
                code,
                path.first().map_or("", String::as_str),
                format!("cycle: {}", path.join(" → ")),
            ));
        }
    }
}

fn same_repo_edges(
    valid: &[&Task],
    get: impl Fn(&Task) -> Vec<String>,
) -> BTreeMap<String, Vec<String>> {
    valid
        .iter()
        .map(|t| {
            let mut targets = get(t);
            targets.retain(|x| !x.contains('#'));
            (t.id.clone(), targets)
        })
        .collect()
}

/// Smallest-keyed cycle path, if any (deterministic for goldens).
fn find_cycle(edges: &BTreeMap<String, Vec<String>>) -> Option<Vec<String>> {
    fn visit<'a>(
        node: &'a str,
        edges: &'a BTreeMap<String, Vec<String>>,
        path: &mut Vec<&'a str>,
        done: &mut Vec<&'a str>,
    ) -> Option<Vec<String>> {
        if let Some(pos) = path.iter().position(|n| *n == node) {
            let mut cycle: Vec<String> = path[pos..].iter().map(ToString::to_string).collect();
            cycle.push(node.to_string());
            return Some(cycle);
        }
        if done.contains(&node) {
            return None;
        }
        path.push(node);
        let hit = edges
            .get(node)
            .and_then(|targets| targets.iter().find_map(|t| visit(t, edges, path, done)));
        path.pop();
        done.push(node);
        hit
    }
    let mut done = Vec::new();
    edges
        .keys()
        .find_map(|n| visit(n, edges, &mut Vec::new(), &mut done))
}

/// blocked-without-reason (MW-E1), missing verify while open (MW-E2),
/// done parents with live children (MW-B7).
fn check_lifecycle(valid: &[&Task], out: &mut Vec<Finding>) {
    let live_children: BTreeMap<&str, Vec<&str>> = valid
        .iter()
        .filter(|t| matches!(t.status, Status::Open | Status::Doing | Status::Blocked))
        .filter_map(|t| Some((t.parent.as_deref()?, t.id.as_str())))
        .fold(BTreeMap::new(), |mut acc, (p, c)| {
            acc.entry(p).or_default().push(c);
            acc
        });
    for t in valid {
        if t.status == Status::Blocked
            && t.blocked_reason
                .as_deref()
                .is_none_or(|r| r.trim().is_empty())
        {
            out.push(finding(
                Severity::Error,
                "blocked-no-reason",
                &t.id,
                "blocked without blocked-reason — name the blocker + unblock condition (MW-E1)"
                    .to_string(),
            ));
        }
        if t.status == Status::Open && t.verify.is_none() {
            out.push(finding(
                Severity::Warning,
                "no-verify",
                &t.id,
                "open without verify: — close will demand --waive (MW-E2)".to_string(),
            ));
        }
        if t.status == Status::Done {
            if let Some(children) = live_children.get(t.id.as_str()) {
                out.push(finding(
                    Severity::Warning,
                    "parent-rollup",
                    &t.id,
                    format!(
                        "done, but children still live: {} (MW-B7)",
                        children.join(", ")
                    ),
                ));
            }
        }
        if matches!(t.status, Status::Done | Status::Dropped)
            && t.handoff.as_deref().is_some_and(|h| !h.trim().is_empty())
        {
            out.push(finding(
                Severity::Warning,
                "handoff-stale",
                &t.id,
                "handoff: on a closed task — the voice belongs on whatever is up next (DESIGN §7b)"
                    .to_string(),
            ));
        }
    }
}

/// Terminal tasks belong in `archive/`, live tasks in the store root
/// (mw-45e2qf4); a hand-edit that flips status without moving the file
/// is mechanical damage `--fix` repairs.
fn check_misplaced(store: &RepoStore, out: &mut Vec<Finding>) {
    for entry in &store.entries {
        let ParsedTask::Valid(t) = &entry.parsed else {
            continue;
        };
        let terminal = matches!(t.status, Status::Done | Status::Dropped);
        let in_archive = entry.file_name.starts_with("archive/");
        if terminal && !in_archive {
            out.push(finding(
                Severity::Warning,
                "misplaced",
                &t.id,
                format!(
                    "{} but its file sits in the store root — terminal tasks live in archive/ (lint --fix moves it)",
                    t.status.as_str()
                ),
            ));
        } else if !terminal && in_archive {
            out.push(finding(
                Severity::Warning,
                "misplaced",
                &t.id,
                format!(
                    "{} but its file sits in archive/ — live tasks belong in the store root (lint --fix moves it)",
                    t.status.as_str()
                ),
            ));
        }
    }
}

/// Byte budgets (MW-A5/D5/K3, §15.5) + attachment paths exist.
fn check_budgets(store: &RepoStore, valid: &[&Task], out: &mut Vec<Finding>) {
    for entry in &store.entries {
        let path = store
            .root
            .join("docs")
            .join("meshwork")
            .join(&entry.file_name);
        if let Ok(meta) = std::fs::metadata(&path) {
            if meta.len() > FILE_BUDGET {
                let subject = match &entry.parsed {
                    ParsedTask::Valid(t) => t.id.clone(),
                    ParsedTask::Invalid(inv) => inv.id.clone(),
                };
                out.push(finding(
                    Severity::Warning,
                    "file-size",
                    &subject,
                    format!(
                        "{} bytes >64KB — usually means split the task (§15.5)",
                        meta.len()
                    ),
                ));
            }
        }
    }
    for t in valid {
        if t.description.len() > DESCRIPTION_BUDGET {
            out.push(finding(
                Severity::Warning,
                "description-size",
                &t.id,
                format!(
                    "description {}B over the ~2KB budget — long design goes behind docs: (MW-A5)",
                    t.description.len()
                ),
            ));
        }
        for rel in &t.attachments {
            let path = store.root.join("docs").join("meshwork").join(rel);
            match std::fs::metadata(&path) {
                Err(_) => out.push(finding(
                    Severity::Warning,
                    "attachment-missing",
                    &t.id,
                    format!("attachment `{rel}` not found"),
                )),
                Ok(meta) if meta.len() > ATTACHMENT_BUDGET => out.push(finding(
                    Severity::Warning,
                    "attachment-size",
                    &t.id,
                    format!(
                        "attachment `{rel}` is {} bytes >1MB — excerpt it (MW-K3)",
                        meta.len()
                    ),
                )),
                Ok(_) => {}
            }
        }
    }
}
