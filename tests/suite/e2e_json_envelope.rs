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
        assert_eq!(v["meshwork"]["schema"], 2, "{args:?}: {out}");
        assert!(v["verb"].is_string(), "{args:?}: {out}");
        assert!(!v["data"].is_null(), "{args:?}: {out}");
    }
}

/// `q --help` lists every queryable table with its columns, and says
/// where `--json` puts the rows — the graph is discoverable from help,
/// not from `SELECT *` archaeology. The failing-query error points at it.
#[test]
fn q_help_lists_schema() {
    let (_g, repo) = git_repo("work");
    init_store(&repo);
    let help = stdout_of(&meshwork(&repo).args(["q", "--help"]).assert().success());
    for (table, first_col, last_col) in [
        ("tasks", "gid", "parent"),
        ("edges", "src_gid", "resolved"),
        ("labels", "gid", "label"),
        ("comments", "gid", "hash"),
        ("log", "gid", "note"),
        ("repos", "repo", "present"),
    ] {
        let line = help
            .lines()
            .find(|l| l.trim_start().starts_with(table))
            .unwrap_or_else(|| panic!("{table} listed:\n{help}"));
        assert!(line.contains(first_col) && line.contains(last_col), "{line}");
    }
    assert!(help.contains("data.rows"), "{help}");

    // The rows really do live where the help says.
    let out = stdout_of(
        &meshwork(&repo)
            .args(["q", "SELECT repo FROM repos", "--json"])
            .assert()
            .success(),
    );
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(v["data"]["rows"].is_array(), "{v}");
    assert_eq!(v["data"]["columns"][0], "repo", "{v}");

    // A failing query names the tables and points at the columns.
    let err = stderr_of(
        &meshwork(&repo)
            .args(["q", "SELECT nope FROM nowhere"])
            .assert()
            .failure(),
    );
    assert!(err.contains("queryable tables") && err.contains("q --help"), "{err}");
}
