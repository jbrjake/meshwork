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
         (error IS NOT NULL) AS has_error FROM tasks ORDER BY gid",
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
