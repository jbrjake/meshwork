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
