// mw-5kp033j: every --json output carries an in-band identity — per-repo
// version pinning makes cross-repo aggregation the NORMAL case, so the
// consumer can't rely on knowing which binary produced a stream. Amends
// MW-C3's "versioned with the binary" to versioned in-band.

#[test]
fn json_envelope_stamps_version_and_schema() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let id = add_task(&repo, "enveloped");

    for args in [
        vec!["ready", "--json"],
        vec!["show", &id, "--json"],
        vec!["prime", "--json"],
        vec!["q", "SELECT id FROM tasks", "--json"],
    ] {
        let out = stdout_of(&meshwork(&repo).args(&args).assert().success());
        let v: serde_json::Value = serde_json::from_str(&out)
            .unwrap_or_else(|e| panic!("{args:?}: bad json ({e}): {out}"));
        assert_eq!(
            v["meshwork"]["version"],
            env!("CARGO_PKG_VERSION"),
            "{args:?}: {out}"
        );
        assert_eq!(v["meshwork"]["schema"], 1, "{args:?}: {out}");
        assert!(v["verb"].is_string(), "{args:?}: {out}");
        assert!(!v["data"].is_null(), "{args:?}: {out}");
    }
}
