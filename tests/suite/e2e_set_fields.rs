// MW-L1 (owner ruling 2026-09-20, R-B item B.1): the fields the cross-repo
// mechanism depends on get a CLI path — `--to`, `--answers`, `--relates` on
// add and set; `set` grows `--body`, `--parent`, `--from` and a `--docs
// <old> <new>` replacement. Each writes frontmatter in this store and
// transmits nothing; twelve of twelve such writes in the field study went
// through hand-edits, and two of those broke the file.

/// `--to`/`--answers`/`--relates` land at add and at set: scalars replace,
/// `relates` appends, targets are validated like every other edge, and
/// the old "frontmatter-only" refusal is gone from verbs that lack the
/// flag — the error now names the two verbs that carry it.
#[test]
fn add_set_frontmatter_flags() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let t1 = add_task(&repo, "Target one");
    let t2 = add_task(&repo, "Target two");

    // add: an ask with a soft link, and an answer to a foreign ask.
    let ask = add_id(
        &repo,
        &["add", "Ask leras", "--verify", "true", "--to", "leras", "--relates", &t1],
    );
    let text = std::fs::read_to_string(task_file(&repo, &ask)).unwrap();
    assert!(text.contains("\nto: leras\n"), "{text}");
    assert!(text.contains(&format!("\nrelates: [{t1}]\n")), "{text}");
    let answer = add_id(
        &repo,
        &["add", "Answer", "--verify", "true", "--answers", "leras#le-a1b2c3d"],
    );
    let text = std::fs::read_to_string(task_file(&repo, &answer)).unwrap();
    assert!(text.contains("\nanswers: leras#le-a1b2c3d\n"), "{text}");
    let shown = stdout_of(&meshwork(&repo).args(["show", &answer]).assert().success());
    assert!(shown.contains("answers: leras#le-a1b2c3d"), "{shown}");

    // A same-repo target that does not exist is refused, as for --needs.
    meshwork(&repo)
        .args(["add", "Dangling", "--verify", "true", "--relates", "wo-nope"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("relates target `wo-nope` does not exist"));

    // set: scalars replace, relates appends without losing what is there.
    let out = meshwork(&repo)
        .args(["set", &t2, "--to", "marasi", "--answers", "portfolio#po-x1y2z3w", "--relates", &t1])
        .assert()
        .success();
    let out = stdout_of(&out);
    for field in ["to", "answers", "relates"] {
        assert!(out.contains(&format!("{t2} {field} set")), "{out}");
    }
    let text = std::fs::read_to_string(task_file(&repo, &t2)).unwrap();
    assert!(text.contains("\nto: marasi\n"), "{text}");
    assert!(text.contains("\nanswers: portfolio#po-x1y2z3w\n"), "{text}");
    assert!(text.contains(&format!("\nrelates: [{t1}]\n")), "{text}");
    meshwork(&repo)
        .args(["set", &t2, "--to", "sazed", "--relates", &ask])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &t2)).unwrap();
    assert!(text.contains("\nto: sazed\n") && !text.contains("marasi"), "{text}");
    assert!(text.contains(&format!("\nrelates: [{t1}, {ask}]\n")), "{text}");
    // An edge already present is not appended twice.
    meshwork(&repo)
        .args(["set", &t2, "--relates", &t1])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &t2)).unwrap();
    assert!(text.contains(&format!("\nrelates: [{t1}, {ask}]\n")), "{text}");
    let v: serde_json::Value = serde_json::from_str(&stdout_of(
        &meshwork(&repo)
            .args(["set", &t2, "--answers", "sazed#sa-q9w8e7r", "--json"])
            .assert()
            .success(),
    ))
    .unwrap();
    assert_eq!(v["data"]["set"], serde_json::json!(["answers"]), "{v}");

    // The projection sees every write — nothing was sent anywhere.
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", &format!("SELECT addressed_to FROM tasks WHERE id = '{t2}'")])
            .assert()
            .success(),
    );
    assert!(q.contains("sazed"), "{q}");
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", &format!("SELECT count(*) FROM edges WHERE src_gid = 'work#{t2}' AND kind = 'relates'")])
            .assert()
            .success(),
    );
    assert!(q.contains('2'), "{q}");

    // A verb without the flag names the verbs that carry it.
    let err = stderr_of(&meshwork(&repo).args(["show", &t2, "--to", "x"]).assert().failure());
    assert!(
        err.contains("not a flag on this verb") && err.contains("set <id> --to") && !err.contains("frontmatter-only"),
        "{err}"
    );

    // With a registry, an addressee nobody registered is warned about at
    // the flag, as at the batch (the same door).
    let (dir, portfolio) = portfolio_fixture();
    let beta = dir.path().join("beta");
    let out = meshwork(&beta)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["add", "Ask nobody", "--verify", "true", "--to", "nowhere"])
        .assert()
        .success();
    assert!(stderr_of(&out).contains("no registered repo"), "{}", stderr_of(&out));
}

