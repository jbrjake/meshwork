// mw-2pz0zqc (DESIGN §12b adjacent): `docs:` links and attachment paths
// are attacker-supplied strings from merged task files, joined onto the
// repo root at read time. Confinement refuses absolute paths, `..`
// traversal, and symlink escapes before any read — loudly, running
// nothing. MW-A3's "never reads/writes outside the repo" made mechanical.

/// A hostile fixture cannot read outside the repo through any confined
/// surface: absolute and traversing docs: links, a traversing
/// attachment, and (unix) a symlink pointing out of the repo.
#[test]
fn path_confinement() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    // The secret lives OUTSIDE the repo, beside it in the tempdir.
    let secret = repo.parent().unwrap().join("secret.md");
    std::fs::write(&secret, "TOPSECRET payload\n").unwrap();

    // Absolute docs: link — refused, loud, view survives.
    let abs = add_task(&repo, "Absolute link");
    meshwork(&repo)
        .args(["set", &abs, "--docs", secret.to_str().unwrap()])
        .assert()
        .success();
    let out = stdout_of(
        &meshwork(&repo)
            .args(["show", &abs, "--docs"])
            .assert()
            .success(),
    );
    assert!(!out.contains("TOPSECRET"), "absolute path read: {out}");
    assert!(out.contains("escapes the repo"), "loud refusal: {out}");

    // Traversing docs: link — same refusal.
    let dots = add_task(&repo, "Traversing link");
    meshwork(&repo)
        .args(["set", &dots, "--docs", "../secret.md"])
        .assert()
        .success();
    let out = stdout_of(
        &meshwork(&repo)
            .args(["show", &dots, "--docs"])
            .assert()
            .success(),
    );
    assert!(!out.contains("TOPSECRET"), "traversal read: {out}");
    assert!(out.contains("escapes the repo"), "loud refusal: {out}");

    // Symlink inside the repo pointing outside — lexically clean, still
    // refused by the canonicalize comparison.
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&secret, repo.join("leak.md")).unwrap();
        let link = add_task(&repo, "Symlink escape");
        meshwork(&repo)
            .args(["set", &link, "--docs", "leak.md"])
            .assert()
            .success();
        let out = stdout_of(
            &meshwork(&repo)
                .args(["show", &link, "--docs"])
                .assert()
                .success(),
        );
        assert!(!out.contains("TOPSECRET"), "symlink read: {out}");
        assert!(out.contains("escapes the repo"), "loud refusal: {out}");
    }

    // A traversing attachment: lint warns path-escape and never stats
    // the target; the docs: escapes above warn the same way.
    let att = add_task(&repo, "Traversing attachment");
    let path = task_file(&repo, &att);
    let text = std::fs::read_to_string(&path).unwrap();
    let text = text.replace(
        "status: open",
        "status: open\nattachments:\n  - ../../secret.md",
    );
    std::fs::write(&path, text).unwrap();
    let lint = stdout_of(&meshwork(&repo).arg("lint").assert().success());
    assert!(lint.contains("path-escape"), "lint warns: {lint}");
    assert!(
        !lint.contains("attachment-missing")
            || !lint.contains("secret"),
        "escape is not misreported as merely missing: {lint}"
    );

    // The verify DSL shares the confinement: a symlink escape refuses at
    // the predicate, not just lexically.
    #[cfg(unix)]
    {
        let v = add_id(
            &repo,
            &["add", "Symlink verify", "--verify", "contains leak.md /TOPSECRET/"],
        );
        let out = meshwork(&repo).args(["close", &v]).assert().failure();
        let err = stderr_of(&out);
        assert!(err.contains("unsafe path"), "verify refusal: {err}");
    }
}
