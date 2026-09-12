//! Verify-shaped lint checks, split from `lint.rs` at the 500-line
//! target: the static tier of verify hygiene. Trivially-satisfiable
//! verifies (mw-221f3jt), the migration-pressure pair (mw-4aqmf0t,
//! DESIGN §12b): `verify-shell` warns on legacy shell text — legal
//! forever behind the MW-E5 gate, but loud so stores drift toward the
//! DSL — and `verify-malformed` warns on keyword-led text that will
//! refuse at close — plus `verify-changed-since-approval` (mw-yyf1bab):
//! a live verify differing from what this clone approved, shown as
//! approved-vs-current, because the close-time re-approval prompt is
//! only a speed bump if the operator is clicking through. Warnings all;
//! nothing here executes anything.

use crate::lint::{finding, Finding, Severity};
use crate::parse::{Status, Task};
use crate::store::RepoStore;
use crate::verify_dsl::{classify, Classified, Predicate};

/// All verify-shaped checks over the live tasks.
pub(crate) fn check(store: &RepoStore, valid: &[&Task], out: &mut Vec<Finding>) {
    for (t, approved) in changed_since_approval(store, valid) {
        let now = t.verify.as_deref().map(str::trim).unwrap_or_default();
        out.push(finding(
            Severity::Warning,
            "verify-changed-since-approval",
            &t.id,
            format!(
                "verify changed since this clone approved it — approved: \
                 `{approved}`, now: `{now}`; review the change (close --approve \
                 re-records)"
            ),
        ));
    }
    for t in valid {
        if matches!(t.status, Status::Done | Status::Dropped) {
            continue;
        }
        let Some(v) = t.verify.as_deref().map(str::trim) else {
            continue;
        };
        if let Some(why) = trivial_reason(&store.root, v) {
            out.push(finding(
                Severity::Warning,
                "verify-trivial",
                &t.id,
                format!("verify `{v}` {why} — it cannot detect the work"),
            ));
        }
        let classified = classify(v);
        if let Some(pattern) = self_satisfying(store, t, v, &classified) {
            out.push(finding(
                Severity::Warning,
                "verify-self-satisfying",
                &t.id,
                format!(
                    "verify reads the task's own file for `{pattern}` — satisfiable by whoever \
                     writes the file (a heuristic: lint cannot know who); the sanctioned shape \
                     is the date-first owner marker, `contains <own file> /^- 2026-…/`"
                ),
            ));
        }
        if let Some(path) = missing_read_path(&store.root, v, &classified) {
            out.push(finding(
                Severity::Warning,
                "verify-path-missing",
                &t.id,
                format!(
                    "verify reads `{path}`, which does not exist — the task can never \
                     close until it appears; if the file moved, re-point the verify"
                ),
            ));
        }
        match classified {
            Classified::LegacyShell => out.push(finding(
                Severity::Warning,
                "verify-shell",
                &t.id,
                format!(
                    "verify `{v}` is legacy shell — runs only behind the per-clone \
                     approval gate; prefer the DSL (exists/absent/contains/run)"
                ),
            )),
            Classified::Malformed(why) => out.push(finding(
                Severity::Warning,
                "verify-malformed",
                &t.id,
                format!(
                    "verify `{v}` is keyword-led but does not parse ({why}) — close will refuse it"
                ),
            )),
            Classified::Dsl(_) => {}
        }
    }
}

/// mw-c3s9209: the first path a verify *reads* that is absent from the
/// tree — a `contains <path>` predicate, or a plain legacy `grep … <path>`
/// with no shell plumbing. Such a task can never close and looks like
/// unfinished work. `exists` is excluded by definition (an absent
/// artifact is that verify's red state); a path that escapes the repo is
/// `path-escape`'s finding, not this one.
fn missing_read_path(root: &std::path::Path, v: &str, classified: &Classified) -> Option<String> {
    let paths: Vec<String> = match classified {
        Classified::Dsl(preds) => preds
            .iter()
            .filter_map(|p| match p {
                Predicate::Contains { path, .. } => Some(path.clone()),
                _ => None,
            })
            .collect(),
        Classified::LegacyShell => legacy_grep_path(v).into_iter().collect(),
        Classified::Malformed(_) => Vec::new(),
    };
    paths.into_iter().find(|p| {
        crate::paths::confine(root, p)
            .ok()
            .is_some_and(|abs| !abs.exists())
    })
}

