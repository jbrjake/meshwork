// e2e part-file: CLI forgiveness (mw-5hrb22q). Included by e2e.rs.

/// mw-5hrb22q: the pilot lost a whole session's progress note to `log <id>
/// "…"` answering with bare usage — no reason, no near verb — and paid
/// 3-attempt round-trips on `--category`/`--doc`. Curated unknown verbs
/// fail with a two-line did-you-mean (short enough to survive `| tail -3`
/// or `| head -3` truncation from either end); the flag misses are real
/// aliases now (§6 ruling via the set-fields unfreeze).
#[test]
fn cli_forgiveness() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    // `log` — the natural guess; task files HAVE a log: section.
    let assert = meshwork(&repo)
        .args(["log", "zz-none", "harness landed"])
        .assert()
        .code(2);
    let err = stderr_of(&assert);
    assert!(err.contains("did you mean `comment`"), "{err}");
    assert!(
        err.contains("meshwork comment"),
        "reason + usage, not a bare pointer: {err}"
    );
    assert!(
        err.lines().count() <= 3,
        "must survive tail/head truncation: {err}"
    );
    assert!(
        err.lines().next().unwrap_or_default().contains("comment"),
        "teaches on line 1: {err}"
    );

    // Verbs clap can't guess textually still map: done→close, rm→drop.
    let err = stderr_of(&meshwork(&repo).args(["done", "zz-none"]).assert().code(2));
    assert!(err.contains("`close`"), "{err}");
    let err = stderr_of(&meshwork(&repo).args(["rm", "zz-none"]).assert().code(2));
    assert!(err.contains("`drop`"), "{err}");

    // A plain typo keeps clap's own similarity tip (no table entry needed).
    let err = stderr_of(&meshwork(&repo).args(["redy"]).assert().code(2));
    assert!(err.contains("ready"), "{err}");

    // --category / --doc are aliases of --cat / --docs.
    let out = stdout_of(
        &meshwork(&repo)
            .args([
                "add",
                "Aliased",
                "--category",
                "core/x",
                "--doc",
                "FORMAT.md#task-file",
                "--verify",
                "true",
            ])
            .assert()
            .success(),
    );
    let id = out.lines().next().unwrap();
    let text = std::fs::read_to_string(task_file(&repo, id)).unwrap();
    assert!(text.contains("category: core/x"), "{text}");
    assert!(text.contains("FORMAT.md#task-file"), "{text}");

    meshwork(&repo)
        .args(["set", id, "--doc", "FORMAT.md#configtoml"])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, id)).unwrap();
    assert!(
        text.contains("FORMAT.md#configtoml"),
        "set --doc appends: {text}"
    );
}
