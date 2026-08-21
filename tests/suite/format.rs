//! `format::` — FORMAT.md contract checks that aren't parser or lint
//! specifics: value constraints the config table promises (mw-a6jdf5s),
//! and the conformance corpus a third-party reader self-checks against
//! (mw-7c6svyn).

use meshwork::lint::{lint_store, Severity};
use meshwork::store::load_repo;

/// mw-a6jdf5s: ID recovery from an invalid file takes the first two
/// dash-segments of the stem, so a dashed alias (`my-repo`) silently
/// corrupts recovery. The alias is `[a-z0-9]+` — lint errors on anything
/// else; `init` refuses to write one.
#[test]
fn alias_charset() {
    assert!(meshwork::id::valid_alias("mw"));
    assert!(meshwork::id::valid_alias("sazed42"));
    for bad in ["my-repo", "MW", "", "a_b", "a.b", "é"] {
        assert!(!meshwork::id::valid_alias(bad), "accepted `{bad}`");
    }

    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"my-repo\"\n").unwrap();
    let f = lint_store(&load_repo(&dir.path().join("repo")).unwrap());
    assert!(
        f.iter().any(|f| f.severity == Severity::Error
            && f.code == "alias-charset"
            && f.message.contains("my-repo")),
        "{f:?}"
    );
}

/// The five deterministic projection tables over the conformance store,
/// column order and sort as documented in `fixtures/conformance/README.md`.
/// `error` is pinned as presence only — its text is reader-defined.
const CONFORMANCE_QUERIES: [(&str, &str); 5] = [
    (
        "tasks",
        "SELECT gid, repo, id, title, status, category, verify, waived, seq, \
         created, blocked_reason, claimed_by, github, addressed_to, path, \
         (error IS NOT NULL) AS has_error, body, handoff FROM tasks ORDER BY gid",
    ),
    (
        "edges",
        "SELECT src_gid, dst_gid, kind, resolved FROM edges \
         ORDER BY src_gid, kind, dst_gid",
    ),
    (
        "labels",
        "SELECT gid, label FROM labels ORDER BY gid, label",
    ),
    (
        "comments",
        "SELECT gid, ord, date, author, text, hash FROM comments ORDER BY gid, ord",
    ),
    (
        "log",
        "SELECT gid, ord, date, from_status, to_status, note FROM log ORDER BY gid, ord",
    ),
];

/// mw-7c6svyn: the FORMAT.md conformance corpus — a golden store plus its
/// expected projection, self-contained under `fixtures/conformance/`, so a
/// third-party reader implemented from the spec can diff itself against
/// the same bytes. Where this corpus and FORMAT.md disagree, the spec
/// wins and either the corpus or the binary has a bug.
#[tokio::test]
async fn conformance_corpus() {
    use crate::common::{fixtures_root, sql_rows};
    use meshwork::tables::session_for;

    let corpus = fixtures_root().join("conformance");
    let store = load_repo(&corpus).unwrap();
    let ctx = session_for(&[store], &[]).unwrap();

    let mut doc = serde_json::Map::new();
    for (table, sql) in CONFORMANCE_QUERIES {
        let rows = sql_rows(&ctx, sql).await;
        doc.insert(table.to_string(), serde_json::json!(rows));
    }
    let actual = serde_json::to_string_pretty(&serde_json::Value::Object(doc)).unwrap() + "\n";

    let expected_path = corpus.join("expected.json");
    if std::env::var_os("MESHWORK_BLESS").is_some() {
        std::fs::write(&expected_path, &actual).unwrap();
        eprintln!("blessed conformance expected.json — review the diff against FORMAT.md");
    }
    let expected = std::fs::read_to_string(&expected_path).unwrap_or_else(|_| {
        panic!("missing conformance expected.json; generate with MESHWORK_BLESS=1")
    });
    assert_eq!(
        expected, actual,
        "conformance drift: projection no longer matches fixtures/conformance/expected.json \
         (if FORMAT.md sides with the new output: MESHWORK_BLESS=1, then review)"
    );
}

/// mw-hmg3f3d: `blocked-reason` is required WHEN blocked — the only-if
/// half is dropped. A task that was blocked, got unblocked, and kept its
/// reason is a store a human calls fine: it parses valid and draws no
/// error (the binary never enforced the iff; the spec text was the bug —
/// a third-party writer implementing it would fail healthy stores).
#[test]
fn stale_blocked_reason_legal() {
    let dir = tempfile::tempdir().unwrap();
    let mw = dir.path().join("repo/docs/meshwork");
    std::fs::create_dir_all(&mw).unwrap();
    std::fs::write(mw.join("config.toml"), "alias = \"zz\"\n").unwrap();
    std::fs::write(
        mw.join("zz-stl1-once-blocked.md"),
        "---\nid: zz-stl1\ntitle: Once blocked\nstatus: open\nverify: \"true\"\n\
         blocked-reason: waiting on the rig — resolved since\n---\nx\n",
    )
    .unwrap();
    let store = load_repo(&dir.path().join("repo")).unwrap();
    assert!(
        store.entries.iter().any(|e| matches!(
            &e.parsed,
            meshwork::parse::ParsedTask::Valid(t) if t.id == "zz-stl1"
        )),
        "a stale blocked-reason must parse valid"
    );
    let f = lint_store(&store);
    assert!(
        !f.iter()
            .any(|x| x.severity == Severity::Error && x.subject == "zz-stl1"),
        "never an error: {f:?}"
    );

    // The spec side: FORMAT.md states required-when-blocked, not iff.
    let spec =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("FORMAT.md"))
            .unwrap();
    assert!(
        !spec.contains("iff `status: blocked`"),
        "FORMAT.md still demands the only-if half"
    );
    assert!(
        spec.contains("required non-empty when `status: blocked`"),
        "FORMAT.md lost the required-when-blocked rule"
    );
}

/// mw-5rgq9ka: one contract, one number — the `--json` envelope's
/// `meshwork.schema` IS the store format version, observed from a real
/// invocation, and FORMAT.md states the mapping (a reader otherwise
/// meets `format = 1` on disk and `schema` in output with no stated
/// relationship).
#[test]
fn version_matches_envelope() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("work");
    std::fs::create_dir_all(&repo).unwrap();
    assert!(std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .unwrap()
        .success());
    let mw = |args: &[&str]| {
        let mut cmd = assert_cmd::Command::cargo_bin("meshwork").unwrap();
        // Hermetic per the suite's rule (mw-k7r5): HOME inside the
        // tempdir so no real portfolio registry can reach the test.
        cmd.current_dir(&repo)
            .env("HOME", dir.path())
            .env("MESHWORK_TRUST", "1")
            .env_remove("MESHWORK_PORTFOLIO");
        cmd.args(args);
        cmd
    };
    mw(&["init"]).assert().success();
    let out = mw(&["lint", "--json"]).assert().success();
    let text = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let envelope: serde_json::Value =
        serde_json::from_str(text.lines().next().expect("one envelope line")).unwrap();
    assert_eq!(
        envelope["meshwork"]["schema"].as_u64(),
        Some(meshwork::store::STORE_FORMAT),
        "envelope schema and store format are the same number: {envelope}"
    );

    // The spec side of the pin: FORMAT.md must name the envelope and tie
    // its `schema` to the format version.
    let spec =
        std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("FORMAT.md"))
            .unwrap();
    assert!(
        spec.contains("envelope") && spec.contains("`schema`"),
        "FORMAT.md never states the envelope `schema` ≡ format mapping"
    );
}
