// mw-3jwwh5d: prime stamps store provenance — HEAD short-sha + uncommitted
// task-edit count (+ ahead-of-upstream when one exists), scoped to
// docs/meshwork/. An incoming session sees staleness up front instead of
// discovering it mid-work. Degrades silently when git info is unavailable.

#[test]
fn prime_provenance_line_present() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "committed work");
    git(&repo, &["add", "."]);
    git(&repo, &["commit", "-qm", "chore(store): seed"]);

    // A clean store: sha line, no uncommitted segment.
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains("store @ "), "{prime}");
    assert!(!prime.contains("uncommitted"), "{prime}");

    // Dirty one task file — the count appears.
    let path = task_file(&repo, &id);
    let text = std::fs::read_to_string(&path).unwrap() + "\ndirty edit\n";
    std::fs::write(&path, text).unwrap();
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains("store @ "), "{prime}");
    assert!(prime.contains("1 uncommitted task edit"), "{prime}");
}

#[test]
fn prime_provenance_degrades_silently() {
    // No commits yet: HEAD is unborn, git info unavailable — the line is
    // omitted and prime still works (MW-D5: never fail the digest).
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    add_task(&repo, "uncommitted era");
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(!prime.contains("store @"), "{prime}");
    assert!(prime.contains("open"), "digest still renders: {prime}");
}
