// e2e part-file: graph verbs — dep add/rm, tree/why/blocked (M1).
// Included by e2e.rs.

/// PLAN 1.1 / MW-B1: edge edits without opening the file — validated,
/// one-line diffs, reflected in the SQL tables.
#[test]
fn dep_edit() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let tx = add_task(&repo, "X depends on things");
    let ty = add_task(&repo, "Y the dependency");
    let tz = add_task(&repo, "Z another dependency");

    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", &ty])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &tx)).unwrap();
    assert!(text.contains(&format!("needs: [{ty}]")), "{text}");

    // Ready now excludes x (y is open).
    let ready = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    assert!(!ready.lines().any(|l| l.starts_with(&tx)), "{ready}");

    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", &tz])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &tx)).unwrap();
    assert!(text.contains(&format!("needs: [{ty}, {tz}]")), "{text}");

    // Guardrails: duplicates, self-deps, and dangling targets refuse.
    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", &ty])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already"));
    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", &tx])
        .assert()
        .failure();
    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", "wo-none"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("wo-none"));
    // Cross-repo targets are the registry's business — allowed here.
    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", "beta#bz-c0r3"])
        .assert()
        .success();

    // The edge lands in the tables.
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", &format!("SELECT count(*) FROM edges WHERE src_gid='work#{tx}' AND kind='needs'")])
            .assert()
            .success(),
    );
    assert!(q.contains('3'), "{q}");

    // rm: down to one, then to none — the key line disappears entirely.
    meshwork(&repo)
        .args(["dep", "rm", &tx, "--needs", &ty])
        .assert()
        .success();
    meshwork(&repo)
        .args(["dep", "rm", &tx, "--needs", "beta#bz-c0r3"])
        .assert()
        .success();
    let json = stdout_of(
        &meshwork(&repo)
            .args(["dep", "rm", &tx, "--needs", &tz, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["verb"], "dep");
    let text = std::fs::read_to_string(task_file(&repo, &tx)).unwrap();
    assert!(!text.contains("needs:"), "empty list drops the key: {text}");

    meshwork(&repo)
        .args(["dep", "rm", &tx, "--needs", &ty])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not"));
}
