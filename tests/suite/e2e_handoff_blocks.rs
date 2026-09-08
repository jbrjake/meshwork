// Block scalars with unindented blank lines — legal YAML, legal per the
// skill, and the shape hand-authored `handoff: |` blocks take. The edit
// primitives must treat such a block as one value, and lint --fix must
// mend the file a narrower reading left behind.

/// A `handoff: |` block whose paragraphs are separated by an unindented
/// blank line is one value. close must remove all of it — not the key
/// and first paragraph only, leaving the rest stranded inside the
/// frontmatter as invalid YAML.
#[test]
fn close_strips_handoff_block_with_blank_line() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Hand-authored handoff");
    let path = task_file(&repo, &id);
    std::fs::write(
        &path,
        format!(
            "---\nid: {id}\ntitle: Hand-authored handoff\nstatus: open\nhandoff: |\n  \
             para one here\n\n  para two after a blank\nverify: \"true\"\n\
             created: 2026-09-01\n---\n\n## log\n- 2026-09-01 created\n"
        ),
    )
    .unwrap();

    // Precondition: the hand-authored block parses and renders whole.
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains("para two after a blank"), "{shown}");

    meshwork(&repo).args(["close", &id]).assert().success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(text.contains("status: done"), "{text}");
    assert!(!text.contains("handoff:"), "key stripped: {text}");
    assert!(!text.contains("para one here"), "first paragraph stripped: {text}");
    assert!(
        !text.contains("para two after a blank"),
        "the paragraph after the blank line goes with the block: {text}"
    );
    // The file still parses — lint sees no error.
    let lint = meshwork(&repo).arg("lint").assert().success();
    assert!(stdout_of(&lint).contains("0 error(s)"), "{}", stdout_of(&lint));
    // set --handoff / drop on a second task with the same shape: same rule.
    let other = add_task(&repo, "Dropped hand-authored handoff");
    std::fs::write(
        task_file(&repo, &other),
        format!(
            "---\nid: {other}\ntitle: Dropped hand-authored handoff\nstatus: open\n\
             handoff: |\n  first\n\n  second\nverify: \"true\"\ncreated: 2026-09-01\n---\n\n\
             ## log\n- 2026-09-01 created\n"
        ),
    )
    .unwrap();
    meshwork(&repo).args(["drop", &other]).assert().success();
    let text = std::fs::read_to_string(task_file(&repo, &other)).unwrap();
    assert!(!text.contains("second"), "drop strips the whole block: {text}");
    meshwork(&repo).arg("lint").assert().success();
}

/// A block-item list (`docs:`) with a blank line inside stays one list
/// under set --docs, and set --handoff replaces a blank-bearing block
/// whole instead of leaving its tail behind the new one.
#[test]
fn set_replaces_blank_bearing_blocks_whole() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::write(repo.join("NOTES.md"), "# Notes\n\n## Second\n").unwrap();
    let id = add_task(&repo, "Blocks with blanks");
    std::fs::write(
        task_file(&repo, &id),
        format!(
            "---\nid: {id}\ntitle: Blocks with blanks\nstatus: open\nhandoff: |\n  old one\n\n  \
             old two\ndocs:\n  - NOTES.md\n\n  - NOTES.md#second\nverify: \"true\"\n\
             created: 2026-09-01\n---\n\n## log\n- 2026-09-01 created\n"
        ),
    )
    .unwrap();
    meshwork(&repo)
        .args(["set", &id, "--handoff", "fresh voice", "--docs", "NOTES.md#notes"])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(!text.contains("old two"), "old block replaced whole: {text}");
    assert!(text.contains("  - NOTES.md#second\n  - NOTES.md#notes"), "{text}");
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains("fresh voice"), "{shown}");
    meshwork(&repo).arg("lint").assert().success();
}

/// The damage an earlier strip left behind — indented lines stranded
/// under no key after a blank line — is mechanical, so lint --fix mends
/// it and logs the repair. A file it cannot mend gets a plain
/// `cannot repair` line naming the reason, never silence.
#[test]
fn lint_fix_repairs_stranded_block() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Stranded after strip");
    std::fs::write(
        task_file(&repo, &id),
        format!(
            "---\nid: {id}\ntitle: Stranded after strip\nstatus: done\n\n  para two after a blank\n\
             verify: \"true\"\ncreated: 2026-09-01\n---\n\n## log\n- 2026-09-01 created\n\
             - 2026-09-02 open→done — verify exit 0\n"
        ),
    )
    .unwrap();
    // Broken: lint reports the parse error and exits non-zero.
    meshwork(&repo).arg("lint").assert().failure();

    let fixed = stdout_of(&meshwork(&repo).args(["lint", "--fix"]).assert().success());
    assert!(fixed.contains("fixed 1 file(s)"), "{fixed}");
    let text = std::fs::read_to_string(task_file(&repo, &id)).unwrap();
    assert!(!text.contains("para two after a blank"), "{text}");
    assert!(text.contains("lint --fix: dropped 1 stranded line"), "{text}");
    let shown = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(shown.contains("[done]"), "{shown}");

    // Not mechanical: an unterminated flow list. --fix says so, by name.
    let bad = add_task(&repo, "Beyond repair");
    std::fs::write(
        task_file(&repo, &bad),
        format!(
            "---\nid: {bad}\ntitle: Beyond repair\nstatus: open\nneeds: [\nverify: \"true\"\n\
             created: 2026-09-01\n---\n\n## log\n- 2026-09-01 created\n"
        ),
    )
    .unwrap();
    let out = meshwork(&repo).args(["lint", "--fix"]).assert().failure();
    let text = stdout_of(&out);
    assert!(text.contains(&format!("cannot repair {bad}:")), "{text}");
}
