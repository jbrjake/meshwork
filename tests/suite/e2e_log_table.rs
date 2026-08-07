// mw-3wnhhvp: the normative log-line grammar becomes SQL — `log` joins the
// contract as the sixth table (gid, ord, date, from_status, to_status,
// note). Transition lines project from/to; anything else is free text with
// both NULL. Parsing is positional and never validates history: date-only
// and free-text lines stay legal forever (MW-E3/C1).

/// Every minted form parses: `created`, start's claim suffix, the failed
/// close attempt, block's reason, close's waive, and verify exit 0 — with
/// minute-res stamps in the date column (mw-zp1h12d).
#[test]
fn log_table_minted_forms() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_id(&repo, &["add", "graph me", "--verify", "false"]);
    meshwork(&repo)
        .args(["start", &id, "--as", "maya"])
        .assert()
        .success();
    meshwork(&repo).args(["close", &id]).assert().failure();
    meshwork(&repo)
        .args(["block", &id, "--reason", "waiting on rig"])
        .assert()
        .success();
    meshwork(&repo)
        .args(["close", &id, "--waive", "obsolete"])
        .assert()
        .success();

    let js = stdout_of(
        &meshwork(&repo)
            .args([
                "q",
                "SELECT date, from_status, to_status, note FROM log ORDER BY ord",
                "--json",
            ])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let rows = v["data"]["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 5, "{rows:?}");

    // Dates are minted minute stamps: YYYY-MM-DDTHH:MMZ (mw-zp1h12d).
    for row in rows {
        let date = row[0].as_str().unwrap();
        assert!(
            date.len() == 17 && date.ends_with('Z'),
            "minute stamp: {date}"
        );
    }
    let shape: Vec<Vec<Option<&str>>> = rows
        .iter()
        .map(|r| (1..4).map(|i| r[i].as_str()).collect())
        .collect();
    assert_eq!(
        shape,
        [
            vec![None, None, Some("created")],
            vec![Some("open"), Some("doing"), Some("claimed by maya")],
            vec![None, None, Some("close attempt — verify exit 1")],
            vec![Some("doing"), Some("blocked"), Some("waiting on rig")],
            vec![Some("blocked"), Some("done"), Some("waived: obsolete")],
        ],
        "{rows:?}"
    );

    // The remaining minted form: a clean close → `— verify exit 0`.
    let ok = add_id(&repo, &["add", "closes clean", "--verify", "true"]);
    meshwork(&repo).args(["close", &ok]).assert().success();
    let js = stdout_of(
        &meshwork(&repo)
            .args([
                "q",
                &format!(
                    "SELECT from_status, to_status, note FROM log \
                     WHERE gid LIKE '%#{ok}' AND to_status = 'done'"
                ),
                "--json",
            ])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(
        v["data"]["rows"],
        serde_json::json!([["open", "done", "verify exit 0"]])
    );
}

/// The kitchen-sink corpus queries cleanly: date-only historical transitions
/// keep their dates, notes split on the em dash, free text stays NULL/NULL.
#[test]
fn log_table_fixture_corpus() {
    let (_g, repo) = fixture_repo("alpha");
    let q_json = |sql: &str| -> serde_json::Value {
        let js = stdout_of(&meshwork(&repo).args(["q", sql, "--json"]).assert().success());
        serde_json::from_str::<serde_json::Value>(&js).unwrap()["data"]["rows"].clone()
    };

    let rows = q_json(
        "SELECT date, from_status, to_status, note FROM log \
         WHERE gid='alpha#az-d0n3' ORDER BY ord",
    );
    assert_eq!(
        rows,
        serde_json::json!([
            ["2026-07-30", "open", "doing", null],
            ["2026-08-01", "doing", "done", "verify exit 0"],
        ])
    );

    let rows = q_json("SELECT date, from_status, to_status, note FROM log WHERE gid='alpha#az-s4g0'");
    assert_eq!(rows, serde_json::json!([["2026-08-01", null, null, "created"]]));

    // The unlocked query class: done-transition dates via plain SQL.
    let rows = q_json(
        "SELECT count(*) FROM log WHERE to_status='done' AND note LIKE 'verify exit 0%'",
    );
    assert!(rows[0][0].as_i64().unwrap() >= 3, "{rows:?}");
}

/// Parse never validates history: hand-written junk — dateless free text,
/// nonsense statuses, a bare date — projects as free-text rows, warns
/// nothing, breaks nothing (MW-I2 visibility, not judgment).
#[test]
fn log_table_legacy_lines_stay_legal() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    std::fs::write(
        repo.join("docs/meshwork/zz-legacy1-old-log-style.md"),
        "---\nid: zz-legacy1\ntitle: Old log style\nstatus: done\nverify: \"true\"\n---\n\n\
         ## log\n\
         - 2026-08-01 open→doing\n\
         - migrated from TODO.md\n\
         - 2026-08-02 fixed→done — nonsense statuses stay free text\n\
         - 2026-08-03\n\
         - 2026-08-04 doing→done — verify exit 0\n",
    )
    .unwrap();

    let js = stdout_of(
        &meshwork(&repo)
            .args([
                "q",
                "SELECT date, from_status, to_status, note FROM log ORDER BY ord",
                "--json",
            ])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(
        v["data"]["rows"],
        serde_json::json!([
            ["2026-08-01", "open", "doing", null],
            ["migrated", null, null, "from TODO.md"],
            ["2026-08-02", null, null, "fixed→done — nonsense statuses stay free text"],
            ["2026-08-03", null, null, null],
            ["2026-08-04", "doing", "done", "verify exit 0"],
        ])
    );

    // Still a valid task everywhere — free-text log lines are not damage.
    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(lint.contains("0 error(s)"), "{lint}");
    let show = stdout_of(&meshwork(&repo).args(["show", "zz-legacy1"]).assert().success());
    assert!(show.contains("migrated from TODO.md"), "{show}");

    // And prime's recently-done reads the grammar, not a substring hunt:
    // the last →done transition dates this task 2026-08-04.
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(
        prime.contains("2026-08-04 zz-legacy1"),
        "recently-done via grammar:\n{prime}"
    );
}
