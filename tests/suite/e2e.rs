//! E2E scenario tests — the real binary against real git in tempdirs
//! (DESIGN §13). Zero network anywhere (MW-J6).

use crate::common::git;
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

fn meshwork(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("meshwork").unwrap();
    cmd.current_dir(dir);
    cmd
}

/// With no args the binary prints usage and exits 2 — it never pretends.
#[test]
fn no_args_shows_usage_exit_2() {
    let (_g, repo) = git_repo("work");
    meshwork(&repo)
        .assert()
        .code(2)
        .stderr(predicates::str::contains("Usage"));
}

/// PLAN 0.4 / MW-A3, MW-I1: `init` writes the full layout at the git
/// toplevel — config, union merge attribute, cache gitignore — and
/// installs no hooks, touches nothing outside the repo.
#[test]
fn init_layout() {
    let (_g, repo) = git_repo("work");
    let hooks_before = std::fs::read_dir(repo.join(".git/hooks")).map_or(0, Iterator::count);

    meshwork(&repo)
        .arg("init")
        .assert()
        .success()
        .stdout(predicates::str::contains("meshwork/config.toml"));

    let mw = repo.join("meshwork");
    let config = std::fs::read_to_string(mw.join("config.toml")).unwrap();
    assert!(config.contains("alias = \"wo\""), "config: {config}");
    assert!(
        config.contains("default_author = \"Fixture User\""),
        "seeded from git user.name: {config}"
    );
    assert_eq!(
        std::fs::read_to_string(mw.join(".gitattributes")).unwrap(),
        "tasks/*.md merge=union\n",
        "the committed union attr is MW-I1's whole mechanism"
    );
    assert_eq!(
        std::fs::read_to_string(mw.join(".cache/.gitignore")).unwrap(),
        "*\n!.gitignore\n"
    );
    assert!(mw.join("tasks").is_dir());
    assert!(mw.join("attachments").is_dir());

    // MW-A3: no hooks installed, no hooksPath redirection.
    let hooks_after = std::fs::read_dir(repo.join(".git/hooks")).map_or(0, Iterator::count);
    assert_eq!(hooks_before, hooks_after, "no git hooks installed");
    let out = std::process::Command::new("git")
        .args(["config", "core.hooksPath"])
        .current_dir(&repo)
        .output()
        .unwrap();
    assert!(!out.status.success(), "core.hooksPath must stay unset");
}

/// `init` from a subdirectory still writes at the repo root.
#[test]
fn init_from_subdir_writes_at_root() {
    let (_g, repo) = git_repo("work");
    let sub = repo.join("src/deep");
    std::fs::create_dir_all(&sub).unwrap();
    meshwork(&sub).arg("init").assert().success();
    assert!(repo.join("meshwork/config.toml").is_file());
    assert!(!sub.join("meshwork").exists());
}

/// MW-A3: refuses to write anywhere that isn't a git repo.
#[test]
fn init_refuses_outside_git_repo() {
    let dir = tempfile::tempdir().unwrap();
    meshwork(dir.path())
        .arg("init")
        .assert()
        .failure()
        .stderr(predicates::str::contains("git repo"));
    assert!(!dir.path().join("meshwork").exists());
}

/// Re-running init must not clobber an existing store.
#[test]
fn init_twice_refuses() {
    let (_g, repo) = git_repo("work");
    meshwork(&repo).arg("init").assert().success();
    std::fs::write(
        repo.join("meshwork/config.toml"),
        "alias = \"xx\"\n", // hand-edited; init must not overwrite
    )
    .unwrap();
    meshwork(&repo)
        .arg("init")
        .assert()
        .failure()
        .stderr(predicates::str::contains("already"));
    let config = std::fs::read_to_string(repo.join("meshwork/config.toml")).unwrap();
    assert!(config.contains("xx"), "hand-edited config untouched");
}

/// MW-C3: every command supports --json with the stable envelope.
#[test]
fn init_json_envelope() {
    let (_g, repo) = git_repo("work");
    let out = meshwork(&repo).args(["init", "--json"]).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("valid JSON");
    assert_eq!(v["v"], 1);
    assert_eq!(v["verb"], "init");
    assert!(v["data"]["created"].as_array().unwrap().len() >= 4);
}
