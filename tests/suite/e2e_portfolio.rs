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

/// mw-jpbv (PLAN 2.4, MW-G4): `portfolio next` = the first READY task in
/// the total ordering — sequence.md entries first (file order, non-ready
/// entries skipped), then unsequenced ready tasks by repos.toml order,
/// then per-repo seq/created. Total and deterministic; resequencing is
/// editing one file.
#[test]
fn portfolio_next_ordering() {
    let (dir, portfolio) = portfolio_fixture();

    // Sequenced: az-t5k1 (doing → skipped), bz-r34d (open+ready → NEXT),
    // az-n33d (ready, but later in the sequence).
    let assert = meshwork(dir.path())
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["portfolio", "next"])
        .assert()
        .success();
    let text = stdout_of(&assert);
    assert!(
        text.starts_with("beta#bz-r34d"),
        "first sequenced READY task, non-ready entries skipped: {text}"
    );
    crate::common::assert_golden("portfolio-next.txt", &text);

    let js = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", "next", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["repo"], "beta", "{v}");
    assert_eq!(v["data"]["id"], "bz-r34d", "{v}");
    assert_eq!(v["data"]["sequenced"], true, "{v}");

    // No sequence.md → pure fallback: repos.toml order (alpha first),
    // then per-repo seq — alpha's lowest-seq ready task wins.
    std::fs::remove_file(portfolio.join("sequence.md")).unwrap();
    let text = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", "next"])
            .assert()
            .success(),
    );
    assert!(
        text.starts_with("alpha#az-n33d"),
        "fallback: repos.toml order then per-repo seq (MW-G4): {text}"
    );

    // A sequence whose every entry is non-ready falls back too — and an
    // absent-repo entry is skipped, never an error (MW-G5).
    std::fs::write(
        portfolio.join("sequence.md"),
        "## Tranche 1\n\n- alpha#az-t5k1\n- gamma#gm-zzz9\n",
    )
    .unwrap();
    let text = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", "next"])
            .assert()
            .success(),
    );
    assert!(text.starts_with("alpha#az-n33d"), "{text}");
}

/// mw-2nmsys2: sequence.md is hand-maintained cross-repo state, so a
/// typo'd or deleted id is the dangling-edge class — surfaced by the
/// registry-aware lint pass (env-opt-in, §9), never silently skipped.
/// Three cases: resolves nowhere in a registered, present repo → the
/// `dangling-sequence` warning; repo absent from disk → the skipped-repo
/// notice's business, no finding; resolves to done/dropped → satisfied,
/// prune's business, no finding.
#[test]
fn portfolio_sequence_dangling() {
    let (dir, portfolio) = portfolio_fixture();
    std::fs::write(
        portfolio.join("sequence.md"),
        "## Tranche 1\n\n\
         - alpha#az-t5k1\n\
         - alpha#az-nope9\n\
         - gamma#gm-zzz9\n\
         - zeta#zz-1234\n\
         - beta#bz-c0r3\n",
    )
    .unwrap();
    let alpha = dir.path().join("alpha");

    // Without registry context lint stays silent — the sequence check is
    // registry work and keeps the §9 env-opt-in trigger.
    let plain = stdout_of(&meshwork(&alpha).arg("lint").assert().success());
    assert!(!plain.contains("dangling-sequence"), "{plain}");

    // A dangling entry is a warning (the overlay still functions — first
    // ready one wins), so lint still exits 0.
    let assert = meshwork(&alpha)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .arg("lint")
        .assert()
        .success();
    let out = stdout_of(&assert);
    assert!(
        out.contains("dangling-sequence") && out.contains("alpha#az-nope9"),
        "typo'd id in a registered, present repo is the finding: {out}"
    );
    assert!(
        out.contains("zeta#zz-1234") && out.contains("repos.toml"),
        "an unregistered repo name can never resolve — found, names the fix: {out}"
    );
    assert!(
        !out.contains("gm-zzz9"),
        "absent gamma is unresolvable, not dangling (MW-G5): {out}"
    );
    assert!(
        !out.contains("bz-c0r3"),
        "a done target is satisfied — prune's business, not an error: {out}"
    );
    // Specifically no dangling-sequence row — other checks (verify-shell)
    // may legitimately name this task.
    assert!(
        !out.contains("dangling-sequence] alpha#az-t5k1"),
        "a live resolving entry is coherent: {out}"
    );
}

