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

}

/// A same-repo edge target that does not exist is refused at add — a typo
/// in `--from` once minted two dangling edges silently. A dead docs
/// anchor and an inline body carrying shell syntax are warned about at
/// add, where the author is listening. add --batch validates the same
/// way, atomically, with batch handles counting as known.
#[test]
fn add_refuses_dangling_edge() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::write(repo.join("NOTES.md"), "# Notes\n\nbody\n").unwrap();
    let real = add_task(&repo, "Real target");

    for flag in ["--from", "--needs", "--parent"] {
        let err = stderr_of(
            &meshwork(&repo)
                .args(["add", "Dangling", "--verify", "true", flag, "wo-nope123"])
                .assert()
                .failure(),
        );
        assert!(
            err.contains("wo-nope123") && err.contains("does not exist"),
            "{flag}: {err}"
        );
    }
    let listing = stdout_of(&meshwork(&repo).args(["ready", "--all"]).assert().success());
    assert!(!listing.contains("Dangling"), "nothing minted: {listing}");

    // A real target is fine; a cross-repo one is the registry's business
    // and stays quiet when no registry is loaded.
    let out = meshwork(&repo)
        .args(["add", "Wired", "--verify", "true", "--from", &real, "--needs", "nowhere#zz-1"])
        .assert()
        .success();
    assert!(!stderr_of(&out).contains("warning"), "{}", stderr_of(&out));

    // A dead anchor is warned about now, not at the next lint.
    let out = meshwork(&repo)
        .args(["add", "Anchored", "--verify", "true", "--docs", "NOTES.md#§-nope"])
        .assert()
        .success();
    assert!(stderr_of(&out).contains("anchor not found"), "{}", stderr_of(&out));

    // Inline prose carrying a backtick or $( is warned about, pointing at
    // the paths that never transit shell quoting.
    let out = meshwork(&repo)
        .args(["add", "Prose", "--verify", "true", "--body", "run `cargo test` then $(date)"])
        .assert()
        .success();
    let err = stderr_of(&out);
    assert!(err.contains("backtick") && err.contains("@<file>"), "{err}");

    // Batch: handles resolve, a dangling id refuses the whole batch.
    let err = stderr_of(
        &meshwork(&repo)
            .args(["add", "--batch", "-"])
            .write_stdin(
                "---\nhandle: a\ntitle: Batch alpha\nverify: \"true\"\n---\n\
                 ---\ntitle: Batch beta\nneeds: [\"@a\"]\nverify: \"true\"\n---\n\
                 ---\ntitle: Batch gamma\nneeds: [wo-nope123]\nverify: \"true\"\n---\n",
            )
            .assert()
            .failure(),
    );
    assert!(
        err.contains("batch task 3") && err.contains("wo-nope123") && err.contains("nothing written"),
        "{err}"
    );
    let listing = stdout_of(&meshwork(&repo).args(["ready", "--all"]).assert().success());
    assert!(!listing.contains("Batch alpha"), "atomic: {listing}");
}

/// The five messages that cost the most (field study §1.3, §2.4): the
/// inbox verbs agents guess point at prime/ready/--help, never at a
/// writing verb; frontmatter-only flags say so instead of clap's `--`
/// tip; a local `addressed_to` query that finds nothing says the inbox
/// is elsewhere; `dep add A B` is redirected to `--needs` and the
/// success line models the flag.
#[test]
fn did_you_mean_inbox_verbs() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    for (verb, near) in [
        ("next", "prime"),
        ("addressed", "prime"),
        ("inbox", "prime"),
        ("help", "--help"),
        ("list", "ready"),
    ] {
        let err = stderr_of(&meshwork(&repo).arg(verb).assert().code(2));
        assert!(err.contains(near), "{verb}: {err}");
        for wrong in ["`set`", "`add`", "`dep`"] {
            assert!(!err.contains(wrong), "{verb} must not point at a writer: {err}");
        }
        assert!(err.lines().count() <= 3, "survives tail -3: {err}");
    }
    let err = stderr_of(
        &meshwork(&repo)
            .args(["portfolio", "show", "zz-1"])
            .assert()
            .code(2),
    );
    assert!(err.contains("single-repo") && err.contains("portfolio q"), "{err}");

    let id = add_task(&repo, "Flagged");
    for flag in ["--to", "--answers", "--relates"] {
        let err = stderr_of(&meshwork(&repo).args(["set", &id, flag, "x"]).assert().code(2));
        assert!(
            err.contains("frontmatter-only") && err.contains("add --batch"),
            "{flag}: {err}"
        );
        assert!(!err.contains("-- --"), "clap's tip is gone: {err}");
    }
    let err = stderr_of(&meshwork(&repo).args(["set", &id, "--body", "x"]).assert().code(2));
    assert!(err.contains("--body") && err.contains("add"), "{err}");

    let out = meshwork(&repo)
        .args(["q", "SELECT id FROM tasks WHERE addressed_to = 'work'"])
        .assert()
        .success();
    let note = stderr_of(&out);
    assert!(note.contains("outbound") && note.contains("prime"), "{note}");

    let target = add_task(&repo, "Target");
    let err = stderr_of(
        &meshwork(&repo)
            .args(["dep", "add", &id, &target])
            .assert()
            .failure(),
    );
    assert!(err.contains(&format!("dep add {id} --needs {target}")), "{err}");
    let out = stdout_of(
        &meshwork(&repo)
            .args(["dep", "add", &id, "--needs", &target])
            .assert()
            .success(),
    );
    assert!(out.contains("--needs"), "success line models the flag: {out}");
}

/// An id with a sibling's prefix is not "not found" — it is elsewhere,
/// and show says where and gives the one-liner. Cross-repo edge targets
/// the registry cannot resolve are warned about at add.
#[test]
fn show_foreign_id_hint() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    for id in ["az-x9b2", "alpha#az-x9b2"] {
        let err = stderr_of(
            &meshwork(&beta)
                .env("MESHWORK_PORTFOLIO", &portfolio)
                .args(["show", id])
                .assert()
                .failure(),
        );
        assert!(err.contains("belongs to alpha"), "{id}: {err}");
        assert!(err.contains("meshwork show az-x9b2"), "{id}: {err}");
        assert!(err.contains(&alpha.display().to_string()), "{id}: {err}");
    }
    // No registry: the plain refusal, nothing invented.
    let err = stderr_of(&meshwork(&beta).args(["show", "az-x9b2"]).assert().failure());
    assert!(err.contains("not found") && !err.contains("belongs to"), "{err}");

    let out = meshwork(&beta)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["add", "Edges out", "--verify", "true", "--needs", "nowhere#zz-1", "--needs", "alpha#az-nope"])
        .assert()
        .success();
    let err = stderr_of(&out);
    assert!(err.contains("nowhere#zz-1") && err.contains("no registered repo"), "{err}");
    assert!(err.contains("alpha#az-nope") && err.contains("does not exist"), "{err}");
}

/// `--category`/`--doc` are hidden aliases of `--cat`/`--docs` on add and
/// set (§6 ruling, mw-42ygb52).
#[test]
fn category_doc_aliases() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

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
        .args(["set", id, "--category", "core/y", "--doc", "FORMAT.md#configtoml"])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, id)).unwrap();
    assert!(text.contains("category: core/y"), "{text}");
    assert!(text.contains("FORMAT.md#configtoml"), "{text}");
}
