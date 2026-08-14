//! `ride_along::` — the merge-unit provenance guard (mw-egksvhm; DESIGN
//! §12b gate routing). Confirmed threat: one PR carries a task plus the
//! test its `run` verify names — merged, the task self-verifies against
//! attacker code at close. The guard judges the MERGE that delivered
//! each task-file commit, never the commit alone; direct first-parent
//! commits are the operator's own and pass without content judgment.

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

/// Direct first-parent commits are the operator's own: task+code in one
/// commit (this repo's close ritual) and uncommitted tasks both pass.
#[test]
fn ride_along_direct_commits_trusted() {
    let (_g, repo) = repo_with_base();
    write_task(&repo, TASK);
    std::fs::write(repo.join("feature.rs"), "operator code\n").unwrap();
    commit_all(&repo, "feat: work + task flip together");
    assert!(
        matches!(task_provenance(&repo, TASK), Provenance::Trusted),
        "operator's own task+code commit must stay frictionless"
    );

    let fresh = "docs/meshwork/mw-t2-fresh.md";
    write_task(&repo, fresh);
    assert!(
        matches!(task_provenance(&repo, fresh), Provenance::Trusted),
        "uncommitted task = authored in this clone"
    );
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