/// mw-kkvs8zq: only done/dropped satisfies a dependency, and drop crosses
/// a trust boundary done does not — whoever drops beta#bz-r34d silently
/// unblocks every cross-repo task needing it; the needed thing never
/// happened. On drop, when portfolio context resolves (the mw-k7r5 quiet
/// chain), scan registered present repos for inbound cross-repo needs on
/// the dropped id and warn one per line on stderr; absent checkouts are
/// noted as unscanned. Advisory only: the drop always proceeds — refusal
/// would be a §6 question this task explicitly does not take.
#[test]
fn drop_inbound_cross_repo_warns() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha");
    let beta = dir.path().join("beta");
    // A live inbound need: alpha#az-n33d needs beta#bz-r34d (open).
    meshwork(&alpha)
        .args(["dep", "add", "az-n33d", "--needs", "beta#bz-r34d"])
        .assert()
        .success();

    let assert = meshwork(&beta)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["drop", "bz-r34d"])
        .assert()
        .success(); // advisory — the drop itself always proceeds
    let out = stdout_of(&assert);
    assert!(out.contains("bz-r34d open→dropped"), "{out}");
    let err = stderr_of(&assert);
    assert!(
        err.contains("alpha#az-n33d") && err.contains("beta#bz-r34d"),
        "names the task whose need a drop, not a done, just cleared: {err}"
    );
    assert!(
        err.contains("gamma") && err.contains("unscanned"),
        "an absent checkout is honestly unscanned, never guessed (MW-G5): {err}"
    );

    // A drop with no inbound cross-repo needs prints no need-warnings —
    // but the unscanned note stays: silence about gamma would read as
    // "all clear" when nothing was checked there.
    let assert = meshwork(&beta)
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["drop", "bz-s3q1"])
        .assert()
        .success();
    let err = stderr_of(&assert);
    assert!(
        !err.contains("cleared by a drop"),
        "no inbound needs, no need-warning: {err}"
    );
    assert!(err.contains("gamma") && err.contains("unscanned"), "{err}");

    // No registry context (the meshwork() harness strips it): quiet chain,
    // today's behavior — no scan, no noise.
    let (_g, plain) = git_repo("plain");
    init_store(&plain);
    let id = {
        let out = stdout_of(
            &meshwork(&plain)
                .args(["add", "loner", "--verify", "true", "--json"])
                .assert()
                .success(),
        );
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        v["data"]["id"].as_str().unwrap().to_string()
    };
    let assert = meshwork(&plain).args(["drop", &id]).assert().success();
    let err = stderr_of(&assert);
    assert!(err.is_empty(), "no registry, no scan, no noise: {err}");
}

/// mw-chcqk6g (owner-ruled 2026-08-10: no --prune flag — running any
/// portfolio verb autoprunes): satisfied sequence.md entries — done or
/// dropped in a registered, PRESENT repo — are removed on every portfolio
/// run, the clutter fix archive/ already is for task files. Headings and
/// prose survive byte-for-byte; dangling entries stay (lint's finding);
/// unresolvable entries stay (an absent checkout is not evidence of
/// death, MW-G5). Removals are reported: stderr in text mode, a `pruned`
/// list in JSON data. git diff in the portfolio repo is the review
/// surface and the undo.
#[test]
fn portfolio_sequence_prune() {
    let (dir, portfolio) = portfolio_fixture();
    std::fs::write(
        portfolio.join("sequence.md"),
        "# sequence\n\n## Tranche 1\n\n\
         - beta#bz-c0r3\n\
         - alpha#az-n33d\n\
         - alpha#az-dr0p\n\n\
         ## Tranche 2\n\n\
         - gamma#gm-zzz9\n\
         - alpha#az-nope9\n",
    )
    .unwrap();

    // Text mode: both terminal-status entries prune, reported on stderr.
    let assert = meshwork(dir.path())
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["portfolio", "ready"])
        .assert()
        .success();
    let err = stderr_of(&assert);
    assert!(
        err.contains("pruned beta#bz-c0r3") && err.contains("done"),
        "a done entry prunes and says why: {err}"
    );
    assert!(
        err.contains("pruned alpha#az-dr0p") && err.contains("dropped"),
        "a dropped entry prunes too: {err}"
    );

    // The file after: satisfied entries gone, everything else verbatim.
    let after = std::fs::read_to_string(portfolio.join("sequence.md")).unwrap();
    assert_eq!(
        after,
        "# sequence\n\n## Tranche 1\n\n\
         - alpha#az-n33d\n\n\
         ## Tranche 2\n\n\
         - gamma#gm-zzz9\n\
         - alpha#az-nope9\n",
        "headings, prose, live, absent-repo, and dangling entries survive"
    );

    // Second run: idempotent — nothing left to prune, JSON list empty.
    let js = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", "next", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["pruned"], serde_json::json!([]), "{v}");

    // JSON mode reports the prune structurally: re-add a satisfied entry.
    std::fs::write(
        portfolio.join("sequence.md"),
        "- beta#bz-c0r3\n- alpha#az-n33d\n",
    )
    .unwrap();
    let js = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", "ready", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(
        v["data"]["pruned"],
        serde_json::json!([{ "ref": "beta#bz-c0r3", "status": "done" }]),
        "{v}"
    );
}

/// Discovery: default is ~/Documents/code/portfolio (§15.4); no
/// registry anywhere is a loud error.
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

}

