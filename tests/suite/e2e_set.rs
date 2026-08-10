// `e2e::field_setters` — mw-0f4j (README spec): `--seq`/`--docs` on `add`
// plus the `set` verb. Hand-editing stays legal; it is just never the only
// path (supersedes DESIGN §7b's hand-edit-only ruling for these fields).

/// `add --seq/--docs` land in the new file; `set` edits existing files:
/// scalar seq, appended docs (comment suffixes preserved), replaced
/// handoff block, wrapped for readable files and » rendering.
#[test]
fn field_setters() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    // add: --seq and repeatable --docs, written as the block-list style
    // hand-edited stores already use.
    let id = add_id(
        &repo,
        &[
            "add",
            "Seeded fields",
            "--verify",
            "true",
            "--seq",
            "10",
            "--docs",
            "DESIGN.md#§-one",
            "--docs",
            "REQ.md#§-two",
        ],
    );
    let path = task_file(&repo, &id);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("seq: 10"), "{text}");
    assert!(
        text.contains("docs:\n  - DESIGN.md#§-one\n  - REQ.md#§-two"),
        "block list docs:\n{text}"
    );

    // set --seq replaces the scalar.
    let out = meshwork(&repo)
        .args(["set", &id, "--seq", "40"])
        .assert()
        .success();
    assert!(stdout_of(&out).contains(&format!("{id} seq set")));
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("seq: 40") && !text.contains("seq: 10"), "{text}");

    // Hand-edit a doc comment suffix, then set --docs must append without
    // rewriting (comments survive — hand-edits are legal, MW-A1).
    let text = std::fs::read_to_string(&path).unwrap();
    let edited = text.replace("  - DESIGN.md#§-one", "  - DESIGN.md#§-one # why");
    std::fs::write(&path, edited).unwrap();
    meshwork(&repo)
        .args(["set", &id, "--docs", "PLAN.md#§-three"])
        .assert()
        .success();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("  - DESIGN.md#§-one # why"),
        "hand comment survives: {text}"
    );
    assert!(
        text.contains("  - REQ.md#§-two\n  - PLAN.md#§-three"),
        "appended at block end: {text}"
    );

    // set --handoff writes a wrapped block scalar; a second set replaces it.
    let long = "Cliff is governor wakeup, not batch size - do not burn a session re-deriving that; try wakeup=250ms before touching batch math.";
    let out = meshwork(&repo)
        .args(["set", &id, "--handoff", long])
        .assert()
        .success();
    assert!(stdout_of(&out).contains(&format!("{id} handoff set")));
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("handoff: |\n"), "{text}");
    for line in text.lines().filter(|l| l.starts_with("  ") && l.contains("governor")) {
        assert!(line.len() <= 74, "wrapped ≤72 + indent: {line}");
    }
    meshwork(&repo)
        .args(["set", &id, "--handoff", "Short note."])
        .assert()
        .success();
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("handoff: |\n  Short note.\n"), "{text}");
    assert!(!text.contains("governor"), "old block fully replaced: {text}");

    // The » lines in prime come from exactly this block (DESIGN §7b).
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains("» Short note."), "{prime}");

    // Multiple fields in one call report each.
    let out = meshwork(&repo)
        .args(["set", &id, "--seq", "50", "--handoff", "Again."])
        .assert()
        .success();
    let s = stdout_of(&out);
    assert!(s.contains("seq set") && s.contains("handoff set"), "{s}");

    // JSON envelope (MW-C3).
    let js = stdout_of(
        &meshwork(&repo)
            .args(["set", &id, "--seq", "60", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["verb"], "set");
    assert_eq!(v["data"]["id"], serde_json::json!(id));
    assert_eq!(v["data"]["set"], serde_json::json!(["seq"]));

    // No field flags → error; unknown id → error naming it.
    meshwork(&repo).args(["set", &id]).assert().failure();
    meshwork(&repo)
        .args(["set", "wo-zzzzzzz", "--seq", "1"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("wo-zzzzzzz"));
}

/// mw-rz4ey2h (§6 ruling 2026-08-10): prose fields get a path that never
/// transits shell quoting — the pilot's inline --handoff had a backticked
/// chunk EXECUTED as command substitution and the stored body mangled.
/// `--handoff`/`comment` accept `@file` and `-` (stdin), verbatim.
#[test]
fn handoff_from_file() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Prose target");

    // Hostile payload: backticks, $(), quotes, emoji — everything the
    // shell mangles. Via @file it lands byte-faithful (modulo wrap).
    let hostile = "Refactor `pub fn spill()` first.\n\nThen $(watch) the \u{2705} gate — don't \"quote\" me.";
    std::fs::write(repo.join("notes.md"), hostile).unwrap();
    meshwork(&repo)
        .args(["set", &id, "--handoff", "@notes.md"])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("`pub fn spill()`"), "backticks survive: {text}");
    assert!(text.contains("$(watch)"), "substitution inert: {text}");
    assert!(text.contains('\u{2705}'), "emoji survive: {text}");

    // Stdin spelling replaces the block.
    meshwork(&repo)
        .args(["set", &id, "--handoff", "-"])
        .write_stdin("From stdin, take two.")
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("From stdin, take two."), "{text}");
    assert!(!text.contains("$(watch)"), "handoff replaced: {text}");

    // comment rides the same rule, both spellings.
    std::fs::write(repo.join("c.md"), "Comment with `ticks` and $(subst).").unwrap();
    meshwork(&repo)
        .args(["comment", &id, "--as", "tester", "@c.md"])
        .assert()
        .success();
    meshwork(&repo)
        .args(["comment", &id, "--as", "tester", "-"])
        .write_stdin("Stdin comment.")
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("Comment with `ticks` and $(subst)."), "{text}");
    assert!(text.contains("Stdin comment."), "{text}");

    // Inline text still works; a missing @file is a loud error, and the
    // error teaches the literal-@ escape (stdin).
    meshwork(&repo)
        .args(["set", &id, "--handoff", "plain inline"])
        .assert()
        .success();
    meshwork(&repo)
        .args(["set", &id, "--handoff", "@no-such-file.md"])
        .assert()
        .code(1)
        .stderr(predicates::str::contains("no-such-file.md"));
}

