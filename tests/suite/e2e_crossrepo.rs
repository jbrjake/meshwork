// mw-k7r5 (PLAN 2.3): single-repo commands resolve foreign `repo#id` refs
// through the registry with a DIRECT file lookup — the ID-prefixed
// filename is the index (DESIGN §5, MW-B3) — never a full portfolio load.
// Only terminal statuses (done/dropped) inject task rows: that is the one
// delta the frozen dep predicate needs, and it keeps foreign tasks out of
// listings. Everything else stays NULL → conservative blocking; an
// unregistered or absent repo resolves to nothing, reported, exit 0
// (MW-G5, §13 scenario 6). No registry anywhere = today's behavior.

/// MW-B3: a dep on a done task in another registered repo is satisfied —
/// resolved through the registry, one file read, no portfolio load.
#[test]
fn crossrepo_resolution() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");

    // Hermetic baseline (no registry): the foreign dep can't resolve,
    // az-x9b2 stays conservatively blocked.
    let plain = stdout_of(&meshwork(&alpha).arg("ready").assert().success());
    assert!(!plain.contains("az-x9b2"), "{plain}");

    // With the registry: beta#bz-c0r3 is done — dep satisfied.
    let out = stdout_of(
        &meshwork(&alpha)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .arg("ready")
            .assert()
            .success(),
    );
    assert!(out.contains("az-x9b2"), "resolves via registry: {out}");
    assert!(!out.contains("az-g4m8"), "absent gamma still blocks: {out}");
    assert!(
        !out.contains("bz-"),
        "foreign tasks never leak into single-repo listings: {out}"
    );

    // The injected row carries its real status; the edge reads resolved.
    let q = |sql: &str| {
        stdout_of(
            &meshwork(&alpha)
                .env("MESHWORK_PORTFOLIO", &portfolio)
                .args(["q", sql])
                .assert()
                .success(),
        )
    };
    assert!(
        q("SELECT status FROM tasks WHERE gid='beta#bz-c0r3'").contains("done"),
        "resolved foreign target is SQL-visible"
    );
    assert!(
        q("SELECT resolved FROM edges WHERE dst_gid='beta#bz-c0r3'").contains("true"),
        "its inbound edge counts as resolved"
    );

    // A foreign target that is merely OPEN blocks exactly as before …
    let id = add_id(
        &alpha,
        &[
            "add",
            "wait on beta retry policy",
            "--verify",
            "true",
            "--needs",
            "beta#bz-r34d",
        ],
    );
    let out = stdout_of(
        &meshwork(&alpha)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .arg("ready")
            .assert()
            .success(),
    );
    assert!(!out.contains(&id), "open foreign dep still blocks: {out}");

    // … but `why` names it with its real status, not `unresolved`.
    let why = stdout_of(
        &meshwork(&alpha)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["why", &id, "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&why).unwrap();
    let frontier = v["data"]["frontier"].as_array().unwrap();
    let entry = frontier
        .iter()
        .find(|f| f["ref"] == "beta#bz-r34d")
        .unwrap_or_else(|| panic!("beta#bz-r34d in frontier: {frontier:?}"));
    assert_eq!(entry["status"], "open", "{entry}");
    assert!(entry["unresolved"].is_null(), "resolved, not guessed: {entry}");
}

/// MW-G5 / §13 scenario 6: absent or unregistered repo → unresolved
/// edges, reported, conservatively blocking — and always exit 0.
#[test]
fn absent_repo() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");

    // Registered but absent (gamma has a registry entry, no checkout).
    let why = stdout_of(
        &meshwork(&alpha)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["why", "az-g4m8", "--json"])
            .assert()
            .success(), // reported, never an error
    );
    let v: serde_json::Value = serde_json::from_str(&why).unwrap();
    let frontier = v["data"]["frontier"].as_array().unwrap();
    assert!(
        frontier
            .iter()
            .any(|f| f["ref"] == "gamma#gm-zzz9" && f["unresolved"] == true),
        "absent repo → unresolved, reported: {frontier:?}"
    );

    // Unregistered (no registry at all): same conservative report.
    let why = stdout_of(
        &meshwork(&alpha)
            .args(["why", "az-g4m8", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&why).unwrap();
    assert!(
        v["data"]["frontier"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["unresolved"] == true),
        "{v}"
    );

    // Text mode says why it can't resolve.
    let text = stdout_of(
        &meshwork(&alpha)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["why", "az-g4m8"])
            .assert()
            .success(),
    );
    assert!(text.contains("unresolved"), "{text}");
}

