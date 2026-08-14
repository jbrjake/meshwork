//! Ride-along provenance guard (mw-egksvhm; DESIGN §12b gate routing,
//! owner ruling 2026-08-14). Confirmed threat: one PR carries a task
//! plus the test its `run` verify names — merged, the task self-verifies
//! against attacker code at close. Judgment is a union, conservative in
//! both directions: every commit touching the task file is judged by its
//! OWN full delta — a squash-merge is the whole PR in one linear commit,
//! so single-commit task+code combos always gate — and a commit that
//! reached mainline through a merge is ADDITIONALLY judged by that
//! merge's entire first-parent delta, so a PR splitting task and test
//! across inner commits is still one arrival. The merge basis widens
//! the net to the delivered group; it never exempts a commit. No merge
//! style is forced on contributors. Local refs only, zero network
//! (MW-J6); when git cannot answer, degrade toward gating, never trust.

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;

/// The store prefix that makes a merge delta harmless.
const STORE_PREFIX: &str = "docs/meshwork/";

/// Verdict on one task file's arrival history.
#[derive(Debug)]
pub enum Provenance {
    /// Operator-authored commits and store-only merges — `run` verifies
    /// on this task may execute approval-free.
    Trusted,
    /// A commit or merge delivered this task alongside non-store
    /// content; every `run` on the task gates like legacy shell.
    RodeAlong {
        /// The arrival judged mixed: the commit itself, or its landing
        /// merge (full hash).
        arrival: String,
        /// One offending path outside the store, for the refusal line.
        path: String,
    },
    /// Git could not answer; callers gate — never degrade toward trust.
    Unknown {
        /// What failed.
        why: String,
    },
}

/// Judge the merges that delivered `rel` (a repo-relative task file).
#[must_use]
pub fn task_provenance(root: &Path, rel: &str) -> Provenance {
    match compute(root, rel) {
        Ok(p) => p,
        Err(why) => Provenance::Unknown { why },
    }
}

fn compute(root: &Path, rel: &str) -> Result<Provenance, String> {
    let touching = git_lines(root, &["log", "--follow", "--format=%H", "--", rel])?;
    if touching.is_empty() {
        // Never committed: authored in this clone — the operator's own.
        return Ok(Provenance::Trusted);
    }
    let mainline: HashSet<String> = git_lines(root, &["rev-list", "--first-parent", "HEAD"])?
        .into_iter()
        .collect();
    // Oldest→newest so the FIRST match below is the landing merge.
    let merges = git_lines(
        root,
        &[
            "rev-list",
            "--first-parent",
            "--merges",
            "--reverse",
            "HEAD",
        ],
    )?;
    for commit in &touching {
        // The commit's own full delta — catches squash-merges and every
        // other single-commit task+code combo.
        if let Some(path) = commit_offender(root, commit)? {
            return Ok(Provenance::RodeAlong {
                arrival: commit.clone(),
                path,
            });
        }
        // Merge-landed commits: additionally judge the whole delivered
        // group, so task and test split across a PR's inner commits are
        // still seen as one arrival.
        if !mainline.contains(commit) {
            let landing = landing_merge(root, commit, &merges)?;
            if let Some(path) = merge_offender(root, &landing)? {
                return Ok(Provenance::RodeAlong {
                    arrival: landing,
                    path,
                });
            }
        }
    }
    Ok(Provenance::Trusted)
}

/// First non-store path in one commit's own delta. Merge commits judge
/// their whole first-parent delta; root commits diff against empty.
fn commit_offender(root: &Path, commit: &str) -> Result<Option<String>, String> {
    if is_merge(root, commit)? {
        return merge_offender(root, commit);
    }
    let delta = git_lines(
        root,
        &[
            "diff-tree",
            "--no-commit-id",
            "--name-only",
            "-r",
            "--root",
            commit,
        ],
    )?;
    Ok(delta.into_iter().find(|p| !p.starts_with(STORE_PREFIX)))
}

/// The first-parent merge that brought `commit` to mainline: the oldest
/// merge containing it whose mainline parent does not.
fn landing_merge(root: &Path, commit: &str, merges: &[String]) -> Result<String, String> {
    for m in merges {
        if is_ancestor(root, commit, m)? && !is_ancestor(root, commit, &format!("{m}^1"))? {
            return Ok(m.clone());
        }
    }
    Err(format!("no landing merge found for {commit}"))
}

/// First path in a merge's whole first-parent delta that falls outside
/// the store — the ride-along payload, if any.
fn merge_offender(root: &Path, merge: &str) -> Result<Option<String>, String> {
    let delta = git_lines(root, &["diff", "--name-only", &format!("{merge}^1"), merge])?;
    Ok(delta.into_iter().find(|p| !p.starts_with(STORE_PREFIX)))
}

fn is_merge(root: &Path, commit: &str) -> Result<bool, String> {
    let line = git_lines(root, &["rev-list", "--parents", "-n", "1", commit])?;
    Ok(line
        .first()
        .is_some_and(|l| l.split_whitespace().count() > 2))
}

fn is_ancestor(root: &Path, a: &str, b: &str) -> Result<bool, String> {
    let status = Command::new("git")
        .args(["merge-base", "--is-ancestor", a, b])
        .current_dir(root)
        .status()
        .map_err(|e| format!("git merge-base: {e}"))?;
    match status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        other => Err(format!("git merge-base --is-ancestor {a} {b}: {other:?}")),
    }
}

fn git_lines(root: &Path, args: &[&str]) -> Result<Vec<String>, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|e| format!("git {}: {e}", args.join(" ")))?;
    if !out.status.success() {
        return Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}
