//! E2E scenario tests — the real binary against real git in tempdirs
//! (DESIGN §13). Zero network anywhere (MW-J6).
//!
//! Tests live in `include!`d part-files so test paths stay flat
//! (`e2e::<name>` — TRACE.md and gate §6 grep full paths) while each file
//! respects the 500/750 line caps.

use crate::common::{copy_dir, fixtures_root, git};
use assert_cmd::Command;
use std::path::Path;

/// Fresh tempdir with an initialized git repo inside, isolated from the
/// machine's git config.
fn git_repo(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join(name);
    std::fs::create_dir_all(&repo).unwrap();
    git(&repo, &["init", "-q"]);
    git(&repo, &["config", "user.name", "Fixture User"]);
    git(&repo, &["config", "user.email", "fixture@example.invalid"]);
    (dir, repo)
}

/// Copy a fixture repo into a tempdir and git-init it — the binary demands
/// a git toplevel, and the committed corpus is never touched (DESIGN §13).
fn fixture_repo(name: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join(name);
    copy_dir(&fixtures_root().join(name), &repo);
    git(&repo, &["init", "-q"]);
    (dir, repo)
}

fn meshwork(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("meshwork").unwrap();
    cmd.current_dir(dir);
    cmd
}

fn init_store(repo: &Path) {
    meshwork(repo).arg("init").assert().success();
}

fn stdout_of(assert: &assert_cmd::assert::Assert) -> String {
    String::from_utf8(assert.get_output().stdout.clone()).unwrap()
}

fn task_file(repo: &Path, id: &str) -> std::path::PathBuf {
    // Root first, then archive/ — terminal tasks move there (mw-45e2qf4).
    ["docs/meshwork", "docs/meshwork/archive"]
        .iter()
        .filter_map(|dir| std::fs::read_dir(repo.join(dir)).ok())
        .flatten()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(&format!("{id}-"))
        })
        .unwrap_or_else(|| panic!("no file for {id}"))
}

fn add_task(repo: &Path, title: &str) -> String {
    add_id(repo, &["add", title, "--verify", "true"])
}

fn add_id(repo: &Path, args: &[&str]) -> String {
    let out = meshwork(repo).args(args).assert().success();
    stdout_of(&out).lines().next().unwrap().to_string()
}

include!("e2e_archive.rs");
include!("e2e_claim.rs");
include!("e2e_graph.rs");
include!("e2e_lifecycle.rs");
include!("e2e_lint.rs");
include!("e2e_merge.rs");
include!("e2e_notes.rs");
include!("e2e_prime.rs");
include!("e2e_query.rs");
include!("e2e_set.rs");
