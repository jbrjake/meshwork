// mw-tb6gdr9: advisory work claiming rides on `start` — `claimed-by:` via
// the MW-K1 identity chain, released by close/drop/reopen, annotated in
// ready/prime, double-claims surfaced by lint (never auto-resolved).

fn file_text(repo: &Path, id: &str) -> String {
    std::fs::read_to_string(task_file(repo, id)).unwrap()
}

/// Strip init's seeded `default_author` so the chain ends empty.
fn strip_default_author(repo: &Path) {
    let cfg = repo.join("docs/meshwork/config.toml");
    let kept: String = std::fs::read_to_string(&cfg)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with("default_author"))
        .fold(String::new(), |acc, l| acc + l + "\n");
    std::fs::write(&cfg, kept).unwrap();
}

#[test]
fn claim_rides_on_start() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "claimable");

    meshwork(&repo)
        .args(["start", &id, "--as", "alice"])
        .assert()
        .success();

    let text = file_text(&repo, &id);
    assert!(text.contains("claimed-by: alice"), "{text}");
    assert!(text.contains("open→doing — claimed by alice"), "{text}");

    let show = stdout_of(&meshwork(&repo).args(["show", &id]).assert().success());
    assert!(show.contains("claimed-by: alice"), "{show}");

    // The claim is a queryable column (DESIGN §4).
    let q = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT claimed_by FROM tasks WHERE claimed_by IS NOT NULL"])
            .assert()
            .success(),
    );
    assert!(q.contains("alice"), "{q}");
}

#[test]
fn claim_identity_chain_env_then_config() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    // env beats config…
    let a = add_task(&repo, "env claimed");
    meshwork(&repo)
        .args(["start", &a])
        .env("MESHWORK_AUTHOR", "bob")
        .assert()
        .success();
    assert!(file_text(&repo, &a).contains("claimed-by: bob"));

    // …config (init seeds git user.name) is the last resort.
    let b = add_task(&repo, "config claimed");
    meshwork(&repo)
        .args(["start", &b])
        .env_remove("MESHWORK_AUTHOR")
        .assert()
        .success();
    assert!(file_text(&repo, &b).contains("claimed-by: Fixture User"));
}

#[test]
fn claim_absent_identity_starts_unclaimed() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    strip_default_author(&repo);
    let id = add_task(&repo, "nobody home");

    meshwork(&repo)
        .args(["start", &id])
        .env_remove("MESHWORK_AUTHOR")
        .assert()
        .success();

    let text = file_text(&repo, &id);
    assert!(!text.contains("claimed-by"), "{text}");
    assert!(text.contains("status: doing"), "{text}");
}

#[test]
fn claim_released_on_close_drop_reopen_kept_on_block() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    let closes = add_task(&repo, "will close");
    meshwork(&repo)
        .args(["start", &closes, "--as", "alice"])
        .assert()
        .success();
    meshwork(&repo).args(["close", &closes]).assert().success();
    assert!(!file_text(&repo, &closes).contains("claimed-by"));

    let drops = add_task(&repo, "will drop");
    meshwork(&repo)
        .args(["start", &drops, "--as", "alice"])
        .assert()
        .success();
    meshwork(&repo).args(["drop", &drops]).assert().success();
    assert!(!file_text(&repo, &drops).contains("claimed-by"));

    let reopens = add_task(&repo, "will reopen");
    meshwork(&repo)
        .args(["start", &reopens, "--as", "alice"])
        .assert()
        .success();
    meshwork(&repo).args(["reopen", &reopens]).assert().success();
    assert!(!file_text(&repo, &reopens).contains("claimed-by"));

    // blocked is still claimed work — the claim survives.
    let blocks = add_task(&repo, "will block");
    meshwork(&repo)
        .args(["start", &blocks, "--as", "alice"])
        .assert()
        .success();
    meshwork(&repo)
        .args(["block", &blocks, "--reason", "waiting on upstream"])
        .assert()
        .success();
    assert!(file_text(&repo, &blocks).contains("claimed-by: alice"));
}

#[test]
fn claim_annotated_in_prime_and_ready() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    // A claimed doing task shows its claimant in prime's weather.
    let doing = add_task(&repo, "under way");
    meshwork(&repo)
        .args(["start", &doing, "--as", "alice"])
        .assert()
        .success();
    let prime = stdout_of(&meshwork(&repo).arg("prime").assert().success());
    assert!(prime.contains("[claimed: alice]"), "{prime}");

    // A stale claim on an open task (merge artifact) is annotated in ready,
    // not hidden — advisory means visible, never a lock.
    let open = add_task(&repo, "stale claim");
    let path = task_file(&repo, &open);
    let text = std::fs::read_to_string(&path)
        .unwrap()
        .replace("status: open", "status: open\nclaimed-by: mallory");
    std::fs::write(&path, text).unwrap();
    let ready = stdout_of(&meshwork(&repo).arg("ready").assert().success());
    assert!(ready.contains("[claimed: mallory]"), "{ready}");
}

#[test]
fn claim_lint_reports_double_and_stale() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);

    // Post-merge double-claim: two claim log lines, no release between —
    // exactly what parallel worktrees produce (reported, never auto-fixed).
    std::fs::write(
        repo.join("docs/meshwork/zz-dblclm1-merged-twice.md"),
        "---\nid: zz-dblclm1\ntitle: Merged twice\nstatus: doing\nclaimed-by: alice\nverify: \"true\"\ncreated: 2026-08-06\n---\n\n## log\n- 2026-08-06 open→doing — claimed by alice\n- 2026-08-06 open→doing — claimed by bob\n",
    )
    .unwrap();

    // Stale: a claim on an open task.
    std::fs::write(
        repo.join("docs/meshwork/zz-stale01-stale-claim.md"),
        "---\nid: zz-stale01\ntitle: Stale claim\nstatus: open\nclaimed-by: mallory\nverify: \"true\"\ncreated: 2026-08-06\n---\n",
    )
    .unwrap();

    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(lint.contains("double-claim"), "{lint}");
    assert!(lint.contains("alice") && lint.contains("bob"), "{lint}");
    assert!(lint.contains("claim-stale"), "{lint}");
}
