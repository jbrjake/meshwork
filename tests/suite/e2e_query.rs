// e2e part-file: ready / q / JSON shapes (PLAN 0.8). Included by e2e.rs —
// tests here are `e2e::<name>`.

/// PLAN 0.8 / MW-B6, D1: `ready` over the kitchen-sink corpus matches the
/// committed golden byte-for-byte. Semantics pinned explicitly first so the
/// golden can't drift into nonsense via a careless bless.
#[test]
fn ready_golden() {
    // Registry context included since 2.3 (mw-k7r5): the kitchen-sink run
    // exercises cross-repo resolution — beta present, gamma absent.
    let (dir, portfolio) = portfolio_fixture();
    let repo = dir.path().join("alpha");
    let js = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["ready", "--json"])
            .assert()
            .success(),
    );

    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let ids: Vec<&str> = v["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"az-n33d"), "met hard dep → ready: {ids:?}");
    assert!(
        ids.contains(&"az-x9b2"),
        "cross-repo dep on done beta#bz-c0r3 resolves (MW-B3, mw-k7r5): {ids:?}"
    );
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
    assert_eq!(ids[1], "az-x9b2", "seq 40 slots in once its dep resolves");
    assert_eq!(ids[2], "az-r3l8", "seq 80 third");

    crate::common::assert_golden("ready-alpha.json", &js);

    let text = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    assert!(text.lines().next().unwrap().starts_with("az-n33d"));
}

/// MW-C1: real SQL over the six virtual tables — no bespoke language.
#[test]
fn raw_sql_tables() {
    let (_g, repo) = fixture_repo("alpha");
    for (sql, expect) in [
        ("SELECT count(*) FROM tasks", "33"),
        ("SELECT count(*) FROM edges WHERE kind='needs'", "9"),
        ("SELECT count(*) FROM repos", "1"),
        (
            "SELECT date FROM log WHERE gid='alpha#az-d0n3' AND to_status='done'",
            "2026-08-01",
        ),
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
        assert_eq!(v["meshwork"]["schema"], 2, "{verb}: envelope schema");
        assert_eq!(
            v["meshwork"]["version"],
            env!("CARGO_PKG_VERSION"),
            "{verb}: in-band version (mw-5kp033j)"
        );
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

/// PLAN 1.7 / MW-J3: `import todo` converts the baseline checkbox format
/// — [ ]/[~]/[x]/[!], bold titles, indented verify: lines, ## Now → seq —
/// into a golden-pinned task set. The source file stays untouched.
#[test]
fn import_todo_golden() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let sample = fixtures_root().join("import/TODO-sample.md");
    let before = std::fs::read_to_string(&sample).unwrap();

    let out = stdout_of(
        &meshwork(&repo)
            .env("MESHWORK_ID_SEED", "7")
            .env("MESHWORK_TODAY", "2026-08-04")
            .args(["import", "todo", sample.to_str().unwrap()])
            .assert()
            .success(),
    );
    assert!(out.contains("5 imported"), "{out}");

    // Byte-stable task set (seeded ids + fixed date) → one golden blob.
    let tasks_dir = repo.join("docs/meshwork");
    // Root + archive/ — done imports land archived (mw-45e2qf4) and must
    // stay pinned by the golden.
    let mut names: Vec<String> = [("", tasks_dir.clone()), ("archive/", tasks_dir.join("archive"))]
        .into_iter()
        .filter_map(|(prefix, dir)| std::fs::read_dir(dir).ok().map(|rd| (prefix, rd)))
        .flat_map(|(prefix, rd)| {
            rd.map(|e| e.unwrap().path())
                .filter(|p| {
                    // flat store: skip config/attrs — same filter as the loader
                    p.extension()
                        .is_some_and(|x| x.eq_ignore_ascii_case("md"))
                })
                .map(move |p| {
                    format!("{prefix}{}", p.file_name().unwrap().to_string_lossy())
                })
                .collect::<Vec<_>>()
        })
        .collect();
    names.sort();
    let mut blob = String::new();
    for name in &names {
        blob.push_str("=== ");
        blob.push_str(name);
        blob.push('\n');
        blob.push_str(&std::fs::read_to_string(tasks_dir.join(name)).unwrap());
        blob.push('\n');
    }
    crate::common::assert_golden("import-todo.md", &blob);

    // Semantics: statuses mapped, Now ordering → seq 10/20/30, verify
    // extracted, blocked-reason carried, source untouched.
    let counts = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT status, count(*) FROM tasks GROUP BY status ORDER BY status"])
            .assert()
            .success(),
    );
    // [~] lands open, not unclaimed doing (mw-x5a8g9w) — hence 3 open.
    for expected in ["blocked | 1", "done | 1", "open | 3"] {
        assert!(counts.contains(expected), "{counts}");
    }
    assert!(!counts.contains("doing"), "no unclaimed doing rows: {counts}");
    let seqs = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT seq FROM tasks WHERE seq IS NOT NULL ORDER BY seq"])
            .assert()
            .success(),
    );
    assert!(seqs.contains("10") && seqs.contains("20") && seqs.contains("30"), "{seqs}");

    let blocked = stdout_of(&meshwork(&repo).args(["blocked"]).assert().success());
    assert!(blocked.contains("52 unreleased"), "{blocked}");

    // Exactly one lint warning: the Later item without a verify.
    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(lint.contains("no-verify"), "{lint}");

    assert_eq!(before, std::fs::read_to_string(&sample).unwrap());
}

