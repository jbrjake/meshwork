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

/// MW-M3: `portfolio search` is `search`'s canned SQL over every
/// registered store — hits grouped by repo under a header with the
/// repo's full count, rows named `repo#id`, live before terminal within
/// a group, the MW-D2 cap with `--all`, absent repos reported.
#[test]
fn portfolio_search() {
    let (dir, portfolio) = portfolio_fixture();
    let beta = dir.path().join("beta");
    // alpha carries 14 tasks naming spill; eight more in beta cross the cap.
    let beta_hit = add_id(&beta, &["add", "Spill retry policy", "--verify", "true"]);
    for i in 2..=8 {
        add_task(&beta, &format!("Spill retry policy {i}"));
    }
    let run = |args: &[&str]| {
        meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(args)
            .assert()
            .success()
    };

    let out = stdout_of(&run(&["portfolio", "search", "spill"]));
    let alpha_at = out.find("alpha (").expect(&out);
    let beta_at = out.find("beta (8 hits):").expect(&out);
    assert!(alpha_at < beta_at, "grouped by repo, in order:\n{out}");
    assert!(out.contains("alpha#az-e9p2  "), "rows are named repo#id:\n{out}");
    let rows = |text: &str| {
        text.lines()
            .filter(|l| l.starts_with("alpha#") || l.starts_with("beta#"))
            .count()
    };
    assert_eq!(rows(&out), 20, "the cap holds:\n{out}");
    assert!(out.contains("… and "), "{out}");
    // Within alpha's group no terminal row precedes a live one.
    let alpha_rows: Vec<&str> = out[alpha_at..beta_at]
        .lines()
        .filter(|l| l.starts_with("alpha#"))
        .collect();
    let first_terminal = alpha_rows
        .iter()
        .position(|l| l.ends_with("[done]") || l.ends_with("[dropped]"));
    if let Some(at) = first_terminal {
        assert!(
            alpha_rows[at..].iter().all(|l| !l.ends_with("[open]")),
            "live first:\n{out}"
        );
    }

    let all = stdout_of(&run(&["portfolio", "search", "spill", "--all"]));
    assert!(rows(&all) > 20 && !all.contains("… and "), "{all}");
    assert!(all.contains(&format!("beta#{beta_hit}  Spill retry policy [open]")), "{all}");

    let v: serde_json::Value = serde_json::from_str(&stdout_of(&run(&[
        "portfolio", "search", "spill", "--json",
    ])))
    .unwrap();
    assert_eq!(v["verb"], "portfolio search", "{v}");
    assert_eq!(
        v["data"]["total"].as_u64(),
        Some(u64::try_from(rows(&all)).unwrap()),
        "{v}"
    );
    assert_eq!(v["data"]["rows"].as_array().map(Vec::len), Some(20), "{v}");
    assert_eq!(v["data"]["rows"][0]["repo"], "alpha", "{v}");
    assert!(v["data"]["rows"][0]["gid"].as_str().unwrap().starts_with("alpha#"), "{v}");
    assert_eq!(v["data"]["skipped"][0]["repo"], "gamma", "{v}");
}

/// MW-M3 / MW-S14: `portfolio search` is a read — the overlay is never
/// rewritten under it, in either output mode.
#[test]
fn portfolio_search_never_prunes() {
    let (dir, portfolio) = portfolio_fixture();
    let before = "# sequence\n\n- beta#bz-c0r3\n- alpha#az-n33d\n";
    std::fs::write(portfolio.join("sequence.md"), before).unwrap();
    for json in [false, true] {
        let mut args = vec!["portfolio", "search", "reader"];
        if json {
            args.push("--json");
        }
        let assert = meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(&args)
            .assert()
            .success();
        let err = stderr_of(&assert);
        assert!(!err.contains("pruned"), "a read never prunes (json={json}): {err}");
        assert_eq!(err.contains("skipped gamma"), !json, "skips on stderr in text only: {err}");
        assert_eq!(std::fs::read_to_string(portfolio.join("sequence.md")).unwrap(), before);
    }
}
