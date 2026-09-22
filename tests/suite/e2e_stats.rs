// e2e part-file: `stats` and `portfolio stats` (mw-549rh9w, MW-M1) —
// every table a canned SELECT over a view, the pulse first, `--json`
// carrying each table as `{columns, rows}`. Included by e2e.rs — tests
// here are `e2e::<name>`.

/// The sections, in order, and the JSON keys each one lands under.
const STATS_TABLES: &[(&str, &str)] = &[
    ("pulse", "pulse ("),
    ("flow_weekly", "flow, weekly"),
    ("hazard", "close hazard"),
    ("spans", "spans by state"),
    ("lanes", "lanes ("),
    ("top_gravity", "top 10 by gravity"),
    ("top_spawners", "top 10 spawners"),
    ("top_touched", "top 10 touched since last move"),
    ("mentions", "mentions:"),
    ("placement", "placement ("),
];

fn stats_json(repo: &Path, flags: &[&str]) -> serde_json::Value {
    let mut command = vec!["stats"];
    command.extend_from_slice(flags);
    command.push("--json");
    serde_json::from_str(&stdout_of(
        &meshwork(repo)
            .env("MESHWORK_TODAY", PULSE_STAMP)
            .args(&command)
            .assert()
            .success(),
    ))
    .unwrap()
}

/// MW-M1: `stats` on the conformance store at the blessed stamp — every
/// section present in text and JSON, the pulse table equal to
/// `SELECT * FROM pulse`, the hazard and flow tables equal to the views
/// they pool, and `--window` reaching the clock.
#[test]
fn stats_tables() {
    let (_g, repo) = fixture_repo("conformance");
    let text = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_TODAY", PULSE_STAMP)
            .arg("stats")
            .assert()
            .success(),
    );
    let mut at = 0;
    for (_, heading) in STATS_TABLES {
        let pos = text[at..]
            .find(heading)
            .unwrap_or_else(|| panic!("`{heading}` after byte {at} in:\n{text}"));
        at += pos;
    }
    assert!(text.contains("- flow 7d:"), "the pulse block leads: {text}");
    assert!(text.contains("placed by:"), "placement carries its note: {text}");

    let v = stats_json(&repo, &[]);
    assert_eq!(v["verb"], "stats");
    assert_eq!(v["data"]["window_days"], 7);
    let tables = &v["data"]["tables"];
    for (key, _) in STATS_TABLES {
        assert!(
            tables[key]["columns"].is_array() && tables[key]["rows"].is_array(),
            "{key} as {{columns, rows}}: {tables}"
        );
    }

    // The pulse table IS the view row.
    let view: serde_json::Value = serde_json::from_str(&stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_TODAY", PULSE_STAMP)
            .args(["q", "SELECT * FROM pulse ORDER BY repo", "--json"])
            .assert()
            .success(),
    ))
    .unwrap();
    assert_eq!(tables["pulse"]["columns"], view["data"]["columns"]);
    assert_eq!(tables["pulse"]["rows"], view["data"]["rows"]);

    // Hazard bin 0 pools the view; survival is a non-increasing product.
    let hazard = tables["hazard"]["rows"].as_array().unwrap();
    let pooled = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_TODAY", PULSE_STAMP)
            .args(["q", "SELECT sum(at_risk) FROM hazard WHERE age_d = 0"])
            .assert()
            .success(),
    );
    assert_eq!(hazard[0][0], 0);
    assert!(
        pooled.contains(&hazard[0][1].to_string()),
        "bin 0 at_risk {} in {pooled}",
        hazard[0][1]
    );
    let survival: Vec<f64> = hazard.iter().map(|r| r[6].as_f64().unwrap()).collect();
    assert!(survival.windows(2).all(|w| w[1] <= w[0]), "{survival:?}");
    assert!(survival.iter().all(|s| (0.0..=1.0).contains(s)));
    // Only the listed bins show, ascending; a young store fills few.
    let bins: Vec<i64> = hazard.iter().map(|r| r[0].as_i64().unwrap()).collect();
    assert!(
        bins.iter().all(|b| (0..=14).contains(b) || *b == 21 || *b == 28),
        "{bins:?}"
    );
    assert!(bins.windows(2).all(|w| w[0] < w[1]), "{bins:?}");

    // The clock's week is first; the weeks together carry every filing.
    let flow = tables["flow_weekly"]["rows"].as_array().unwrap();
    assert_eq!(flow[0][0], "2026-08-10", "{flow:?}");
    let filed: i64 = flow.iter().map(|r| r[1].as_i64().unwrap()).sum();
    let total = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_TODAY", PULSE_STAMP)
            .args(["q", "SELECT sum(filed) FROM flow"])
            .assert()
            .success(),
    );
    assert!(total.contains(&filed.to_string()), "filed {filed} in {total}");

    // Placement counts the live tasks the pulse counts.
    let pulse: std::collections::BTreeMap<String, serde_json::Value> = tables["pulse"]["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().to_string())
        .zip(tables["pulse"]["rows"][0].as_array().unwrap().iter().cloned())
        .collect();
    let live = pulse["open_n"].as_i64().unwrap()
        + pulse["doing_n"].as_i64().unwrap()
        + pulse["blocked_n"].as_i64().unwrap();
    assert_eq!(tables["placement"]["rows"][0][1], live, "{tables}");
    assert_eq!(tables["mentions"]["rows"][0][0], pulse["handoff_stale_n"]);
}