/// PLAN 1.6 / MW-D4, REQUIREMENTS §3: the CLI surface IS DESIGN §6 —
/// verbatim, nothing more, nothing less. A verb that isn't in this list is
/// a non-goal; adding one here requires an owner ruling amending §3.
#[test]
fn cli_surface_frozen() {
    let (_g, repo) = git_repo("work");
    let help = {
        let out = meshwork(&repo).arg("--help").assert().success();
        stdout_of(&out)
    };
    let verbs: Vec<String> = help
        .lines()
        .skip_while(|l| !l.starts_with("Commands:"))
        .skip(1)
        .take_while(|l| l.starts_with("  "))
        .filter_map(|l| l.split_whitespace().next().map(ToString::to_string))
        .collect();
    assert_eq!(
        verbs,
        [
            "init", "add", "set", "show", "comment", "attach", "start", "block", "drop", "reopen",
            "close", "verify", "dep", "ready", "blocked", "tree", "why", "q", "search", "prime",
            "lint", "mirror", "portfolio", "import",
        ],
        "DESIGN §6, frozen:\n{help}"
    );

    // Sub-surfaces are frozen too.
    let subs = |args: &[&str], expected: &[&str]| {
        let out = meshwork(&repo).args(args).assert().success();
        let help = stdout_of(&out);
        let got: Vec<String> = help
            .lines()
            .skip_while(|l| !l.starts_with("Commands:"))
            .skip(1)
            .take_while(|l| l.starts_with("  "))
            .filter_map(|l| l.split_whitespace().next().map(ToString::to_string))
            .collect();
        assert_eq!(got, expected, "{args:?}:\n{help}");
    };
    subs(&["dep", "--help"], &["add", "rm"]);
    subs(&["mirror", "--help"], &["push", "status"]);
    subs(&["portfolio", "--help"], &["ready", "next", "q", "seq"]);
    subs(&["import", "--help"], &["todo"]);

    // show carries --docs and --comments (behavior for --docs lands M4).
    let show_help = stdout_of(&meshwork(&repo).args(["show", "--help"]).assert().success());
    assert!(show_help.contains("--docs") && show_help.contains("--comments"));

    // set carries the ruled field set: seq/docs/handoff (mw-0f4j) +
    // cat/verify/title (mw-f1x71yg, §6 ruling 2026-08-10) + body/parent/
    // from/relates/to/answers (MW-L1, ruling 2026-09-20).
    let set_help = stdout_of(&meshwork(&repo).args(["set", "--help"]).assert().success());
    for flag in [
        "--seq", "--docs", "--handoff", "--cat", "--verify", "--title", "--body", "--parent",
        "--from", "--relates", "--to", "--answers",
    ] {
        assert!(set_help.contains(flag), "{flag} in set --help:\n{set_help}");
    }
    let add_help = stdout_of(&meshwork(&repo).args(["add", "--help"]).assert().success());
    for flag in ["--relates", "--to", "--answers"] {
        assert!(add_help.contains(flag), "{flag} in add --help:\n{add_help}");
    }
    // Prose fields advertise the shell-safe spellings (mw-rz4ey2h ruling).
    assert!(set_help.contains("@FILE"), "{set_help}");
    let cmt_help = stdout_of(&meshwork(&repo).args(["comment", "--help"]).assert().success());
    assert!(cmt_help.contains("@FILE"), "{cmt_help}");

    // Unbuilt verbs never pretend: honest errors, no fake output.
    init_store(&repo);
    meshwork(&repo)
        .args(["mirror", "status"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not built yet"));
    // portfolio ready/q went live at 2.2 (mw-9093); next/seq stay honest
    // stubs until 2.4 — pinned in e2e_portfolio.rs.
}

/// PLAN 1.5 / MW-D3, D5: `prime` is the ≤6KB session-start digest — ready
/// top-10, in-progress with last log line, blocked with reasons, counts —
/// measured in BYTES on the kitchen-sink corpus and on a hostile one.
#[test]
fn prime_budget() {
    let (_g, repo) = fixture_repo("alpha");
    let out = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    let bytes = out.len();
    assert!(bytes > 0 && bytes <= 6144, "budget (MW-D3): {bytes} bytes");
    assert!(out.contains("az-n33d"), "top ready task:\n{out}");
    assert!(out.contains("az-t5k1"), "in-progress present:\n{out}");
    assert!(out.contains("bisecting"), "last log line rides along:\n{out}");
    assert!(out.contains("az-b10k") && out.contains("datafusion 52"), "blocked + reason:\n{out}");
    assert!(out.contains("open") && out.contains("done"), "counts line:\n{out}");

    // Hostile store: 80 giant-titled ready tasks + 40 doing — still ≤6KB,
    // with the truncation loud.
    let (_g2, big) = git_repo("bulk");
    init_store(&big);
    let long = "very long title segment that pads the line ".repeat(4);
    for i in 0..80 {
        add_task(&big, &format!("Task {i:02} {long}"));
    }
    for i in 0..40 {
        let id = add_task(&big, &format!("Doing {i:02} {long}"));
        meshwork(&big).args(["start", &id]).assert().success();
    }
    let out = stdout_of(&meshwork(&big).arg("prime").assert().success());
    assert!(out.len() <= 6144, "hostile store: {} bytes", out.len());
    assert!(out.contains("truncated"), "truncation is explicit:\n{out}");

    // JSON parity.
    let js = stdout_of(&meshwork(&repo).args(["prime", "--json"]).assert().success());
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["verb"], "prime");
    assert!(v["data"]["counts"]["open"].as_i64().unwrap() > 0);
    assert!(v["data"]["ready"].as_array().unwrap().len() <= 10);
}

/// MW-A2: the cache is an optimization, never a dependency — deleting
/// .cache (or the whole layout reservation) is always safe.
#[test]
fn cache_delete_safe() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "Cacheless");
    std::fs::remove_dir_all(repo.join("docs/meshwork/.cache")).unwrap();
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

