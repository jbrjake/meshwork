// mw-4n00yte: a readable lint channel — silence on immutable history, on
// a deliverable that is its own doc, and on edits this clone made itself.
// Part-file of `lint.rs` (include!), so the paths stay `lint::<name>`.

/// A store dir with a config, ready for task files.
fn channel_store() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("repo");
    let mw = root.join("docs/meshwork");
    std::fs::create_dir_all(mw.join("archive")).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    (dir, root)
}

/// description-size on a terminal task is noise: the file is history,
/// nobody will trim it, and it buries the live findings.
#[test]
fn archived_description_size_silent() {
    let (_dir, root) = channel_store();
    let mw = root.join("docs/meshwork");
    let big = "long design narrative that belongs behind docs: links. ".repeat(60);
    let task = |id: &str, status: &str| {
        format!(
            "---\nid: {id}\ntitle: Oversized\nstatus: {status}\nverify: \"true\"\n---\n{big}\n\n\
             ## log\n- 2026-08-01 created\n- 2026-08-02 open\u{2192}{status}\n"
        )
    };
    std::fs::write(
        mw.join("archive/zz-old1-oversized.md"),
        task("zz-old1", "done"),
    )
    .unwrap();
    std::fs::write(mw.join("zz-big1-oversized.md"), task("zz-big1", "open")).unwrap();
    let f = lint_store(&load_repo(&root).unwrap());
    assert!(
        !has(&f, Severity::Warning, "description-size", "zz-old1"),
        "history is silent: {f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "description-size", "zz-big1"),
        "live still warns: {f:?}"
    );
}

/// A `docs:` link naming the task's own `exists` deliverable is the work,
/// not a dead link — silent until the file appears; any other absent
/// link still warns.
#[test]
fn doc_missing_exempts_own_deliverable() {
    let (_dir, root) = channel_store();
    std::fs::write(
        root.join("docs/meshwork/zz-own1-report.md"),
        "---\nid: zz-own1\ntitle: Write the report\nstatus: open\n\
         verify: exists docs/report.md\n\
         docs:\n  - docs/report.md\n  - docs/report.md#§-2-findings\n  - docs/other.md\n---\n",
    )
    .unwrap();
    let f = lint_store(&load_repo(&root).unwrap());
    let missing: Vec<&str> = f
        .iter()
        .filter(|x| x.code == "doc-missing")
        .map(|x| x.message.as_str())
        .collect();
    assert!(
        !missing.iter().any(|m| m.contains("docs/report.md")),
        "the deliverable is exempt, anchor or not: {missing:?}"
    );
    assert!(
        missing.iter().any(|m| m.contains("docs/other.md")),
        "an unrelated absent link still warns: {missing:?}"
    );
}

/// A verify rewritten by hand on this clone — uncommitted in the tree —
/// is not a silent weakening arriving from elsewhere. The finding waits
/// until the differing text arrives by commit; the close gate is untouched.
#[test]
fn verify_changed_since_approval_quiet_when_dirty() {
    let (_dir, root) = channel_store();
    crate::common::git(&root, &["init", "-q"]);
    crate::common::git(&root, &["config", "user.name", "Fixture User"]);
    crate::common::git(&root, &["config", "user.email", "fixture@example.invalid"]);
    let file = root.join("docs/meshwork/zz-chg2-weakened.md");
    let task = |verify: &str| {
        format!("---\nid: zz-chg2\ntitle: Weakened\nstatus: open\nverify: {verify:?}\n---\n")
    };
    std::fs::write(&file, task("run cargo test real::gate")).unwrap();
    crate::common::git(&root, &["add", "-A"]);
    crate::common::git(&root, &["commit", "-q", "-m", "approved text"]);
    meshwork::trust::record_approval(&root, "zz-chg2", "run cargo test real::gate").unwrap();

    // The hand-edit sits uncommitted: this clone wrote it, no finding.
    std::fs::write(&file, task("true")).unwrap();
    let dirty = lint_store(&load_repo(&root).unwrap());
    assert!(
        !dirty
            .iter()
            .any(|x| x.code == "verify-changed-since-approval"),
        "an uncommitted edit is this clone's own: {dirty:?}"
    );

    // Committed, the differing text is an arrival to review.
    crate::common::git(&root, &["add", "-A"]);
    crate::common::git(&root, &["commit", "-q", "-m", "weakened"]);
    let committed = lint_store(&load_repo(&root).unwrap());
    assert!(
        has(
            &committed,
            Severity::Warning,
            "verify-changed-since-approval",
            "zz-chg2"
        ),
        "{committed:?}"
    );
}

