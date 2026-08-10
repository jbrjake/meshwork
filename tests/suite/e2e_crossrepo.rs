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