/// mw-908n9k2 (§15.2): `portfolio seq` — repo-level renumber when a gap
/// exhausts. Trigger: two adjacent live seq weights in one repo with no
/// integer between them (leras exhausted three neighborhoods within 48h
/// of migrating). Action: that repo's live seq-bearing tasks renumber to
/// gaps of 10 in current order (seq, created, id). Unseq'd tasks,
/// terminal tasks, and healthy repos stay byte-identical; weights that
/// already sit on their new value are not rewritten.
#[test]
fn portfolio_seq_renumber() {
    let (dir, portfolio) = portfolio_fixture();
    let alpha = dir.path().join("alpha/docs/meshwork");
    let beta = dir.path().join("beta/docs/meshwork");
    let seq_of = |p: &Path| -> Option<String> {
        std::fs::read_to_string(p)
            .unwrap()
            .lines()
            .find(|l| l.starts_with("seq:"))
            .map(str::to_string)
    };

    // Exhaust alpha's first gap: az-n33d 20 → 11, adjacent to az-s4g0's
    // 10. Alpha's live weights become 10, 11, 30, 40, 80.
    let n33d = alpha.join("az-n33d-publish-spill-report.md");
    let text = std::fs::read_to_string(&n33d).unwrap();
    std::fs::write(&n33d, text.replace("seq: 20", "seq: 11")).unwrap();
    let beta_before =
        std::fs::read_to_string(beta.join("bz-s3q1-schema-qualifier-cleanup.md")).unwrap();

    // The mutating run: alpha renumbers to 10,20,30,40,50 in current
    // order — only n33d (11→20) and r3l8 (80→50) actually move.
    let js = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", "seq", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(
        v["data"]["renumbered"],
        serde_json::json!([{ "repo": "alpha", "rewritten": 2, "total": 5 }]),
        "{v}"
    );
    assert!(
        v["data"]["skipped"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s["repo"] == "gamma"),
        "absent repo reported, never an error (MW-G5): {v}"
    );
    assert_eq!(
        seq_of(&n33d),
        Some("seq: 20".into()),
        "order preserved: n33d follows s4g0"
    );
    assert_eq!(
        seq_of(&alpha.join("az-r3l8-document-spill-knobs.md")),
        Some("seq: 50".into()),
        "the tail compacts"
    );
    for (file, kept) in [
        ("az-s4g0-governed-spill-program.md", "seq: 10"),
        ("az-cw55-cache-warmup-pass.md", "seq: 30"),
        ("az-x9b2-cross-repo-consumer-bump.md", "seq: 40"),
    ] {
        assert_eq!(seq_of(&alpha.join(file)), Some(kept.into()), "{file}");
    }
    assert_eq!(
        seq_of(&alpha.join("az-f1nd-fix-flaky-governor-test.md")),
        None,
        "unseq'd tasks never gain a weight"
    );
    assert_eq!(
        std::fs::read_to_string(beta.join("bz-s3q1-schema-qualifier-cleanup.md")).unwrap(),
        beta_before,
        "a healthy repo is byte-identical"
    );

    // Idempotent: every alpha gap is healthy now.
    let js = stdout_of(
        &meshwork(dir.path())
            .env("MESHWORK_PORTFOLIO", &portfolio)
            .args(["portfolio", "seq", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&js).unwrap();
    assert_eq!(v["data"]["renumbered"], serde_json::json!([]), "{v}");
}

/// §15.2's edges in text mode: terminal tasks keep their stale weight
/// (they order nothing), and a healthy repo stays out of the report.
#[test]
fn portfolio_seq_terminal_and_text() {
    let (dir, portfolio) = portfolio_fixture();
    let beta = dir.path().join("beta/docs/meshwork");
    let seq_of = |p: &Path| -> Option<String> {
        std::fs::read_to_string(p)
            .unwrap()
            .lines()
            .find(|l| l.starts_with("seq:"))
            .map(str::to_string)
    };

    // Exhaust beta: r34d live at 11 beside s3q1's 10; c0r3 done at 12.
    for (file, marker, insert) in [
        (
            "bz-r34d-retry-policy-fetch.md",
            "status: open\n",
            "status: open\nseq: 11\n",
        ),
        (
            "bz-c0r3-core-reader-v2.md",
            "status: done\n",
            "status: done\nseq: 12\n",
        ),
    ] {
        let p = beta.join(file);
        let text = std::fs::read_to_string(&p).unwrap();
        assert!(text.contains(marker), "{file} lost its status line");
        std::fs::write(&p, text.replacen(marker, insert, 1)).unwrap();
    }
    let assert = meshwork(dir.path())
        .env("MESHWORK_PORTFOLIO", &portfolio)
        .args(["portfolio", "seq"])
        .assert()
        .success();
    let out = stdout_of(&assert);
    assert!(
        out.contains("beta") && out.contains("1 of 2"),
        "text mode names the repo and the rewrite count: {out}"
    );
    assert!(
        !out.contains("alpha"),
        "healthy alpha stays out of the report: {out}"
    );
    assert_eq!(
        seq_of(&beta.join("bz-r34d-retry-policy-fetch.md")),
        Some("seq: 20".into())
    );
    assert_eq!(
        seq_of(&beta.join("bz-c0r3-core-reader-v2.md")),
        Some("seq: 12".into()),
        "terminal tasks keep their stale weight \u{2014} they order nothing"
    );
}
