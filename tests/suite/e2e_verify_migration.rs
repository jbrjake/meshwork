// e2e part-file: DSL migration wiring (mw-4aqmf0t; DESIGN §12b gate
// routing). close/start route every verify through the classifier:
// native predicates run ungated, `run` runs free only on store-only
// provenance, malformed refuses without ever gate-prompting, legacy
// shell keeps the MW-E5 gate — and `run cargo test` demands an observed
// pass (owner ruling 2026-08-14: a filter matching nothing exits 0).

/// A fake `cargo` on PATH: prints the given summary line, exits 0. The
/// real cargo would drag a workspace into a tempdir e2e; routing and the
/// vacuity rule are what's under test, not cargo itself.
fn stub_cargo(dir: &Path, summary: &str) -> String {
    use std::os::unix::fs::PermissionsExt;
    let bin = dir.join("stubbin");
    std::fs::create_dir_all(&bin).unwrap();
    let path = bin.join("cargo");
    std::fs::write(&path, format!("#!/bin/sh\necho \"{summary}\"\nexit 0\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    format!("{}:{}", bin.display(), std::env::var("PATH").unwrap())
}

fn head_hash(repo: &Path) -> String {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(repo)
        .output()
        .unwrap();
    String::from_utf8(out.stdout).unwrap().trim().to_string()
}

/// Native predicates (exists/contains) load no code: they run with no
/// approval, no trust env, no provenance question — and a failing one
/// fails the close as a verify failure, never as a gate refusal.
#[test]
fn verify_migration_native_dsl_ungated() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    let green = add_id(
        &repo,
        &["add", "native green", "--verify", "exists docs/meshwork/config.toml"],
    );
    untrusted(&repo).args(["close", &green]).assert().success();

    let red = add_id(&repo, &["add", "native red", "--verify", "exists no/such/file.md"]);
    let assert = untrusted(&repo).args(["close", &red]).assert().failure();
    let err = stderr_of(&assert);
    assert!(err.contains("no such path"), "verify failure, named: {err}");
    assert!(!err.contains("--approve"), "not a gate refusal: {err}");
    let text = std::fs::read_to_string(task_file(&repo, &red)).unwrap();
    assert!(text.contains("status: open"), "stays open: {text}");
}

/// `run` on store-only provenance is the frictionless path the DSL
/// exists for: committed alone under docs/meshwork/, it executes with
/// no approval at all.
#[test]
fn verify_migration_run_store_only_ungated() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "run free", "--verify", "run cargo test t"]);
    git(&repo, &["add", "docs/meshwork"]);
    git(&repo, &["commit", "-qm", "chore(store): task only"]);

    let path = stub_cargo(&repo, "test result: ok. 2 passed; 0 failed");
    untrusted(&repo)
        .env("PATH", &path)
        .args(["close", &id])
        .assert()
        .success();
}

/// Owner ruling 2026-08-14: `cargo test` with a filter matching nothing
/// exits 0 — a vacuous pass. The runner demands an observed `ok. N
/// passed`, N ≥ 1; zero tests run means the verify fails even fully
/// trusted.
#[test]
fn verify_migration_run_vacuous_zero_passed_fails() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "vacuous", "--verify", "run cargo test nothing"]);

    let path = stub_cargo(&repo, "test result: ok. 0 passed; 0 failed");
    let assert = meshwork(&repo)
        .env("PATH", &path)
        .args(["close", &id])
        .assert()
        .failure();
    let err = stderr_of(&assert);
    assert!(err.contains("vacuous"), "names the failure class: {err}");
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: open"), "stays open: {text}");
}

/// THE threat (DESIGN §12b): task and code delivered together. The
/// refusal names the arrival and the offending path; `--approve` is the
/// reviewed-it escape, and the approval is remembered like any MW-E5
/// grant.
#[test]
fn verify_migration_run_rode_along_gates() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "rode along", "--verify", "run cargo test t"]);
    std::fs::write(repo.join("payload.rs"), "code that arrived with the task\n").unwrap();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "task plus payload in one commit"]);
    let arrival = head_hash(&repo);

    let path = stub_cargo(&repo, "test result: ok. 1 passed; 0 failed");
    let assert = untrusted(&repo)
        .env("PATH", &path)
        .args(["close", &id])
        .assert()
        .failure();
    let err = stderr_of(&assert);
    assert!(err.contains(&arrival), "names the arrival: {err}");
    assert!(err.contains("payload.rs"), "names the offender: {err}");
    assert!(err.contains("--approve"), "names the escape: {err}");

    untrusted(&repo)
        .env("PATH", &path)
        .args(["close", &id, "--approve"])
        .assert()
        .success();
    untrusted(&repo).args(["reopen", &id]).assert().success();
    untrusted(&repo)
        .env("PATH", &path)
        .args(["close", &id])
        .assert()
        .success();
}

/// Keyword-led text that does not parse REFUSES — trusted or not, with
/// or without --approve. A silent downgrade to shell would reopen the
/// drive-by hole; a gate prompt would invite approving garbage.
#[test]
fn verify_migration_malformed_refuses() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "malformed", "--verify", "exists /etc/passwd"]);

    for extra in [None, Some("--approve")] {
        let mut cmd = meshwork(&repo);
        cmd.args(["close", &id]);
        if let Some(flag) = extra {
            cmd.arg(flag);
        }
        let assert = cmd.assert().failure();
        let err = stderr_of(&assert);
        assert!(err.contains("malformed"), "refuses loudly: {err}");
        assert!(!err.contains("--approve\n"), "never gate-prompts: {err}");
    }
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: open"), "task untouched: {text}");
}

/// start's red-check routes identically: a native DSL verify executes
/// untrusted (pure read), and one already green warns.
#[test]
fn verify_migration_redcheck_native_dsl() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(
        &repo,
        &["add", "green dsl", "--verify", "exists docs/meshwork/config.toml"],
    );
    let assert = untrusted(&repo).args(["start", &id]).assert().success();
    let err = stderr_of(&assert);
    assert!(err.contains("already green"), "native DSL runs untrusted: {err}");
}

/// Lint pressure (the task's second half): legacy shell verifies warn
/// `verify-shell` on live tasks, malformed ones warn `verify-malformed`;
/// DSL verifies wear neither.
#[test]
fn verify_migration_lint_pressure() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let shell = add_id(&repo, &["add", "legacy", "--verify", "grep -q foo README.md"]);
    let broken = add_id(&repo, &["add", "broken", "--verify", "contains"]);
    let dsl = add_id(&repo, &["add", "clean", "--verify", "run cargo test t"]);

    let js = stdout_of(&meshwork(&repo).args(["lint", "--json"]).assert().success());
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let has = |code: &str, subject: &str| {
        v["data"]["findings"].as_array().unwrap().iter().any(|f| {
            f["code"] == code && f["subject"] == subject.trim_end() && f["severity"] == "warning"
        })
    };
    assert!(has("verify-shell", &shell), "{js}");
    assert!(has("verify-malformed", &broken), "{js}");
    let dsl_rows = v["data"]["findings"].as_array().unwrap().iter().any(|f| {
        f["subject"] == dsl && f["code"].as_str().unwrap().starts_with("verify-")
    });
    assert!(!dsl_rows, "DSL verify wears no verify-* warning: {js}");
}
