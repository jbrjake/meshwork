// mw-5xdyxep (owner ask 2026-08-21): full-text search across task content.
// `search <term>` is a canned-SQL verb — a literal substring matched
// case-insensitively over title, body, handoff, comments, and log notes,
// archives included — never a pattern language (REQUIREMENTS §3's fence).

/// The corpus is everything a session might remember writing: title, body,
/// handoff, comment text, log notes — and archived tasks, where stale
/// knowledge lives. Hits list live tasks before terminal ones.
#[test]
fn full_text_search() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    let in_body = add_id(&repo, &["add", "Pump rework", "--verify", "true"]);
    let path = task_file(&repo, &in_body);
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replace(
            "\n## log\n",
            "The spillway gasket narrative lives here.\n\n## log\n",
        ),
    )
    .unwrap();

    let in_handoff = add_id(&repo, &["add", "Valve audit", "--verify", "true"]);
    meshwork(&repo)
        .args(["set", &in_handoff, "--handoff", "next session: check the SPILLWAY drain"])
        .assert()
        .success();

    let in_comment = add_id(&repo, &["add", "Filter swap", "--verify", "true"]);
    meshwork(&repo)
        .args(["comment", &in_comment, "--as", "maya", "the spillway leak traces here"])
        .assert()
        .success();

    let in_title = add_id(&repo, &["add", "Spillway ledger", "--verify", "true"]);

    // An archived (closed) hit still surfaces — after the live ones.
    let archived = add_id(&repo, &["add", "Old spillway notes", "--verify", "true"]);
    meshwork(&repo).args(["close", &archived]).assert().success();

    let miss = add_id(&repo, &["add", "Unrelated", "--verify", "true"]);

    // Case-insensitive literal substring; live before terminal.
    let out = stdout_of(&meshwork(&repo).args(["search", "Spillway"]).assert().success());
    for id in [&in_body, &in_handoff, &in_comment, &in_title, &archived] {
        assert!(out.contains(id.as_str()), "missing {id}:\n{out}");
    }
    assert!(!out.contains(miss.as_str()), "{out}");
    let pos = |id: &str| out.find(id).unwrap();
    assert!(pos(&archived) > pos(&in_body), "live first:\n{out}");
    assert!(pos(&archived) > pos(&in_title), "live first:\n{out}");

    // Matched-field context: the field name and the matching line surface.
    assert!(out.contains("body:"), "{out}");
    assert!(out.contains("spillway gasket narrative"), "{out}");
    assert!(out.contains("handoff:"), "{out}");
    assert!(out.contains("comment:"), "{out}");

    // A metacharacter is literal text, never a wildcard.
    let none = stdout_of(&meshwork(&repo).args(["search", "spill%"]).assert().success());
    assert!(!none.contains(in_body.as_str()), "wildcard leaked:\n{none}");

    // JSON rides the envelope with the listing idiom.
    let js = stdout_of(
        &meshwork(&repo)
            .args(["search", "spillway", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["verb"], "search", "{js}");
    assert_eq!(v["data"]["total"], 5, "{js}");
    let rows = v["data"]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 5, "{js}");
    let first = &rows[0];
    assert!(first["matches"].as_array().is_some(), "{js}");
}

/// MW-D2: search honors the 20-row cap with the more-marker; `--all` opts
/// out. The handoff column joins the projection for `q` too.
#[test]
fn full_text_search_cap_and_projection() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    for i in 1..=25 {
        add_task(&repo, &format!("Gasket item {i:02}"));
    }
    let out = stdout_of(&meshwork(&repo).args(["search", "gasket"]).assert().success());
    assert_eq!(
        out.lines().filter(|l| l.starts_with("wo-")).count(),
        20,
        "{out}"
    );
    assert!(out.contains("… and 5 more"), "{out}");
    let all = stdout_of(&meshwork(&repo).args(["search", "gasket", "--all"]).assert().success());
    assert_eq!(all.lines().filter(|l| l.starts_with("wo-")).count(), 25);

    // tasks.handoff is queryable directly; absent key is NULL, not ''.
    let id = add_id(&repo, &["add", "Handoff carrier", "--verify", "true"]);
    meshwork(&repo)
        .args(["set", &id, "--handoff", "resume at the drain check"])
        .assert()
        .success();
    let hits = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT id FROM tasks WHERE handoff LIKE '%drain check%'"])
            .assert()
            .success(),
    );
    assert!(hits.contains(&id), "{hits}");
    let nulls = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT count(*) AS n FROM tasks WHERE handoff IS NULL"])
            .assert()
            .success(),
    );
    assert!(nulls.contains("25"), "absent handoff must be NULL:\n{nulls}");
}
