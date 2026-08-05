// e2e part-file: graph verbs — dep add/rm, tree/why/blocked (M1).
// Included by e2e.rs.

/// PLAN 1.2 / MW-B8, C2: tree walks parent edges downward at any depth
/// with cosmetic level names; why prints the frontier of actually-open
/// blockers; blocked lists reasons. All three golden-pinned.
#[test]
fn tree_why_blocked_golden() {
    let (_g, repo) = fixture_repo("alpha");

    // tree: the 5-deep chain renders with level names by absolute depth.
    let tree = stdout_of(
        &meshwork(&repo)
            .args(["tree", "az-s4g0", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&tree).unwrap();
    let child = |node: &serde_json::Value, id: &str| -> serde_json::Value {
        node["children"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == id)
            .unwrap_or_else(|| panic!("{id} not under {}", node["id"]))
            .clone()
    };
    let root = &v["data"];
    assert_eq!(root["id"], "az-s4g0");
    assert_eq!(root["level"], "saga");
    let epic = child(root, "az-e9p2");
    assert_eq!(epic["level"], "epic");
    assert_eq!(
        epic["children"].as_array().unwrap().len(),
        2,
        "the epic has a second branch (az-e2f6)"
    );
    let sprint = child(&epic, "az-spr7");
    let story = child(&sprint, "az-st0r");
    let leaf = child(&story, "az-t5k1");
    assert_eq!(leaf["id"], "az-t5k1", "5-deep chain intact");
    assert_eq!(leaf["level"], serde_json::Value::Null, "past the names");
    crate::common::assert_golden("tree-alpha.json", &tree);

    // Text mode: indentation + level names, no JSON noise.
    let text = stdout_of(&meshwork(&repo).args(["tree", "az-s4g0"]).assert().success());
    assert!(text.contains("[saga]") && text.contains("[story]"), "{text}");

    // why: frontier only — both unmet deps are in-progress leaves.
    let why = stdout_of(
        &meshwork(&repo)
            .args(["why", "az-v4g9", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&why).unwrap();
    let ids: Vec<&str> = v["data"]["frontier"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["id"].as_str())
        .collect();
    assert_eq!(ids, ["az-d0w1", "az-t5k1"], "frontier, sorted: {why}");
    crate::common::assert_golden("why-alpha.json", &why);

    // Unresolved cross-repo deps surface as conservative frontier entries.
    let why_x = stdout_of(
        &meshwork(&repo)
            .args(["why", "az-x9b2", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&why_x).unwrap();
    assert_eq!(v["data"]["frontier"][0]["ref"], "beta#bz-c0r3");
    assert_eq!(v["data"]["frontier"][0]["unresolved"], true);

    // A met dep means an empty frontier.
    let why_ok = stdout_of(
        &meshwork(&repo)
            .args(["why", "az-n33d", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&why_ok).unwrap();
    assert_eq!(v["data"]["frontier"].as_array().unwrap().len(), 0);

    // blocked: the one blocked task with its reason.
    let blocked = stdout_of(&meshwork(&repo).args(["blocked", "--json"]).assert().success());
    let v: serde_json::Value = serde_json::from_str(&blocked).unwrap();
    assert_eq!(v["data"]["rows"][0]["id"], "az-b10k");
    assert!(v["data"]["rows"][0]["blocked_reason"]
        .as_str()
        .unwrap()
        .contains("datafusion 52"));
    crate::common::assert_golden("blocked-alpha.json", &blocked);
}

/// PLAN 1.1 / MW-B1: edge edits without opening the file — validated,
/// one-line diffs, reflected in the SQL tables.
#[test]
fn dep_edit() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let tx = add_task(&repo, "X depends on things");
    let ty = add_task(&repo, "Y the dependency");
    let tz = add_task(&repo, "Z another dependency");

    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", &ty])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &tx)).unwrap();
    assert!(text.contains(&format!("needs: [{ty}]")), "{text}");

    // Ready now excludes x (y is open).
    let ready = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    assert!(!ready.lines().any(|l| l.starts_with(&tx)), "{ready}");

    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", &tz])
        .assert()
        .success();
    let text = std::fs::read_to_string(task_file(&repo, &tx)).unwrap();
    assert!(text.contains(&format!("needs: [{ty}, {tz}]")), "{text}");

    // Guardrails: duplicates, self-deps, and dangling targets refuse.
    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", &ty])
        .assert()
        .failure()
        .stderr(predicates::str::contains("already"));
    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", &tx])
        .assert()
        .failure();
    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", "wo-none"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("wo-none"));
    // Cross-repo targets are the registry's business — allowed here.
    meshwork(&repo)
        .args(["dep", "add", &tx, "--needs", "beta#bz-c0r3"])
        .assert()
        .success();

    // The edge lands in the tables.
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", &format!("SELECT count(*) FROM edges WHERE src_gid='work#{tx}' AND kind='needs'")])
            .assert()
            .success(),
    );
    assert!(q.contains('3'), "{q}");

    // rm: down to one, then to none — the key line disappears entirely.
    meshwork(&repo)
        .args(["dep", "rm", &tx, "--needs", &ty])
        .assert()
        .success();
    meshwork(&repo)
        .args(["dep", "rm", &tx, "--needs", "beta#bz-c0r3"])
        .assert()
        .success();
    let json = stdout_of(
        &meshwork(&repo)
            .args(["dep", "rm", &tx, "--needs", &tz, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["verb"], "dep");
    let text = std::fs::read_to_string(task_file(&repo, &tx)).unwrap();
    assert!(!text.contains("needs:"), "empty list drops the key: {text}");

    meshwork(&repo)
        .args(["dep", "rm", &tx, "--needs", &ty])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not"));
}