/// A task file under the store dir: frontmatter lines, then a body.
fn write_task(root: &std::path::Path, id: &str, front: &str, body: &str) {
    std::fs::write(
        root.join(format!("docs/meshwork/{id}-t.md")),
        format!("---\nid: {id}\ntitle: {id}\n{front}---\n{body}"),
    )
    .unwrap();
}

/// mw-axhze9m: a live task whose handoff names a closed task is prose
/// about to be trusted past its date — the mentions view says so; a
/// handoff naming live work is fine.
#[test]
fn handoff_cites_closed() {
    let (_dir, root) = channel_store();
    write_task(
        &root,
        "zz-done1",
        "status: done\nverify: \"true\"\n",
        "\n## log\n- 2026-08-01 created\n- 2026-08-02 open\u{2192}done\n",
    );
    write_task(
        &root,
        "zz-live1",
        "status: open\nverify: \"true\"\nhandoff: |\n  Start from zz-done1, it landed the seam.\n",
        "",
    );
    write_task(
        &root,
        "zz-live2",
        "status: open\nverify: \"true\"\nhandoff: |\n  Pair with zz-live1.\n",
        "",
    );
    let f = lint_store(&load_repo(&root).unwrap());
    assert!(
        has(&f, Severity::Warning, "handoff-cites-closed", "zz-live1"),
        "{f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "handoff-cites-closed", "zz-done1"),
        "names the closed task: {f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "handoff-cites-closed", "zz-live2"),
        "a live reference is not stale: {f:?}"
    );
}

/// Two live tasks on one seq in one repo is an order nobody chose;
/// terminal tasks and distinct seqs are not collisions.
#[test]
fn seq_collision() {
    let (_dir, root) = channel_store();
    for (id, status, seq) in [
        ("zz-seqa", "open", 10),
        ("zz-seqb", "doing", 10),
        ("zz-seqc", "done", 10),
        ("zz-seqd", "open", 20),
    ] {
        write_task(
            &root,
            id,
            &format!("status: {status}\nverify: \"true\"\nseq: {seq}\n"),
            "",
        );
    }
    let f = lint_store(&load_repo(&root).unwrap());
    assert!(
        has(&f, Severity::Warning, "seq-collision", "zz-seqa")
            && has(&f, Severity::Warning, "seq-collision", "zz-seqb"),
        "both live holders named: {f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "seq-collision", "zz-seqc")
            && !has(&f, Severity::Warning, "seq-collision", "zz-seqd"),
        "terminal and distinct seqs are silent: {f:?}"
    );
}

/// The remaining three: a live→live mention with no edge behind it, a
/// discovered-from cycle, and repeated failed closes on a live task.
#[test]
fn views_findings() {
    let (_dir, root) = channel_store();
    write_task(
        &root,
        "zz-impa",
        "status: open\nverify: \"true\"\n",
        "Depends on what zz-impb decides.\n",
    );
    write_task(&root, "zz-impb", "status: open\nverify: \"true\"\n", "");
    write_task(
        &root,
        "zz-impc",
        "status: open\nverify: \"true\"\nrelates: [zz-impb]\n",
        "Sibling of zz-impb.\n",
    );
    write_task(
        &root,
        "zz-cyca",
        "status: open\nverify: \"true\"\ndiscovered-from: zz-cycb\n",
        "",
    );
    write_task(
        &root,
        "zz-cycb",
        "status: open\nverify: \"true\"\ndiscovered-from: zz-cyca\n",
        "",
    );
    write_task(
        &root,
        "zz-cls1",
        "status: open\nverify: \"true\"\n",
        "\n## log\n- 2026-08-01 created\n- 2026-08-02 close attempt \u{2014} verify failed\n\
         - 2026-08-03 close attempt \u{2014} verify failed\n",
    );
    let f = lint_store(&load_repo(&root).unwrap());
    assert!(
        has(&f, Severity::Warning, "implicit-edge", "zz-impa")
            && has(&f, Severity::Warning, "implicit-edge", "zz-impb"),
        "the edgeless mention, both ends named: {f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "implicit-edge", "zz-impc"),
        "an edge behind the mention is not implicit: {f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "discovered-cycle", "zz-cyca"),
        "{f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "close-attempts", "zz-cls1"),
        "{f:?}"
    );
}

