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

/// A store copied from the beta fixture with one spec doc and one pin
/// on it: the task's status, the pinned hash, and the doc's body.
fn pinned_store(
    dir: &std::path::Path,
    status: &str,
    sha: &str,
    body: &str,
) -> std::path::PathBuf {
    let repo = dir.join("drift");
    crate::common::copy_dir(&fixtures_root().join("beta"), &repo);
    std::fs::create_dir_all(repo.join("docs")).unwrap();
    std::fs::write(repo.join("docs/SPEC.md"), format!("## Gate {{#sp-gate}}\n\n{body}\n")).unwrap();
    std::fs::write(
        repo.join("docs/meshwork/bz-dr1ft-t.md"),
        format!(
            "---\nid: bz-dr1ft\ntitle: T\nstatus: {status}\ncovers:\n  - ref: docs/SPEC.md#sp-gate\n    sha: {sha}\n---\n"
        ),
    )
    .unwrap();
    repo
}

/// MW-T4: a live task whose pinned clause hashes differently now draws
/// `spec-drift`, naming both hashes and `cover --repin`; a retitled
/// heading is not drift; a matching pin and a done task are silent.
#[test]
fn spec_drift() {
    let dir = tempfile::tempdir().unwrap();
    let pinned = meshwork::spec::sha_hex("A task closes on exit 0.");
    let repo = pinned_store(dir.path(), "open", &pinned, "A task closes on exit 0, never a waive.");
    let f = lint_store(&load_repo(&repo).unwrap());
    let now = meshwork::spec::sha_hex("A task closes on exit 0, never a waive.");
    assert!(
        has(
            &f,
            Severity::Warning,
            "spec-drift",
            &format!(
                "bz-dr1ft clause `docs/SPEC.md#sp-gate` reads differently from its pin (pinned {}, now {}) — re-read it, then `meshwork cover bz-dr1ft docs/SPEC.md#sp-gate --repin`",
                &pinned[..12],
                &now[..12]
            )
        ),
        "{f:?}"
    );
    assert!(!has(&f, Severity::Error, "covers-malformed", "bz-dr1ft"), "a real pin: {f:?}");

    // Same words, retitled heading: no drift. Same words, done: silent.
    std::fs::write(repo.join("docs/SPEC.md"), "## The gate, restated {#sp-gate}\n\nA task closes on exit 0.\n").unwrap();
    let f = lint_store(&load_repo(&repo).unwrap());
    assert!(!f.iter().any(|x| x.code == "spec-drift"), "{f:?}");
    let done = pinned_store(&dir.path().join("d"), "done", &pinned, "moved words");
    let f = lint_store(&load_repo(&done).unwrap());
    assert!(!f.iter().any(|x| x.code == "spec-drift"), "a done task's pin is history: {f:?}");
}

/// MW-T6: the hash is over clause text as read from disk, so a spec
/// that no version control tracks still drifts visibly — the store here
/// is a bare directory, no `.git` anywhere above it inside the tempdir.
#[test]
fn spec_drift_unversioned_corpus() {
    let dir = tempfile::tempdir().unwrap();
    let pinned = meshwork::spec::sha_hex("first reading");
    let repo = pinned_store(dir.path(), "doing", &pinned, "second reading");
    assert!(!repo.join(".git").exists() && !dir.path().join(".git").exists());
    let f = lint_store(&load_repo(&repo).unwrap());
    assert!(has(&f, Severity::Warning, "spec-drift", "bz-dr1ft"), "{f:?}");
}
