// mw-ntn0t32: "what work went into closing this" without archaeology.
// Close-side, the →done note gains an ` @ <short-sha>[+N]` anchor (HEAD at
// close time — the closing commit lands after, so this names its parent;
// +N counts uncommitted paths). Read-side, `show` derives the commit set
// from the existing id-in-subject convention via `git log --grep` — zero
// network, works retroactively for every task ever closed that way. No
// hooks (MW-A3), no new verbs (§6).

/// The anchor lands on close, and show recovers the closing commits.
#[test]
fn commit_trace() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "seed"]);
    let sha = {
        let out = std::process::Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .current_dir(&repo)
            .output()
            .unwrap();
        String::from_utf8(out.stdout).unwrap().trim().to_string()
    };

    let id = add_task(&repo, "traced work");
    meshwork(&repo).args(["close", &id]).assert().success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    // The store edit itself is uncommitted at close time → dirty marker.
    assert!(
        text.contains(&format!("verify exit 0 @ {sha}+")),
        "anchor with dirty count: {text}"
    );

    // The id-in-subject convention recovers the closing work.
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", &format!("feat: close out widget ({id})")]);
    git(&repo, &["commit", "-q", "--allow-empty", "-m", "chore: unrelated noise"]);

    let show = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(show.contains("commits ("), "{show}");
    assert!(show.contains("close out widget"), "{show}");
    assert!(!show.contains("unrelated noise"), "{show}");

    let js = stdout_of(
        &meshwork(&repo)
            .args(["show", &id, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let commits = v["data"]["commits"].as_array().unwrap();
    assert_eq!(commits.len(), 1, "{commits:?}");
    assert!(commits[0]["subject"]
        .as_str()
        .unwrap()
        .contains("close out widget"));

    // No matching commits → no section, no empty stub.
    let other = add_task(&repo, "untraced");
    let show = stdout_of(&meshwork(&repo).args(["show", &other]).assert().success());
    assert!(!show.contains("commits ("), "{show}");
}

/// Degradation is omission: an unborn HEAD (fresh repo, nothing committed)
/// closes without an anchor and shows without a commits section.
#[test]
fn commit_trace_unborn_head_omits_anchor() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "no history yet");
    meshwork(&repo).args(["close", &id]).assert().success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    let done_line = text.lines().find(|l| l.contains("→done")).unwrap();
    assert!(
        done_line.trim_end().ends_with("verify exit 0"),
        "no anchor without HEAD: {done_line}"
    );
    meshwork(&repo).args(["show", &id]).assert().success();
}