/// mw-xb9prd6: the pattern a verify looks for in the task's own file,
/// when that pattern is not the date-first owner marker — a `contains`
/// (or a bare legacy grep) on the file the author writes is satisfiable
/// by the author. Lexical: it judges the shape, never the intent.
fn self_satisfying(
    store: &RepoStore,
    t: &Task,
    v: &str,
    classified: &Classified,
) -> Option<String> {
    let own = store
        .entries
        .iter()
        .find(|e| matches!(&e.parsed, crate::parse::ParsedTask::Valid(x) if x.id == t.id))
        .map(|e| format!("docs/meshwork/{}", e.file_name))?;
    let is_own = |path: &str| path.trim_start_matches("./") == own;
    let date_first = |pattern: &str| {
        let core = pattern.trim_matches(['/', '"', '\'']);
        let core = core.trim_start_matches(['^', '-', ' ', '\\', 's']);
        core.len() >= 5
            && core[..4].bytes().all(|b| b.is_ascii_digit())
            && core.as_bytes()[4] == b'-'
    };
    match classified {
        Classified::Dsl(preds) => preds.iter().find_map(|p| match p {
            Predicate::Contains { path, pattern } if is_own(path) => {
                let text = match pattern {
                    crate::verify_dsl::Pattern::Literal(s) => s.clone(),
                    crate::verify_dsl::Pattern::Regex(s) => format!("/{s}/"),
                };
                (!date_first(&text)).then_some(text)
            }
            _ => None,
        }),
        Classified::LegacyShell => {
            let path = legacy_grep_path(v)?;
            if !is_own(&path) {
                return None;
            }
            let args: Vec<&str> = v
                .split_whitespace()
                .skip(1)
                .filter(|t| !t.starts_with('-'))
                .collect();
            let pattern = (args.len() >= 2).then(|| args[args.len() - 2])?;
            (!date_first(pattern)).then(|| pattern.trim_matches(['"', '\'']).to_string())
        }
        Classified::Malformed(_) => None,
    }
}

/// The file a bare `grep [flags] <pattern> <path>` reads, when the text
/// is exactly that shape — any pipe, chain, redirect or expansion makes
/// it someone's real gate, unjudged.
fn legacy_grep_path(v: &str) -> Option<String> {
    if v.contains(['|', ';', '&', '<', '>', '$', '`', '(', ')']) {
        return None;
    }
    let toks: Vec<&str> = v.split_whitespace().collect();
    let (&"grep", rest) = toks.split_first()? else {
        return None;
    };
    let args: Vec<&str> = rest
        .iter()
        .copied()
        .filter(|t| !t.starts_with('-'))
        .collect();
    let path = (args.len() >= 2).then(|| args[args.len() - 1])?;
    Some(path.trim_matches(['"', '\'']).to_string())
}

/// mw-yyf1bab: live tasks whose current verify differs from the newest
/// text this clone approved, with that approved text. Empty when no
/// approvals are recorded (fresh clone, hash-era file, CI). A task whose
/// file is dirty in the working tree is skipped (mw-4n00yte): an
/// uncommitted edit was made on this clone, not arrived from elsewhere —
/// the finding waits for the commit; the close gate is untouched. Shared
/// with prime, which surfaces the ids as a session-start nudge.
pub(crate) fn changed_since_approval<'a>(
    store: &RepoStore,
    tasks: &[&'a Task],
) -> Vec<(&'a Task, String)> {
    let approved = crate::trust::approved_texts(&store.root);
    if approved.is_empty() {
        return Vec::new();
    }
    let dirty = dirty_ids(store);
    tasks
        .iter()
        .filter(|t| !matches!(t.status, Status::Done | Status::Dropped))
        .filter(|t| !dirty.contains(&t.id))
        .filter_map(|t| {
            let now = t.verify.as_deref()?.trim();
            let then = approved.get(&t.id)?;
            (then.trim() != now).then(|| (*t, then.clone()))
        })
        .collect()
}

/// Ids whose task file has uncommitted changes — `git status --porcelain`
/// over the store dir, mapped through the loaded entries. Any git failure
/// (no repo, no git) reads as nothing dirty.
fn dirty_ids(store: &RepoStore) -> std::collections::BTreeSet<String> {
    let Ok(out) = std::process::Command::new("git")
        .args(["status", "--porcelain", "--", "docs/meshwork"])
        .current_dir(&store.root)
        .output()
    else {
        return std::collections::BTreeSet::new();
    };
    if !out.status.success() {
        return std::collections::BTreeSet::new();
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let files: std::collections::BTreeSet<&str> = text
        .lines()
        .filter_map(|l| l.get(3..))
        .map(|p| p.rsplit(" -> ").next().unwrap_or(p).trim_matches('"'))
        .filter_map(|p| p.strip_prefix("docs/meshwork/"))
        .collect();
    store
        .entries
        .iter()
        .filter(|e| files.contains(e.file_name.as_str()))
        .filter_map(|e| match &e.parsed {
            crate::parse::ParsedTask::Valid(t) => Some(t.id.clone()),
            crate::parse::ParsedTask::Invalid(_) => None,
        })
        .collect()
}

/// mw-221f3jt: why a verify is trivially satisfiable, if it is. Exact
/// shapes only: compound commands (`&&`, `|`, `;`) are someone's real
/// gate — unjudged. The start red-check (mw-175bn4c) is the dynamic
/// tier; this static tier never executes and never touches the gate.
fn trivial_reason(root: &std::path::Path, v: &str) -> Option<String> {
    let toks: Vec<&str> = v.split_whitespace().collect();
    let present =
        |path: &str| !path.starts_with('/') && !path.contains("..") && root.join(path).exists();
    match toks.as_slice() {
        ["true" | ":"] | ["exit", "0"] => Some("always exits 0".into()),
        ["echo", ..] => Some("echo always exits 0".into()),
        ["touch", ..] => Some("touch satisfies itself".into()),
        ["test", "-f" | "-e", path] | ["exists", path] if present(path) => {
            Some(format!("is already green ({path} exists today)"))
        }
        _ => None,
    }
}