/// mw-getx732: the free-markdown description joins the projection as
/// `tasks.body` — the description section only (tail sections and fenced
/// tail-heading lookalikes excluded), trimmed. Parsed-and-empty is `''`;
/// invalid rows are NULL — the distinction survives into SQL. Observed
/// demand: `coalesce(body,'') LIKE` fell back to grep because the column
/// didn't exist.
#[test]
fn body_projection() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let with_body = add_id(&repo, &["add", "Task with body", "--verify", "true"]);
    let empty = add_id(&repo, &["add", "Bodyless", "--verify", "true"]);

    // Give the first task a body whose fenced block quotes a tail heading —
    // the quoted `## log` must stay body content, and the real tail must not
    // leak into the projection.
    let path = task_file(&repo, &with_body);
    let text = std::fs::read_to_string(&path).unwrap();
    let text = text.replace(
        "\n## log\n",
        "The spillway design narrative.\n\n```md\n## log\n- quoted, not real\n```\n\nTail prose.\n\n## log\n",
    );
    std::fs::write(&path, text).unwrap();

    std::fs::write(
        repo.join("docs/meshwork/zz-inv1-broken.md"),
        "---\nid: zz-inv1\ntitle: [unclosed\n---\nbody of an invalid file\n",
    )
    .unwrap();

    let js = stdout_of(
        &meshwork(&repo)
            .args([
                "q",
                "SELECT id, body FROM tasks ORDER BY id",
                "--json",
            ])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let rows = v["data"]["rows"].as_array().unwrap();
    let body_of = |id: &str| {
        rows.iter()
            .find(|r| r[0] == id)
            .unwrap_or_else(|| panic!("no row for {id}: {rows:?}"))[1]
            .clone()
    };

    let body = body_of(&with_body);
    let body = body.as_str().unwrap();
    assert!(body.contains("spillway design narrative"), "{body}");
    assert!(body.contains("- quoted, not real"), "fenced content stays: {body}");
    assert!(body.contains("Tail prose."), "{body}");
    assert!(!body.contains("created"), "real tail leaked: {body}");
    assert_eq!(body, body.trim(), "body is trimmed");

    assert_eq!(body_of(&empty), serde_json::json!(""), "parsed-and-empty is ''");
    assert_eq!(body_of("zz-inv1"), serde_json::Value::Null, "invalid is NULL");

    // The driving use case: body search joined against structured fields.
    let hits = stdout_of(
        &meshwork(&repo)
            .args([
                "q",
                "SELECT id FROM tasks WHERE body LIKE '%spillway%' AND status = 'open'",
            ])
            .assert()
            .success(),
    );
    assert!(hits.contains(&with_body), "{hits}");
}

