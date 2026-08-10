//! `query::` — category segment-prefix + label queries (PLAN 1.3;
//! MW-B4/B5). The matcher is a library fn AND a SQL UDF registered in
//! every session — filtering stays real SQL, not a bespoke language.

use crate::common::{fixtures_root, sql_rows};
use meshwork::store::load_repo;
use meshwork::tables::{category_matches, session_for};

/// MW-B4: whole-segment prefix — `engine/spill` matches
/// `engine/spill/compaction`, never `engine/spillover`.
#[test]
fn category_segment_prefix() {
    assert!(category_matches("engine/spill", "engine/spill"));
    assert!(category_matches("engine/spill/compaction", "engine/spill"));
    assert!(category_matches("engine/spill", "engine"));
    assert!(
        !category_matches("engine/spillover", "engine/spill"),
        "the normative counterexample"
    );
    assert!(
        !category_matches("engine", "engine/spill"),
        "prefix, not subset"
    );
    assert!(
        !category_matches("other/engine/spill", "engine/spill"),
        "prefixes anchor at the root"
    );
    assert!(
        category_matches("anything/at/all", ""),
        "empty prefix matches all"
    );
}

/// MW-B5: labels are flat, many-per-task, and orthogonal to categories —
/// one label spans several category subtrees.
#[tokio::test]
async fn labels_orthogonal() {
    let store = load_repo(&fixtures_root().join("alpha")).unwrap();
    let ctx = session_for(&[store], &[]).unwrap();
    let rows = sql_rows(
        &ctx,
        "SELECT DISTINCT t.category FROM tasks t \
         JOIN labels l ON l.gid = t.gid WHERE l.label = 'perf' ORDER BY 1",
    )
    .await;
    assert!(rows.len() >= 3, "perf crosses category subtrees: {rows:?}");
}

/// PLAN 1.3 verify: the UDF inside real SQL, composed with label joins.
#[tokio::test]
async fn category_labels_sql() {
    let store = load_repo(&fixtures_root().join("alpha")).unwrap();
    let ctx = session_for(&[store], &[]).unwrap();

    let tools = sql_rows(
        &ctx,
        "SELECT id FROM tasks WHERE category_matches(category, 'tools') ORDER BY id",
    )
    .await;
    let ids: Vec<&str> = tools.iter().map(|r| r[0].as_str()).collect();
    assert_eq!(ids, ["az-a7t2", "az-cw55", "az-m6t7", "az-q2r4"]);

    let spill_perf = sql_rows(
        &ctx,
        "SELECT t.id FROM tasks t JOIN labels l ON l.gid = t.gid \
         WHERE category_matches(t.category, 'engine/spill') AND l.label = 'p0' \
         ORDER BY t.id",
    )
    .await;
    let ids: Vec<&str> = spill_perf.iter().map(|r| r[0].as_str()).collect();
    assert_eq!(ids, ["az-c0m9", "az-t5k1", "az-v4g9"]);
}
