// e2e part-file: lint / lint --fix (PLAN 0.9). Included by e2e.rs.

/// PLAN 0.9 / MW-A4, A6, B2, B3, I2: `lint` finds every planted failure in
/// the broken corpus (golden-pinned), and `--fix` repairs exactly the
/// mechanical damage — duplicate keys, duplicate IDs, missing union
/// attributes (mw-mtn4hp8) — leaving real modeling errors for humans.
#[test]
fn lint_broken_corpus() {
    let (_g, repo) = fixture_repo("alpha-broken");
    let assert = meshwork(&repo).args(["lint", "--json"]).assert().code(1);
    let js = stdout_of(&assert);
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let codes: Vec<&str> = v["data"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["code"].as_str().unwrap())
        .collect();
    for expected in [
        "cycle-needs",
        "cycle-parent",
        "parent-crossrepo",
        "blocked-no-reason",
        "duplicate-id",
        "duplicate-key",
        "parse",
        "dangling",
        "unknown-key",
    ] {
        assert!(codes.contains(&expected), "missing {expected}: {codes:?}");
    }
    crate::common::assert_golden("lint-broken.json", &js);

    // --fix repairs union poison + duplicate IDs, and only those.
    meshwork(&repo)
        .env("MESHWORK_ID_SEED", "42")
        .args(["lint", "--fix"])
        .assert()
        .code(1); // modeling errors (cycles, …) remain — still exit 1

    let after = stdout_of(&meshwork(&repo).args(["lint", "--json"]).assert().code(1));
    let v: serde_json::Value = serde_json::from_str(&after).unwrap();
    let codes: Vec<&str> = v["data"]["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["code"].as_str().unwrap())
        .collect();
    assert!(!codes.contains(&"duplicate-key"), "union poison repaired: {codes:?}");
    assert!(!codes.contains(&"duplicate-id"), "duplicate re-slugged: {codes:?}");
    assert!(codes.contains(&"cycle-needs"), "real errors stay: {codes:?}");

    // The union-poisoned file kept its first status and logged the repair
    // (the log line quotes the dropped value, so scope checks to the fm).
    let un10 = std::fs::read_to_string(task_file(&repo, "ax-un10")).unwrap();
    let fm = un10.split("\n---").next().unwrap();
    assert!(fm.contains("status: doing"), "{un10}");
    assert_eq!(fm.matches("status:").count(), 1, "one status line: {un10}");
    assert!(un10.contains("lint --fix"), "repair logged: {un10}");

    // Exactly one ax-dup1 file remains; the other carries a fresh id.
    let dup_files: Vec<_> = std::fs::read_dir(repo.join("docs/meshwork"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("ax-dup1-"))
        .collect();
    assert_eq!(dup_files.len(), 1, "one keeper: {dup_files:?}");
}

/// MW-I2 (visibility half): unparseable tasks stay loud everywhere —
/// lint, q, and show — never silently dropped.
#[test]
fn invalid_visible() {
    let (_g, repo) = fixture_repo("alpha-broken");

    let lint_out = stdout_of(&meshwork(&repo).arg("lint").assert().code(1));
    assert!(lint_out.contains("ax-brk9"), "{lint_out}");
    assert!(lint_out.contains("ax-un10"), "{lint_out}");

    let q_out = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT id FROM tasks WHERE status='invalid' ORDER BY id"])
            .assert()
            .success(),
    );
    assert!(q_out.contains("ax-brk9") && q_out.contains("ax-un10"), "{q_out}");

    meshwork(&repo)
        .args(["show", "ax-brk9"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("INVALID"));
}
