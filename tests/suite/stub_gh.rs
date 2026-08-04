//! `stub_gh::` — the offline `gh` double (PLAN B4, MW-J6). It records every
//! invocation, replays canned JSON, refuses tests that didn't opt in, and
//! hard-fails on anything mutating (MW-H2 enforced at the boundary).

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn run_stub(args: &[&str], stdin: &str, calls: Option<&Path>) -> Output {
    let mut cmd = Command::new(repo_root().join("tests/bin/gh"));
    cmd.args(args)
        .env("GH_STUB_CANNED", repo_root().join("tests/canned"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    match calls {
        Some(p) => {
            cmd.env("GH_STUB_CALLS", p);
        }
        None => {
            cmd.env_remove("GH_STUB_CALLS");
        }
    }
    let mut child = cmd.spawn().expect("stub gh spawns");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn records_argv_and_stdin() {
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("gh.calls");
    let out = run_stub(
        &["issue", "list", "--search", "meshwork:t:az-j6h5"],
        "line one\nline two\n",
        Some(&calls),
    );
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let rec = fs::read_to_string(&calls).unwrap();
    assert_eq!(
        rec,
        "$ issue list --search meshwork:t:az-j6h5\n> line one\n> line two\n"
    );
}

#[test]
fn appends_across_calls() {
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("gh.calls");
    run_stub(&["issue", "list"], "", Some(&calls));
    run_stub(&["issue", "list"], "", Some(&calls));
    let rec = fs::read_to_string(&calls).unwrap();
    assert_eq!(rec, "$ issue list\n$ issue list\n");
}

#[test]
fn replays_canned_json() {
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("gh.calls");
    let out = run_stub(&["issue", "list"], "", Some(&calls));
    assert!(out.status.success());
    let canned = fs::read(repo_root().join("tests/canned/issue-list.json")).unwrap();
    assert_eq!(
        out.stdout, canned,
        "stdout must be the canned bytes, verbatim"
    );
}

#[test]
fn unknown_call_fails_loudly() {
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("gh.calls");
    let out = run_stub(&["release", "upload", "v1", "blob.tgz"], "", Some(&calls));
    assert_eq!(out.status.code(), Some(64));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("no canned response"), "stderr: {err}");
    // Still recorded — an unexpected call must leave evidence.
    assert!(fs::read_to_string(&calls)
        .unwrap()
        .starts_with("$ release upload"));
}

#[test]
fn refuses_without_calls_env() {
    let out = run_stub(&["issue", "list"], "", None);
    assert_eq!(
        out.status.code(),
        Some(66),
        "a gh call from a test that never opted in is itself a bug (MW-H5/J6)"
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("GH_STUB_CALLS"));
}

#[test]
fn mutation_guard_trips() {
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("gh.calls");
    for argv in [
        vec!["issue", "edit", "12", "--title", "x"],
        vec!["issue", "close", "12"],
        vec!["issue", "delete", "12"],
        vec!["issue", "reopen", "12"],
        vec!["api", "-X", "DELETE", "repos/o/r/issues/12"],
        vec!["api", "--method", "PATCH", "repos/o/r/issues/12"],
    ] {
        let out = run_stub(&argv, "", Some(&calls));
        assert_eq!(
            out.status.code(),
            Some(65),
            "must trip MW-H2 guard: gh {argv:?}"
        );
        assert!(String::from_utf8_lossy(&out.stderr).contains("MUTATION"));
    }
    // Every attempt recorded before refusal — forensics for never-mutate.
    assert_eq!(fs::read_to_string(&calls).unwrap().lines().count(), 6);
}

#[test]
fn append_methods_stay_allowed() {
    let dir = tempfile::tempdir().unwrap();
    let calls = dir.path().join("gh.calls");
    let out = run_stub(
        &["api", "--method", "POST", "repos/o/r/issues/12/comments"],
        "",
        Some(&calls),
    );
    // POST is append (comment/issue creation) — allowed; api-POST is canned.
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
}

#[test]
fn harness_path_resolves_stub() {
    // The harness contract (gate §3 does the same): tests/bin prepended to
    // PATH makes `gh` resolve to the stub, never the real binary.
    let path = format!(
        "{}:{}",
        repo_root().join("tests/bin").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new("sh")
        .args(["-c", "command -v gh"])
        .env("PATH", path)
        .output()
        .unwrap();
    let resolved = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        resolved.trim(),
        repo_root().join("tests/bin/gh").to_string_lossy(),
        "gh must resolve to the stub"
    );
}
