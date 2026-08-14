// e2e part-file: capture-before-verifiable (mw-6wdpz1b). Included by e2e.rs.

/// mw-6wdpz1b (owner-requested): a green `true` is worse than an honest
/// "not verifiable yet" — capture without a `verify:` stays legal, but the
/// task cannot START until its done-test exists, and ready/prime keep the
/// gap loud (needs-verify) so writing the verify stays the visible next
/// action. The lint warning (MW-E2) and `close --waive` are unchanged.
#[test]
fn needs_verify() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    // Capture without a verify is legal (ideas are cheaper than
    // implementations) — the task files fine.
    let out = meshwork(&repo).args(["add", "Idea only"]).assert().success();
    let id = stdout_of(&out).lines().next().unwrap().to_string();

    // …but it cannot start: writing the verify IS the first unit of work.
    let assert = meshwork(&repo).args(["start", &id]).assert().code(1);
    let err = stderr_of(&assert);
    assert!(err.contains(&id) && err.contains("verify"), "{err}");
    assert!(err.contains("needs-verify"), "names the state: {err}");
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: open"), "refusal writes nothing: {text}");
    assert!(!text.contains("open\u{2192}doing"), "no log entry: {text}");

    // ready and prime surface the gap loudly.
    let ready = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    let line = ready.lines().find(|l| l.contains(&id)).expect("ready row");
    assert!(line.contains("[needs-verify]"), "{ready}");
    let js = stdout_of(&meshwork(&repo).args(["ready", "--json"]).assert().success());
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["rows"][0]["needs_verify"], true, "{js}");
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains("needs-verify"), "{prime}");

    // A hand-edited verify: line (the pre-set-fields path) unlocks start.
    let path = task_file(&repo, &id);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replace("status: open", "status: open\nverify: \"true\""),
    )
    .unwrap();
    meshwork(&repo).args(["start", &id]).assert().success();

    // Verified tasks carry no annotation anywhere.
    let id2 = add_task(&repo, "Proper");
    let ready = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    let line = ready.lines().find(|l| l.contains(&id2)).expect("ready row");
    assert!(!line.contains("needs-verify"), "{ready}");
    let js = stdout_of(&meshwork(&repo).args(["ready", "--json"]).assert().success());
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["rows"][0]["needs_verify"], false, "{js}");
}

/// mw-175bn4c: a verify already green at start cannot detect the work.
/// The red-check is advisory (mw-kkvs8zq precedent: a warning is
/// behavior, no new surface) and executes only text this clone already
/// trusts (MW-E5) — untrusted verifies never run; the skip says so.
#[test]
fn verify_red_check() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    // Green at start: the warning names the class; the start proceeds.
    let green = add_id(&repo, &["add", "Green verify", "--verify", "true"]);
    let assert = meshwork(&repo).args(["start", &green]).assert().success();
    let err = stderr_of(&assert);
    assert!(err.contains("already green"), "{err}");

    // Red at start: armed — the check stays quiet.
    let red = add_id(&repo, &["add", "Red verify", "--verify", "false"]);
    let assert = meshwork(&repo).args(["start", &red]).assert().success();
    assert!(
        !stderr_of(&assert).contains("red-check"),
        "armed verify is quiet"
    );

    // Exit 127: close's shell can't even run it — say so, proceed.
    let broken = add_id(&repo, &["add", "Broken verify", "--verify", "no-such-cmd-mw175"]);
    let assert = meshwork(&repo).args(["start", &broken]).assert().success();
    let err = stderr_of(&assert);
    assert!(err.contains("127"), "{err}");

    // Untrusted text never executes — the skip is loud, the start proceeds.
    let cold = add_id(&repo, &["add", "Cold clone", "--verify", "true"]);
    let assert = meshwork(&repo)
        .env_remove("MESHWORK_TRUST")
        .args(["start", &cold])
        .assert()
        .success();
    let err = stderr_of(&assert);
    assert!(err.contains("red-check skipped"), "{err}");
    assert!(!err.contains("already green"), "did not execute: {err}");
}
