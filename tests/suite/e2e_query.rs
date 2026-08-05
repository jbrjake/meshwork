// e2e part-file: ready / q / JSON shapes (PLAN 0.8). Included by e2e.rs —
// tests here are `e2e::<name>`.

/// PLAN 0.8 / MW-B6, D1: `ready` over the kitchen-sink corpus matches the
/// committed golden byte-for-byte. Semantics pinned explicitly first so the
/// golden can't drift into nonsense via a careless bless.
#[test]
fn ready_golden() {
    let (_g, repo) = fixture_repo("alpha");
    let js = stdout_of(&meshwork(&repo).args(["ready", "--json"]).assert().success());

    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let ids: Vec<&str> = v["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"az-n33d"), "met hard dep → ready: {ids:?}");
    assert!(
        !ids.contains(&"az-s4g0") && !ids.contains(&"az-e9p2"),
        "containers with live children are not actionable (MW-B6): {ids:?}"
    );
    assert!(
        !ids.contains(&"az-g4m8"),
        "unresolved absent-repo dep blocks conservatively (MW-G5): {ids:?}"
    );
    assert!(!ids.contains(&"az-w8t1"), "unmet hard dep blocks: {ids:?}");
    assert!(
        !ids.contains(&"az-t5k1") && !ids.contains(&"az-b10k"),
        "only status=open is ready: {ids:?}"
    );
    assert_eq!(ids[0], "az-n33d", "seq 20 sorts before seq 80 and unset");
    assert_eq!(ids[1], "az-r3l8", "seq 80 second");

    crate::common::assert_golden("ready-alpha.json", &js);

    let text = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    assert!(text.lines().next().unwrap().starts_with("az-n33d"));
}

/// MW-C1: real SQL over the five virtual tables — no bespoke language.
#[test]
fn raw_sql_tables() {
    let (_g, repo) = fixture_repo("alpha");
    for (sql, expect) in [
        ("SELECT count(*) FROM tasks", "33"),
        ("SELECT count(*) FROM edges WHERE kind='needs'", "9"),
        ("SELECT count(*) FROM repos", "1"),
        (
            "SELECT id FROM tasks WHERE waived IS NOT NULL",
            "az-w4v3",
        ),
        (
            "SELECT label FROM labels WHERE gid='alpha#az-t5k1' ORDER BY label LIMIT 1",
            "p0",
        ),
        (
            "SELECT author FROM comments WHERE gid='alpha#az-c0m9' AND ord=1",
            "jon",
        ),
    ] {
        let out = stdout_of(&meshwork(&repo).args(["q", sql]).assert().success());
        assert!(out.contains(expect), "q `{sql}` → {out:?}, want {expect}");
    }

    // JSON: typed cells inside the stable envelope.
    let js = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT id, seq FROM tasks WHERE id='az-n33d'", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["columns"], serde_json::json!(["id", "seq"]));
    assert_eq!(v["data"]["rows"], serde_json::json!([["az-n33d", 20]]));

    // Bad SQL fails loudly, exit 1.
    meshwork(&repo)
        .args(["q", "SELEKT nope"])
        .assert()
        .failure();
}

/// MW-C3: one stable, versioned JSON envelope across every verb so far.
#[test]
fn json_stable_shapes() {
    let (_g, repo) = git_repo("work");
    let check = |assert: &assert_cmd::assert::Assert, verb: &str| {
        let v: serde_json::Value = serde_json::from_str(&stdout_of(assert))
            .unwrap_or_else(|e| panic!("{verb}: bad JSON: {e}"));
        assert_eq!(v["v"], 1, "{verb}: envelope version");
        assert_eq!(v["verb"], verb);
        assert!(v["data"].is_object(), "{verb}: data object");
    };
    check(&meshwork(&repo).args(["init", "--json"]).assert().success(), "init");
    let id = {
        let a = meshwork(&repo)
            .args(["add", "Enveloped", "--verify", "true", "--json"])
            .assert()
            .success();
        check(&a, "add");
        serde_json::from_str::<serde_json::Value>(&stdout_of(&a)).unwrap()["data"]["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    check(&meshwork(&repo).args(["show", &id, "--json"]).assert().success(), "show");
    check(&meshwork(&repo).args(["start", &id, "--json"]).assert().success(), "start");
    check(
        &meshwork(&repo)
            .args(["block", &id, "--reason", "r", "--json"])
            .assert()
            .success(),
        "block",
    );
    check(&meshwork(&repo).args(["reopen", &id, "--json"]).assert().success(), "reopen");
    check(&meshwork(&repo).args(["close", &id, "--json"]).assert().success(), "close");
    check(&meshwork(&repo).args(["ready", "--json"]).assert().success(), "ready");
    check(
        &meshwork(&repo)
            .args(["q", "SELECT 1 AS one", "--json"])
            .assert()
            .success(),
        "q",
    );
}

/// PLAN 0.11 / MW-H5, J6: every non-mirror verb runs clean on a PATH that
/// holds exactly git and sh — no gh binary exists, no network is reachable
/// by construction (nothing to call it with).
#[test]
fn offline_all() {
    let bin = tempfile::tempdir().unwrap();
    for tool in ["git", "sh"] {
        let out = std::process::Command::new("sh")
            .args(["-c", &format!("command -v {tool}")])
            .output()
            .unwrap();
        let real = String::from_utf8_lossy(&out.stdout).trim().to_string();
        std::os::unix::fs::symlink(&real, bin.path().join(tool)).unwrap();
    }
    let path_env = bin.path().to_string_lossy().into_owned();

    // Sanity: gh must NOT resolve under this PATH.
    let gh = std::process::Command::new("sh")
        .args(["-c", "command -v gh"])
        .env("PATH", &path_env)
        .output()
        .unwrap();
    assert!(!gh.status.success(), "PATH must have no gh");

    let (_g, repo) = git_repo("work");
    let run = |args: &[&str]| {
        meshwork(&repo)
            .env("PATH", &path_env)
            .args(args)
            .assert()
            .success();
    };
    run(&["init"]);
    let id = {
        let out = meshwork(&repo)
            .env("PATH", &path_env)
            .args(["add", "Fully offline", "--cat", "core", "--label", "x", "--verify", "true"])
            .assert()
            .success();
        stdout_of(&out).lines().next().unwrap().to_string()
    };
    run(&["show", &id]);
    run(&["start", &id]);
    run(&["block", &id, "--reason", "checking offline"]);
    run(&["reopen", &id]);
    run(&["ready"]);
    run(&["q", "SELECT count(*) FROM tasks"]);
    run(&["lint"]);
    run(&["close", &id]); // verify `true` runs via sh -c on the bare PATH
}

/// MW-A2: the cache is an optimization, never a dependency — deleting
/// .cache (or the whole layout reservation) is always safe.
#[test]
fn cache_delete_safe() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Cacheless");
    std::fs::remove_dir_all(repo.join("meshwork/.cache")).unwrap();
    meshwork(&repo).arg("ready").assert().success();
    meshwork(&repo).args(["show", &id]).assert().success();
    meshwork(&repo).arg("lint").assert().success();
    meshwork(&repo)
        .args(["q", "SELECT count(*) FROM tasks"])
        .assert()
        .success();
}

/// MW-D2: listings cap at 20 with the explicit `… and N more` marker;
/// `--all` is the opt-out.
#[test]
fn caps_and_more_marker() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    for i in 1..=25 {
        add_task(&repo, &format!("Task number {i:02}"));
    }
    let text = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    let rows = text.lines().filter(|l| l.starts_with("wo-")).count();
    assert_eq!(rows, 20, "capped at 20:\n{text}");
    assert!(text.contains("… and 5 more"), "{text}");

    let all = stdout_of(&meshwork(&repo).args(["ready", "--all"]).assert().success());
    assert_eq!(all.lines().filter(|l| l.starts_with("wo-")).count(), 25);
    assert!(!all.contains("… and"), "{all}");

    let js = stdout_of(&meshwork(&repo).args(["ready", "--json"]).assert().success());
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["total"], 25);
    assert_eq!(v["data"]["rows"].as_array().unwrap().len(), 20);
}
