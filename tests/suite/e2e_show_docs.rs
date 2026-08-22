// mw-hqs4 / PLAN 4.1 (MW-F2): `show --docs` resolves `docs:` links into
// anchor-scoped excerpts — drill-through is itself progressive
// disclosure: the section, never the whole file, ~4KB per link (bytes,
// MW-D5). Unresolvable links are loud lines, not errors — the task view
// must never die on a stale doc pointer.

/// `show --docs` emits the anchored section, capped, never the whole file.
#[test]
fn show_docs_excerpts() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    // Three sections; the third exceeds the per-link cap.
    let big = "filler line for the cap test\n".repeat(200); // ~5.8KB
    std::fs::write(
        repo.join("DESIGN-x.md"),
        format!(
            "# DESIGN-x\n\n## 1. On-disk layout (per repo)\n\nlayout body line.\n\n\
             ### 1a. Nested sub\n\nnested body stays in scope.\n\n\
             ```\n## not a heading — fenced\n```\n\n\
             ## 2. Query contract\n\nquery body line.\n\n## 3. Big\n\n{big}"
        ),
    )
    .unwrap();

    let id = add_id(
        &repo,
        &[
            "add",
            "Drill task",
            "--verify",
            "true",
            "--docs",
            "DESIGN-x.md#§-1-on-disk-layout",
            "--docs",
            "DESIGN-x.md#§-3-big",
            "--docs",
            "DESIGN-x.md#§-9-missing",
            "--docs",
            "MISSING.md#§-1-nope",
        ],
    );

    let out = stdout_of(
        &meshwork(&repo)
            .args(["show", &id, "--docs"])
            .assert()
            .success(),
    );
    // The anchored section arrives whole — nested subsections included,
    // the fence's fake heading doesn't end it early…
    assert!(out.contains("layout body line."), "{out}");
    assert!(out.contains("nested body stays in scope."), "{out}");
    assert!(out.contains("not a heading — fenced"), "{out}");
    // …and its siblings stay out: an excerpt, not the file.
    assert!(!out.contains("query body line."), "sibling leaked: {out}");

    // The cap is per link, in bytes, with a loud truncation marker.
    let filler = out.matches("filler line for the cap test").count();
    assert!(
        filler > 100 && filler < 145,
        "cap ≈4KB: {filler} filler lines survived"
    );
    assert!(out.contains("truncated"), "cap is loud: {out}");

    // A dead anchor and a dead path are loud lines, never a hard error.
    assert!(
        out.contains("§-9-missing") && out.contains("anchor not found"),
        "{out}"
    );
    assert!(
        out.contains("MISSING.md") && out.contains("not readable"),
        "{out}"
    );

    // Plain `show` stays excerpt-free — drill-through is opt-in (MW-D1).
    let plain = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(!plain.contains("layout body line."), "{plain}");
}

/// mw-7tseswy: ignored tail content surfaces IN the body area, where the
/// content would have rendered — stderr warnings scroll past as tool
/// noise while the reader stares at a missing body. The marker counts
/// the stranded lines and names the remedy.
#[test]
fn show_flags_ignored_tail_content() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Damaged by append");
    let path = task_file(&repo, &id);
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str("\nThis paragraph landed below the log by accident.\nAnd a second line.\n");
    std::fs::write(&path, text).unwrap();

    let out = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(out.contains("⚠ 2 line"), "counted marker: {out}");
    assert!(out.contains("lint --fix"), "names the remedy: {out}");
    let marker = out.find('⚠').unwrap();
    let log = out.find("\nlog:").unwrap();
    assert!(marker < log, "marker sits in the body area: {out}");

    // JSON parity (MW-C3).
    let js = stdout_of(
        &meshwork(&repo)
            .args(["show", &id, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["ignored_tail_lines"], 2);

    // A clean file renders no marker, and the JSON field is null.
    let clean = add_task(&repo, "Clean");
    let s = stdout_of(&meshwork(&repo).args(["show", &clean]).assert().success());
    assert!(!s.contains('⚠'), "{s}");
    let js = stdout_of(
        &meshwork(&repo)
            .args(["show", &clean, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert!(v["data"]["ignored_tail_lines"].is_null(), "{v}");
}
