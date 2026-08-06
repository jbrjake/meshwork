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
