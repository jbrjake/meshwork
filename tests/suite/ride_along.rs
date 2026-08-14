//! `ride_along::` — the provenance guard (mw-egksvhm; DESIGN §12b gate
//! routing). Confirmed threat: one PR carries a task plus the test its
//! `run` verify names — merged, the task self-verifies against attacker
//! code at close. Every commit touching the task file is judged by its
//! own full delta (a squash-merge is the whole PR in one commit), and
//! merge-landed commits are ADDITIONALLY judged by the whole landing
//! merge — the merge basis spans larger combined groups, it never
//! exempts single commits. No merge style is forced on contributors.

use crate::common::git;
use meshwork::provenance::{task_provenance, Provenance};
use std::path::Path;

const TASK: &str = "docs/meshwork/mw-t1-task.md";

fn repo_with_base() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("work");
    std::fs::create_dir_all(repo.join("docs/meshwork")).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["config", "user.name", "Fixture User"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    std::fs::write(repo.join("lib.rs"), "base code\n").unwrap();
    commit_all(&repo, "base");
    (dir, repo)
}

fn commit_all(repo: &Path, msg: &str) {
    git(repo, &["add", "-A"]);
    git(repo, &["commit", "-qm", msg]);
}

fn write_task(repo: &Path, rel: &str) {
    std::fs::write(
        repo.join(rel),
        "---\nid: mw-t1\ntitle: T\nstatus: open\nverify: run cargo test t\n---\n",
    )
    .unwrap();
}

/// Store-only history stays frictionless: a task committed alone, and
/// an uncommitted task, both pass.
#[test]
fn ride_along_store_only_direct_trusted() {
    let (_g, repo) = repo_with_base();
    write_task(&repo, TASK);
    commit_all(&repo, "chore(store): file the task");
    assert!(
        matches!(task_provenance(&repo, TASK), Provenance::Trusted),
        "store-only direct commit carries no payload"
    );

    let fresh = "docs/meshwork/mw-t2-fresh.md";
    write_task(&repo, fresh);
    assert!(
        matches!(task_provenance(&repo, fresh), Provenance::Trusted),
        "uncommitted task = authored in this clone"
    );
}

/// A squash-merge is the whole PR in ONE linear commit — task and test
/// together in its delta. It must gate; single-commit combos are never
/// exempt (ruling 2026-08-14: the merge basis widens, it never narrows).
#[test]
fn ride_along_squash_single_commit_gates() {
    let (_g, repo) = repo_with_base();
    git(&repo, &["checkout", "-qb", "pr"]);
    std::fs::write(repo.join("evil_test.rs"), "malicious test\n").unwrap();
    commit_all(&repo, "innocent-looking test");
    write_task(&repo, TASK);
    commit_all(&repo, "task naming that test");
    git(&repo, &["checkout", "-q", "main"]);
    git(&repo, &["merge", "-q", "--squash", "pr"]);
    git(&repo, &["commit", "-qm", "squashed pr"]);

    match task_provenance(&repo, TASK) {
        Provenance::RodeAlong { path, .. } => {
            assert_eq!(path, "evil_test.rs", "the offender is named");
        }
        other => panic!("squash-merged task+test must gate: {other:?}"),
    }
}

/// THE threat: a merge whose delta carries the task AND code — even in
/// separate inner commits — gates the task's run verifies.
#[test]
fn ride_along_mixed_merge_gates() {
    let (_g, repo) = repo_with_base();
    git(&repo, &["checkout", "-qb", "pr"]);
    std::fs::write(repo.join("evil_test.rs"), "malicious test\n").unwrap();
    commit_all(&repo, "innocent-looking test");
    write_task(&repo, TASK);
    commit_all(&repo, "task naming that test");
    git(&repo, &["checkout", "-q", "main"]);
    git(&repo, &["merge", "-q", "--no-ff", "-m", "merge pr", "pr"]);

    match task_provenance(&repo, TASK) {
        Provenance::RodeAlong { path, .. } => {
            assert_eq!(path, "evil_test.rs", "the offender is named");
        }
        other => panic!("split-commit PR must still gate: {other:?}"),
    }
}

/// A store-only merge is fine — until a later mixed merge touches the
/// task file; every commit touching the file counts.
#[test]
fn ride_along_store_only_merge_trusted() {
    let (_g, repo) = repo_with_base();
    git(&repo, &["checkout", "-qb", "pr1"]);
    write_task(&repo, TASK);
    commit_all(&repo, "task only");
    git(&repo, &["checkout", "-q", "main"]);
    git(&repo, &["merge", "-q", "--no-ff", "-m", "merge pr1", "pr1"]);
    assert!(
        matches!(task_provenance(&repo, TASK), Provenance::Trusted),
        "store-only merge carries no payload"
    );

    git(&repo, &["checkout", "-qb", "pr2"]);
    std::fs::write(
        repo.join(TASK),
        "---\nid: mw-t1\ntitle: T\nstatus: open\nverify: run cargo test evil\nseq: 5\n---\n",
    )
    .unwrap();
    commit_all(&repo, "retarget the verify");
    std::fs::write(repo.join("evil_test.rs"), "malicious test\n").unwrap();
    commit_all(&repo, "the payload");
    git(&repo, &["checkout", "-q", "main"]);
    git(&repo, &["merge", "-q", "--no-ff", "-m", "merge pr2", "pr2"]);
    assert!(
        matches!(task_provenance(&repo, TASK), Provenance::RodeAlong { .. }),
        "a later mixed merge re-poisons the task"
    );
}

/// No git answer → gate, never trust.
#[test]
fn ride_along_unreadable_gates() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("docs/meshwork")).unwrap();
    write_task(dir.path(), TASK);
    assert!(
        matches!(
            task_provenance(dir.path(), TASK),
            Provenance::Unknown { .. }
        ),
        "outside git, the guard degrades toward gating"
    );
}
