// mw-3tzfqmq (mint-side sister of render sanitization): YAML forbids raw
// control characters, so text bound for the frontmatter refuses them at
// the verb — the alternative was a verb that reports success while the
// store gains an unparseable row. Batch input needs no guard: its strict
// document parse already refuses. Files keep bytes as written elsewhere;
// this guards only what the CLI itself is about to write.

/// `add` with a raw control in any frontmatter-bound field refuses
/// loudly, names the character, and writes nothing.
#[test]
fn mint_rejects_controls_on_add() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let count = || {
        std::fs::read_dir(repo.join("docs/meshwork"))
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|x| x == "md")
            })
            .count()
    };
    let before = count();
    let assert = meshwork(&repo)
        .args(["add", "Bad \u{1b}[31mtitle"])
        .assert()
        .failure();
    let err = stderr_of(&assert);
    assert!(err.contains("control character"), "{err}");
    assert_eq!(count(), before, "nothing minted");

    // Every frontmatter-bound field takes the same door.
    for args in [
        vec!["add", "Fine", "--cat", "a\u{7f}b"],
        vec!["add", "Fine", "--verify", "true\u{1b}"],
        vec!["add", "Fine", "--label", "p\u{8}0"],
        vec!["add", "Fine", "--docs", "README.md#\u{1b}"],
        vec!["add", "Fine", "--body", "esc \u{1b} in body"],
    ] {
        meshwork(&repo).args(&args).assert().failure();
    }
    assert_eq!(count(), before, "nothing minted by any field");
}

/// set/block/drop refuse the same way and leave the file untouched;
/// genuinely multi-line fields keep newlines (and tabs) legal.
#[test]
fn mint_rejects_controls_on_set_and_transitions() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Target");
    let path = task_file(&repo, &id);
    let before = std::fs::read_to_string(&path).unwrap();

    for args in [
        vec!["set", &id, "--title", "x\u{1b}]0;spoof\u{7}"],
        vec!["set", &id, "--handoff", "line\u{1b}"],
        vec!["set", &id, "--docs", "a#\u{9c}"],
        vec!["block", &id, "--reason", "esc\u{1b}"],
        vec!["drop", &id, "--reason", "esc\u{9c}"],
    ] {
        let assert = meshwork(&repo).args(&args).assert().failure();
        let err = stderr_of(&assert);
        assert!(err.contains("control character"), "{args:?}: {err}");
    }
    assert_eq!(
        before,
        std::fs::read_to_string(&path).unwrap(),
        "file untouched by refused writes"
    );

    // Multi-line prose stays multi-line — only real controls refuse.
    meshwork(&repo)
        .args(["set", &id, "--handoff", "two\nlines"])
        .assert()
        .success();
    meshwork(&repo)
        .args(["add", "Multiline body", "--verify", "true", "--body", "a\nb\tc"])
        .assert()
        .success();
}
