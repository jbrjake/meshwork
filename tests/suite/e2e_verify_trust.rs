// mw-9rc4vs6 (MW-E5, DESIGN §12b): TOFU trust gate — close refuses to run
// a shell verify: whose exact text this clone's operator hasn't approved.
// Approvals are content hashes in gitignored .cache/ (never merge in);
// MESHWORK_TRUST=1 is the deliberate reviewed-checkout grant. The shared
// `meshwork()` helper sets MESHWORK_TRUST=1 so the rest of the suite runs
// ungated; tests here strip it to stand on the untrusted default.

fn untrusted(repo: &Path) -> Command {
    let mut cmd = meshwork(repo);
    cmd.env_remove("MESHWORK_TRUST");
    cmd
}

/// The drive-by path is closed: an unapproved shell verify refuses loudly,
/// names the approval step, runs nothing, and leaves the task untouched.
#[test]
fn verify_trust_gate_refuses_unapproved() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let marker = repo.join("pwned");
    let id = add_id(
        &repo,
        &[
            "add",
            "malicious import",
            "--verify",
            &format!("touch {}", marker.display()),
        ],
    );

    let assert = untrusted(&repo).args(["close", &id]).assert().failure();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("MW-E5"), "names the requirement: {err}");
    assert!(err.contains("--approve"), "names the approval step: {err}");
    assert!(err.contains("touch"), "verify text on screen: {err}");
    assert!(!marker.exists(), "nothing executed");

    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: open"), "task untouched: {text}");
}

/// `--approve` shows the text, records the approval for this clone, runs —
/// and the approval is remembered: a later close needs no flag. The state
/// lives under gitignored .cache/, invisible to git.
#[test]
fn verify_trust_approve_records_then_remembers() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "seed"]);
    let id = add_id(&repo, &["add", "trusted work", "--verify", "true"]);

    let assert = untrusted(&repo)
        .args(["close", &id, "--approve"])
        .assert()
        .success();
    let out = stdout_of(&assert);
    assert!(out.contains("true"), "approved text on screen: {out}");

    // Approval state is clone-local: inside .cache/, gitignored.
    let status = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&repo)
        .output()
        .unwrap();
    let status = String::from_utf8(status.stdout).unwrap();
    assert!(
        !status.contains(".cache"),
        "approval never reaches git: {status}"
    );

    // Same (id, text) → no re-approval needed after reopen.
    untrusted(&repo).args(["reopen", &id]).assert().success();
    untrusted(&repo).args(["close", &id]).assert().success();
}

/// A changed verify text revokes trust — the hash covers the exact text,
/// so a merged edit to an approved task re-enters the gate.
#[test]
fn verify_trust_changed_text_revokes() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "mutating verify", "--verify", "true"]);
    untrusted(&repo)
        .args(["close", &id, "--approve"])
        .assert()
        .success();
    untrusted(&repo).args(["reopen", &id]).assert().success();

    let path = task_file(&repo, &id);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, text.replace("verify: \"true\"", "verify: \"false\"")).unwrap();

    let assert = untrusted(&repo).args(["close", &id]).assert().failure();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("MW-E5"), "changed text re-gates: {err}");
}

/// `MESHWORK_TRUST=1` is the reviewed-checkout grant (CI, the gate): no
/// approval state, verify still runs.
#[test]
fn verify_trust_env_grant_for_ci() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "ci path", "--verify", "true"]);
    untrusted(&repo)
        .env("MESHWORK_TRUST", "1")
        .args(["close", &id])
        .assert()
        .success();
}

/// `--waive` never shells out, so it never needs trust (MW-E2 loudness is
/// its own gate); and deleting .cache drops approvals — conservative,
/// re-approve, never an error (MW-A2: cache is never a dependency).
#[test]
fn verify_trust_waive_and_cache_delete() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let a = add_id(&repo, &["add", "waived", "--verify", "true"]);
    untrusted(&repo)
        .args(["close", &a, "--waive", "not worth running"])
        .assert()
        .success();

    let b = add_id(&repo, &["add", "cache dropped", "--verify", "true"]);
    untrusted(&repo)
        .args(["close", &b, "--approve"])
        .assert()
        .success();
    untrusted(&repo).args(["reopen", &b]).assert().success();
    std::fs::remove_dir_all(repo.join("docs/meshwork/.cache")).unwrap();
    untrusted(&repo).args(["close", &b]).assert().failure();
    untrusted(&repo)
        .args(["close", &b, "--approve"])
        .assert()
        .success();
}
