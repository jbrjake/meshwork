// mw-dx4pndb (§6 ruling 2026-08-22): `verify <id>` runs the task's
// verify: under close's exact §12b gate routing and reports the verdict —
// closes nothing, releases nothing, writes nothing. Red-first authoring
// and rot-sweeps stop hand-rolling the extraction loop in the interactive
// shell (the authoring-shell mismatch that shipped 28 fail-closed
// rg-verifies). Single-task only — no sweep flag, by the same ruling.

/// Green verify reports exit 0 and leaves the file byte-identical; red
/// verify exits nonzero with the exit on stderr and still writes nothing —
/// unlike close, not even an attempt line.
#[test]
fn verify_verb_reports_without_closing() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    let green = add_task(&repo, "Green dry run"); // verify: true
    let before = std::fs::read_to_string(task_file(&repo, &green)).unwrap();
    meshwork(&repo)
        .args(["verify", &green])
        .assert()
        .success()
        .stdout(predicates::str::contains("exit 0"));
    let after = std::fs::read_to_string(task_file(&repo, &green)).unwrap();
    assert_eq!(before, after, "dry run never writes");
    assert!(after.contains("status: open"), "nothing closed: {after}");

    let red = add_id(&repo, &["add", "Red dry run", "--verify", "exit 3"]);
    let before = std::fs::read_to_string(task_file(&repo, &red)).unwrap();
    meshwork(&repo)
        .args(["verify", &red])
        .assert()
        .failure()
        .stderr(predicates::str::contains("exit 3"));
    let after = std::fs::read_to_string(task_file(&repo, &red)).unwrap();
    assert_eq!(before, after, "no attempt line on a dry run: {after}");
}

/// The dry run is also the rot-sweep: done tasks run too — no status
/// gate, because a rotted verify on a closed task is the sweep's quarry.
#[test]
fn verify_verb_runs_on_done_tasks() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Closed then swept");
    meshwork(&repo).args(["close", &id]).assert().success();
    meshwork(&repo)
        .args(["verify", &id])
        .assert()
        .success()
        .stdout(predicates::str::contains("exit 0"));
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: done"), "still done: {text}");
}

/// No verify: on the task — nothing to run is an error, not a silent pass.
#[test]
fn verify_verb_no_verify_errors() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "No done-test yet"]);
    meshwork(&repo)
        .args(["verify", &id])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no verify"));
}

/// MW-C3: the envelope carries the verdict and says nothing closed.
#[test]
fn verify_verb_json_envelope() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Json dry run");
    let js = stdout_of(
        &meshwork(&repo)
            .args(["verify", &id, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["verb"], "verify");
    assert_eq!(v["data"]["id"], serde_json::json!(id));
    assert_eq!(v["data"]["verify_exit"], 0);
    assert_eq!(v["data"]["closed"], false);
}

/// The §12b gates are close's exactly: an unapproved legacy-shell verify
/// refuses in the dry run too, runs nothing, and the refusal still names
/// close's approval step — approval stays a close-side act. The
/// hand-edit is the merge stand-in (minted text is pre-approved).
#[test]
fn verify_verb_gates_like_close() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let marker = repo.join("pwned");
    let id = add_id(&repo, &["add", "merged-in verify", "--verify", "true"]);
    let path = task_file(&repo, &id);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replace(
            "verify: \"true\"",
            &format!("verify: \"touch {}\"", marker.display()),
        ),
    )
    .unwrap();
    let assert = untrusted(&repo).args(["verify", &id]).assert().failure();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        err.contains("refusing unapproved verify"),
        "same refusal as close: {err}"
    );
    assert!(err.contains("--approve"), "names the approval step: {err}");
    assert!(!marker.exists(), "nothing executed");
}
