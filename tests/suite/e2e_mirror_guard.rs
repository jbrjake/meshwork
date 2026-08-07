// mw-pvfrpd4: mirror is append-only and UNRETRACTABLE, so `mirror push`
// refuses off the repo's default branch — a push from a feature branch
// would publish issues/comments for state that may rebase away or never
// merge. Default branch = local `origin/HEAD` ref, zero network (MW-J6);
// indeterminate is a refusal too. `[mirror] allow_non_default = true` is
// the loud escape hatch. The guard rules BEFORE M3 builds the push path.

/// Off the default branch: refused, both branches named, M3 never reached.
/// On it: the guard passes through to the honest M3 stub error.
#[test]
fn mirror_branch_guard() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "seed"]);
    git(&repo, &["branch", "-M", "main"]);
    // origin/HEAD is a plain local symbolic ref — no remote, no network.
    git(
        &repo,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );

    git(&repo, &["checkout", "-qb", "feature-x"]);
    let assert = meshwork(&repo).args(["mirror", "push"]).assert().failure();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        err.contains("feature-x") && err.contains("main"),
        "names both branches: {err}"
    );
    assert!(!err.contains("M3"), "guard fires before the stub: {err}");

    git(&repo, &["checkout", "-q", "main"]);
    let assert = meshwork(&repo).args(["mirror", "push"]).assert().failure();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(err.contains("M3"), "on default → through to the stub: {err}");
}

/// No origin/HEAD → indeterminate default → refused with the fix named;
/// the config override lets the push proceed but stays loud in output.
#[test]
fn mirror_branch_guard_indeterminate_and_override() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-qm", "seed"]);
    git(&repo, &["checkout", "-qb", "feature-y"]);

    let assert = meshwork(&repo).args(["mirror", "push"]).assert().failure();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        err.contains("default branch") && err.contains("set-head"),
        "indeterminate refuses with the fix: {err}"
    );

    // The loud escape hatch (mw-pvfrpd4): guard skipped, override named.
    let cfg = repo.join("docs/meshwork/config.toml");
    let text = std::fs::read_to_string(&cfg).unwrap();
    std::fs::write(&cfg, format!("{text}\n[mirror]\nallow_non_default = true\n")).unwrap();

    let assert = meshwork(&repo).args(["mirror", "push"]).assert().failure();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let err = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        out.contains("allow_non_default") && out.contains("feature-y"),
        "override is loud: {out}"
    );
    assert!(err.contains("M3"), "push path reached (still the stub): {err}");
}