/// mw-xb9prd6: a `contains` (or legacy grep) whose target is the task's
/// own file is satisfiable by whoever writes the file — unless the
/// pattern is the date-first owner marker only an appended line can
/// match. A lexical heuristic, and it says so.
#[test]
fn verify_self_satisfying() {
    let (_dir, root) = channel_store();
    write_task(
        &root,
        "zz-self1",
        "status: open\nverify: contains docs/meshwork/zz-self1-t.md shipped\n",
        "",
    );
    write_task(
        &root,
        "zz-self2",
        "status: open\nverify: grep -q shipped docs/meshwork/zz-self2-t.md\n",
        "",
    );
    write_task(
        &root,
        "zz-self3",
        "status: open\nverify: contains docs/meshwork/zz-self3-t.md /^- 2026-/\n",
        "",
    );
    write_task(
        &root,
        "zz-self4",
        "status: open\nverify: contains docs/other.md shipped\n",
        "",
    );
    let f = lint_store(&load_repo(&root).unwrap());
    assert!(
        has(&f, Severity::Warning, "verify-self-satisfying", "zz-self1"),
        "{f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "verify-self-satisfying", "zz-self2"),
        "legacy grep on the own file too: {f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "verify-self-satisfying", "zz-self3"),
        "the date-first owner marker is the sanctioned idiom: {f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "verify-self-satisfying", "zz-self4"),
        "another file is not self-satisfying: {f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "verify-self-satisfying", "heuristic"),
        "the finding names itself a heuristic: {f:?}"
    );
}

/// mw-xb9prd6: prose that invokes an owner ruling without quoting the
/// owner is the authority-laundering shape; a quoted span settles it.
#[test]
fn ruling_without_quote() {
    let (_dir, root) = channel_store();
    write_task(
        &root,
        "zz-rul1",
        "status: open\nverify: \"true\"\n",
        "Owner ruling 2026-09-01: ship the fold as is.\n",
    );
    write_task(
        &root,
        "zz-rul2",
        "status: open\nverify: \"true\"\n",
        "Owner ruling 2026-09-01: *\"ship the fold as is\"* — so the fold ships.\n",
    );
    write_task(
        &root,
        "zz-rul3",
        "status: open\nverify: \"true\"\n",
        "\n## comments\n- 2026-09-02 [agent] OWNER CALL: parked until the scale ruling.\n",
    );
    write_task(
        &root,
        "zz-rul4",
        "status: done\nverify: \"true\"\n",
        "owner ruled this closed.\n\n## log\n- 2026-09-01 open\u{2192}done\n",
    );
    let f = lint_store(&load_repo(&root).unwrap());
    assert!(
        has(&f, Severity::Warning, "ruling-without-quote", "zz-rul1"),
        "{f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "ruling-without-quote", "zz-rul2"),
        "a quoted span is the owner's words: {f:?}"
    );
    assert!(
        has(&f, Severity::Warning, "ruling-without-quote", "zz-rul3"),
        "comments count: {f:?}"
    );
    assert!(
        !has(&f, Severity::Warning, "ruling-without-quote", "zz-rul4"),
        "terminal tasks are history: {f:?}"
    );
}