/// MW-M1: `--window` reaches the clock — a wider window counts at least
/// as much as the default — and anything but whole days is refused.
#[test]
fn stats_window_reaches_the_clock() {
    let (_g, repo) = fixture_repo("conformance");
    let filed_w = |v: &serde_json::Value| {
        let pulse = &v["data"]["tables"]["pulse"];
        let at = pulse["columns"]
            .as_array()
            .unwrap()
            .iter()
            .position(|c| c == "filed_w")
            .unwrap();
        pulse["rows"][0][at].as_i64().unwrap()
    };
    let narrow = stats_json(&repo, &["--window", "1d"]);
    let wide = stats_json(&repo, &["--window", "28d"]);
    assert_eq!(narrow["data"]["window_days"], 1);
    assert_eq!(wide["data"]["window_days"], 28);
    assert!(
        filed_w(&wide) > filed_w(&narrow),
        "28d {} vs 1d {}",
        filed_w(&wide),
        filed_w(&narrow)
    );
    meshwork(&repo)
        .args(["stats", "--window", "fortnight"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("whole number of days"));
}

/// MW-M1: `portfolio stats` over the union — one pulse row per repo and
/// a total in text, the view rows in JSON with the skip list; a pure
/// read that never prunes sequence.md (MW-S14).
#[test]
fn portfolio_stats() {
    let (dir, portfolio) = portfolio_fixture();
    let before = "# sequence\n\n- beta#bz-c0r3\n- alpha#az-n33d\n";
    std::fs::write(portfolio.join("sequence.md"), before).unwrap();

    let assert = meshwork(dir.path())
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .env("MESHWORK_TODAY", PULSE_STAMP)
        .args(["portfolio", "stats"])
        .assert()
        .success();
    let text = stdout_of(&assert);
    let err = stderr_of(&assert);
    assert!(text.starts_with("pulse (7d window):\n"), "{text}");
    for needle in ["  alpha ", "  beta ", "  total "] {
        assert!(text.contains(needle), "`{needle}` in:\n{text}");
    }
    assert!(text.contains("alpha#az-"), "the union keeps ids whole: {text}");
    assert!(err.contains("skipped gamma"), "{err}");
    assert!(!err.contains("pruned"), "stats is a pure read: {err}");
    assert_eq!(std::fs::read_to_string(portfolio.join("sequence.md")).unwrap(), before);

    let v: serde_json::Value = serde_json::from_str(&stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .env("MESHWORK_TODAY", PULSE_STAMP)
            .args(["portfolio", "stats", "--window", "28d", "--json"])
            .assert()
            .success(),
    ))
    .unwrap();
    assert_eq!(v["verb"], "portfolio stats");
    assert_eq!(v["data"]["window_days"], 28);
    let repos: Vec<&str> = v["data"]["tables"]["pulse"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r[0].as_str().unwrap())
        .collect();
    assert_eq!(repos, ["alpha", "beta"], "{v}");
    assert_eq!(v["data"]["skipped"][0]["repo"], "gamma", "{v}");
    let placement = v["data"]["tables"]["placement"]["rows"].as_array().unwrap();
    assert_eq!(placement.len(), 2, "one placement row per repo: {placement:?}");
}