/// mw-f1x71yg (§6 ruling 2026-08-10, surface unfrozen): `set` grows
/// `--cat`/`--verify`/`--title`. Nine pilot sessions fell back to python
/// rewrites of task files for exactly these one-line field edits — one
/// close even ran a STALE verify because the CLI rejected the fresh one.
#[test]
fn set_cat_verify() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let out = meshwork(&repo).args(["add", "Rough capture"]).assert().success();
    let id = stdout_of(&out).lines().next().unwrap().to_string();
    let original_file = task_file(&repo, &id);

    // The pilot's rejected-wholesale case: mixed old + new flags land
    // together, atomically.
    meshwork(&repo)
        .args([
            "set", &id,
            "--seq", "12",
            "--cat", "engine/scale",
            "--verify", "cargo test governor",
            "--title", "Door fix: governor restart",
        ])
        .assert()
        .success();
    let text = std::fs::read_to_string(&original_file).unwrap();
    assert!(text.contains("seq: 12"), "{text}");
    assert!(text.contains("category: engine/scale"), "{text}");
    assert!(text.contains("verify: cargo test governor"), "{text}");
    assert!(text.contains("title: \"Door fix: governor restart\""), "{text}");
    assert_eq!(text.matches("verify:").count(), 1, "exactly one verify line: {text}");
    assert_eq!(text.matches("title:").count(), 1, "exactly one title line: {text}");
    // The slug is cosmetic and never load-bearing — retitling never renames.
    assert!(original_file.exists(), "file not renamed by --title");

    // The capture-to-startable flow this exists for (mw-6wdpz1b): the
    // verify set above unlocks start.
    meshwork(&repo).args(["start", &id]).assert().success();

    // Replacing an existing verify: surgical, still one line.
    meshwork(&repo)
        .args(["set", &id, "--verify", "cargo test governor -- --exact"])
        .assert()
        .success();
    let text = std::fs::read_to_string(&original_file).unwrap();
    assert!(text.contains("verify: cargo test governor -- --exact"), "{text}");
    assert_eq!(text.matches("verify:").count(), 1, "{text}");

    // JSON names the canonical file keys it set.
    let js = stdout_of(
        &meshwork(&repo)
            .args(["set", &id, "--cat", "engine/spill", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["set"], serde_json::json!(["category"]), "{js}");
}
