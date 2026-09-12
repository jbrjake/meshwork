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
