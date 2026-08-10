// mw-9093 (PLAN 2.2): portfolio union — the SAME pipeline as single-repo
// verbs fed N stores (MW-G1/G3), never a second code path. The payoff
// case: alpha#az-x9b2 needs beta#bz-c0r3 (done) — single-repo `ready`
// blocks it conservatively (dst unresolvable), the union resolves it.
// Registered-but-absent gamma skips + reports, never errors (MW-G5).
// Discovery: MESHWORK_PORTFOLIO overrides; default is
// ~/Documents/code/portfolio (DESIGN §15.4).

/// Copy alpha + beta into a tempdir, git-init them, and point the
/// portfolio fixture's gitignored repos.local.toml at them; gamma stays
/// registered with no checkout (the §13 scenario).
fn portfolio_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    for name in ["alpha", "beta"] {
        let repo = dir.path().join(name);
        copy_dir(&fixtures_root().join(name), &repo);
        git(&repo, &["init", "-q"]);
    }
    let portfolio = dir.path().join("portfolio");
    copy_dir(&fixtures_root().join("portfolio"), &portfolio);
    std::fs::write(
        portfolio.join("repos.local.toml"),
        format!(
            "[paths]\nalpha = \"{}\"\nbeta = \"{}\"\ngamma = \"{}\"\n",
            dir.path().join("alpha").display(),
            dir.path().join("beta").display(),
            dir.path().join("gamma").display()
        ),
    )
    .unwrap();
    (dir, portfolio)
}

/// MW-G3: `portfolio ready` over the union, goldened. Semantics pinned
/// explicitly first so the golden can't drift via a careless bless.
#[test]
fn portfolio_union_golden() {
    let (dir, portfolio) = portfolio_fixture();
    // Runs from a plain directory — portfolio verbs need no local store.
    let assert = meshwork(dir.path())
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["portfolio", "ready", "--json"])
        .assert()
        .success();
    let js = stdout_of(&assert);
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    let gids: Vec<String> = v["data"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            format!(
                "{}#{}",
                r["repo"].as_str().unwrap(),
                r["id"].as_str().unwrap()
            )
        })
        .collect();
    assert!(
        gids.contains(&"alpha#az-x9b2".to_string()),
        "cross-repo dep on a done task resolves in the union: {gids:?}"
    );
    assert!(
        !gids.contains(&"alpha#az-g4m8".to_string()),
        "absent gamma still blocks conservatively (MW-G5): {gids:?}"
    );
    assert!(
        gids.iter().any(|g| g.starts_with("beta#")),
        "both repos contribute rows: {gids:?}"
    );
    let skipped = v["data"]["skipped"].as_array().unwrap();
    assert!(
        skipped
            .iter()
            .any(|s| s["repo"] == "gamma" && s["reason"] == "no-checkout"),
        "the absent repo is reported, never an error (MW-G5): {skipped:?}"
    );
    crate::common::assert_golden("portfolio-ready.json", &js);

    // Text mode: repo-qualified ids on stdout, the skip report on stderr
    // (stdout stays pipeable).
    let assert = meshwork(dir.path())
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["portfolio", "ready"])
        .assert()
        .success();
    let text = stdout_of(&assert);
    assert!(text.contains("alpha#az-x9b2"), "{text}");
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("gamma"), "skip reported on stderr: {err}");
}

/// MW-G3/C1: `portfolio q` is raw SQL over the union — every table gains
/// the `repo` column and one query spans repos.
#[test]
fn portfolio_q_repo_column() {
    let (dir, portfolio) = portfolio_fixture();
    let out = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args([
                "portfolio",
                "q",
                "SELECT t.repo, count(*) AS n FROM tasks t GROUP BY t.repo ORDER BY t.repo",
            ])
            .assert()
            .success(),
    );
    assert!(out.contains("alpha | 33"), "{out}"); // count pinned by raw_sql_tables
    assert!(out.contains("beta | 3"), "{out}");

    // Cross-repo join in ONE statement — the reason the union exists.
    let out = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args([
                "portfolio",
                "q",
                "SELECT e.src_gid, d.status FROM edges e JOIN tasks d ON e.dst_gid = d.gid \
                 WHERE e.dst_gid = 'beta#bz-c0r3'",
            ])
            .assert()
            .success(),
    );
    assert!(
        out.contains("alpha#az-x9b2") && out.contains("done"),
        "{out}"
    );
}

/// Discovery + honesty: default is ~/Documents/code/portfolio (§15.4); no
/// registry anywhere is a loud error; next/seq stay honest stubs until 2.4.
#[test]
fn portfolio_discovery_default() {
    let (dir, portfolio) = portfolio_fixture();

    // Loud when nothing resolves — empty HOME, no env override.
    let empty_home = dir.path().join("empty-home");
    std::fs::create_dir_all(&empty_home).unwrap();
    let assert = meshwork(dir.path())
        .env_remove("MESHWORK_PORTFOLIO")
        .env("HOME", &empty_home)
        .args(["portfolio", "ready"])
        .assert()
        .code(1);
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("repos.toml"), "names the fix: {err}");

    // Default discovery: ~/Documents/code/portfolio, no env var needed.
    let home = dir.path().join("home");
    let default_dir = home.join("Documents/code/portfolio");
    std::fs::create_dir_all(default_dir.parent().unwrap()).unwrap();
    copy_dir(&portfolio, &default_dir);
    meshwork(dir.path())
        .env_remove("MESHWORK_PORTFOLIO")
        .env("HOME", &home)
        .args(["portfolio", "ready"])
        .assert()
        .success();

    // Unbuilt verbs keep erroring honestly (DESIGN §6 frozen surface).
    for verb in ["next", "seq"] {
        let assert = meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", verb])
            .assert()
            .code(1);
        let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert!(err.contains("2.4"), "{err}");
    }
}