/// A `docs:` link may name a doc in a registered sibling —
/// `repo#path[#anchor]`, the `needs:` spelling — resolved through the
/// registry and confined to that repo. An unregistered repo is reported,
/// never read; a bare `../` path stays refused, and lint --fix rewrites
/// it to the registered form when the registry resolves the directory.
#[test]
fn docs_crossrepo_ref() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    std::fs::write(
        alpha.join("DOC.md"),
        "# Alpha doc\n\n## 1. Landing\n\nlanding body.\n\n## 2. Other\n\nother body.\n",
    )
    .unwrap();
    let batch = "---\ntitle: Reads alpha's doc\nverify: \"true\"\ndocs:\n  \
                 - alpha#DOC.md#§-1-landing\n  - nowhere#X.md\n  - ../alpha/DOC.md#§-2-other\n---\n";
    let id = stdout_of(
        &meshwork(&beta)
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["add", "--batch", "-"])
            .write_stdin(batch)
            .assert()
            .success(),
    )
    .split_whitespace()
    .next()
    .unwrap()
    .to_string();
    let show_docs = |with_registry: bool| {
        let mut cmd = meshwork(&beta);
        if with_registry {
            cmd.env("MESHWORK_PORTFOLIO", &portfolio);
        }
        stdout_of(&cmd.args(["show", &id, "--docs"]).assert().success())
    };
    let lint = |with_registry: bool, fix: bool| {
        let mut cmd = meshwork(&beta);
        if with_registry {
            cmd.env("MESHWORK_PORTFOLIO", &portfolio);
        }
        cmd.arg("lint");
        if fix {
            cmd.arg("--fix");
        }
        stdout_of(&cmd.assert().success())
    };

    // Registered: the excerpt is the anchored section of alpha's file.
    let out = show_docs(true);
    assert!(out.contains("landing body."), "{out}");
    assert!(!out.contains("other body."), "excerpt, not the file: {out}");
    assert!(out.contains("nowhere#X.md") && out.contains("not registered"), "{out}");
    assert!(out.contains("../alpha/DOC.md") && out.contains("escapes the repo"), "{out}");
    let l = lint(true, false);
    assert!(l.contains("[doc-repo-unknown]") && l.contains("nowhere"), "{l}");
    assert!(l.contains("[path-escape]"), "{l}");
    assert!(
        !l.contains("[doc-missing]") && !l.contains("[anchor-missing]"),
        "the registered link resolves clean: {l}"
    );

    // Unregistered session: the link is unverifiable — show says so, lint
    // stays quiet on it rather than inventing a missing doc.
    let out = show_docs(false);
    assert!(out.contains("no registry"), "{out}");
    assert!(!out.contains("landing body."), "{out}");
    let l = lint(false, false);
    assert!(!l.contains("[doc-missing]") && !l.contains("[doc-repo-unknown]"), "{l}");

    // --fix rewrites the ../ spelling to the registered one, logged.
    // (the fixture's done task at the root rides along as a misplaced fix)
    let fixed = lint(true, true);
    assert!(fixed.contains("fixed 2 file(s)"), "{fixed}");
    let text = std::fs::read_to_string(task_file(&beta, &id)).unwrap();
    assert!(text.contains("  - alpha#DOC.md#§-2-other\n"), "{text}");
    assert!(!text.contains("- ../alpha"), "{text}");
    assert!(text.contains("lint --fix: docs: ../alpha/DOC.md#§-2-other → alpha#DOC.md#§-2-other"), "{text}");
    let l = lint(true, false);
    assert!(!l.contains("[path-escape]"), "{l}");
    let out = show_docs(true);
    assert!(out.contains("other body."), "the rewritten link resolves: {out}");
}