/// mw-0ssk8dg: `WHERE parent = …` is the first thing an agent guesses
/// for hierarchy queries — `parent` projects as a tasks column (one edge
/// kind, child-points-up, so the column is well-defined; the edges row
/// stays normative), and a failing q names every queryable table so the
/// graph is discoverable from the error, not archaeology.
#[test]
fn parent_column_and_q_error_tables() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let parent = add_task(&repo, "Parent task");
    let child = add_id(
        &repo,
        &["add", "Child task", "--parent", &parent, "--verify", "true"],
    );

    let out = stdout_of(
        &meshwork(&repo)
            .args([
                "q",
                &format!("SELECT id FROM tasks WHERE parent = '{parent}'"),
            ])
            .assert()
            .success(),
    );
    assert!(out.contains(&child), "child by parent column: {out}");

    let out = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT id FROM tasks WHERE parent IS NULL"])
            .assert()
            .success(),
    );
    assert!(out.contains(&parent), "parentless row is NULL: {out}");

    let out = meshwork(&repo)
        .args(["q", "SELECT no_such_column FROM tasks"])
        .assert()
        .failure();
    let err = stderr_of(&out);
    for table in ["tasks", "edges", "labels", "comments", "log", "repos"] {
        assert!(err.contains(table), "q error must name `{table}`: {err}");
    }
}
