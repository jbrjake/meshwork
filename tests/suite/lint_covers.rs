// lint part-file: pin hygiene (MW-T3, mw-psbn61z). Included by lint.rs —
// tests here are `lint::<name>`.

/// MW-T3: a `covers:` entry the tool did not write — a bare ref, a
/// missing or malformed sha, a fragment that is not a clause id — is an
/// error on a live task, naming `cover --repin`; a real pin is silent,
/// and a terminal task's entries are history.
#[test]
fn covers_hand_written() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("pins");
    crate::common::copy_dir(&fixtures_root().join("beta"), &repo);
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    std::fs::write(repo.join("docs/SPEC.md"), "## Gate {#sp-gate}\n\nbody\n").unwrap();
    let sha = meshwork::spec::sha_hex("body");
    let task = |id: &str, status: &str, covers: &str| {
        std::fs::write(
            repo.join(format!("docs/meshwork/{id}-t.md")),
            format!("---\nid: {id}\ntitle: T\nstatus: {status}\ncovers:\n{covers}---\n"),
        )
        .unwrap();
    };
    task("bz-h4nd1", "open", "  - docs/SPEC.md#sp-gate\n");
    task("bz-h4nd2", "open", "  - ref: docs/SPEC.md#sp-gate\n    sha: deadbeef\n");
    task("bz-h4nd3", "open", "  - ref: docs/SPEC.md#§-gate\n    sha: 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n");
    task("bz-h4nd4", "open", &format!("  - ref: docs/SPEC.md#sp-gate\n    sha: {sha}\n"));
    task("bz-h4nd5", "done", "  - docs/SPEC.md#sp-gate\n");

    let f = lint_store(&load_repo(&repo).unwrap());
    assert!(has(&f, Severity::Error, "covers-malformed", "bz-h4nd1 covers entry `docs/SPEC.md#sp-gate` was not written by cover (no sha)"), "{f:?}");
    assert!(has(&f, Severity::Error, "covers-malformed", "bz-h4nd2"), "{f:?}");
    assert!(has(&f, Severity::Error, "covers-malformed", "bz-h4nd3"), "{f:?}");
    assert!(has(&f, Severity::Error, "covers-malformed", "cover bz-h4nd1 docs/SPEC.md#sp-gate --repin"), "{f:?}");
    let malformed: Vec<&str> = f
        .iter()
        .filter(|x| x.code == "covers-malformed")
        .map(|x| x.subject.as_str())
        .collect();
    assert_eq!(malformed, ["bz-h4nd1", "bz-h4nd2", "bz-h4nd3"], "a real pin and a done task are silent");
}