/// `set --docs <old> <new>` replaces one link in place (`--docs <link>`
/// still appends); `--body` replaces the description above the tail
/// sections; `--parent` and `--from` set or replace the edge, validated.
#[test]
fn set_docs_replace() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let parent = add_task(&repo, "Umbrella");
    let origin = add_task(&repo, "Origin");
    let id = add_id(
        &repo,
        &[
            "add", "Edited later", "--verify", "true",
            "--docs", "DESIGN.md#§-one", "--docs", "REQ.md#§-two",
            "--body", "First body.",
        ],
    );
    let path = task_file(&repo, &id);

    // Replace one link in place; the other stays where it was.
    let out = meshwork(&repo)
        .args(["set", &id, "--docs", "DESIGN.md#§-one", "PLAN.md#§-three"])
        .assert()
        .success();
    assert!(stdout_of(&out).contains(&format!("{id} docs set")));
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("docs:\n  - PLAN.md#§-three\n  - REQ.md#§-two\n") && !text.contains("DESIGN.md"),
        "{text}"
    );
    // A single link still appends.
    meshwork(&repo)
        .args(["set", &id, "--docs", "NOTES.md"])
        .assert()
        .success();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("  - REQ.md#§-two\n  - NOTES.md\n"), "{text}");
    // Replacing a link the task does not carry is refused, naming it.
    meshwork(&repo)
        .args(["set", &id, "--docs", "GONE.md", "X.md"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("GONE.md"));

    // --body replaces the description and leaves the tail sections alone.
    meshwork(&repo)
        .args(["comment", &id, "--as", "jon", "a note"])
        .assert()
        .success();
    meshwork(&repo)
        .args(["set", &id, "--body", "Second body.\n\nTwo paragraphs."])
        .assert()
        .success();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("---\n\nSecond body.\n\nTwo paragraphs.\n\n## log\n") && !text.contains("First body"),
        "{text}"
    );
    assert!(text.contains("## comments\n") && text.contains("a note"), "{text}");
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains("Two paragraphs.") && shown.contains("a note"), "{shown}");
    // @file lands prose without shell quoting; an empty payload clears it.
    let body_file = repo.join("body.md");
    std::fs::write(&body_file, "From a file `with` backticks.\n").unwrap();
    meshwork(&repo)
        .args(["set", &id, "--body", &format!("@{}", body_file.display())])
        .assert()
        .success();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("---\n\nFrom a file `with` backticks.\n\n## log\n"), "{text}");
    meshwork(&repo)
        .args(["set", &id, "--body", ""])
        .assert()
        .success();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("---\n\n## log\n"), "{text}");

    // --parent / --from set the edge, validated like add's.
    let out = meshwork(&repo)
        .args(["set", &id, "--parent", &parent, "--from", &origin])
        .assert()
        .success();
    let out = stdout_of(&out);
    assert!(out.contains(&format!("{id} parent set")) && out.contains(&format!("{id} discovered-from set")), "{out}");
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(&format!("\nparent: {parent}\n")), "{text}");
    assert!(text.contains(&format!("\ndiscovered-from: {origin}\n")), "{text}");
    meshwork(&repo)
        .args(["set", &id, "--parent", "wo-nope"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("parent target `wo-nope` does not exist"));
    meshwork(&repo)
        .args(["set", &id, "--parent", "other#wo-abc"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("in-repo"));
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", &format!("SELECT kind FROM edges WHERE src_gid = 'work#{id}' ORDER BY kind")])
            .assert()
            .success(),
    );
    assert!(q.contains("discovered-from") && q.contains("parent"), "{q}");

    // Every edit left a file the store still parses clean.
    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(!lint.contains("error["), "{lint}");
}
