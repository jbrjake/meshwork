//! `tables::` — ingestion → Arrow `MemTables` → `DataFusion` (PLAN 0.3;
//! MW-A2/B1/C1/I2). One code path for 1..N repos (MW-G3).

use crate::common::{copy_dir, file_inventory, fixtures_root, sql_rows};
use meshwork::store::load_repo;
use meshwork::tables::session_for;

fn session(repos: &[&str]) -> datafusion::prelude::SessionContext {
    let stores: Vec<_> = repos
        .iter()
        .map(|r| load_repo(&fixtures_root().join(r)).unwrap())
        .collect();
    session_for(&stores).unwrap()
}

/// MW-A2: queries run purely in memory — loading and querying a store
/// creates not a single file anywhere in the repo tree.
#[tokio::test]
async fn memtable_no_disk() {
    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("alpha");
    copy_dir(&fixtures_root().join("alpha"), &repo);
    let before = file_inventory(&repo);

    let store = load_repo(&repo).unwrap();
    let ctx = session_for(&[store]).unwrap();
    let rows = sql_rows(&ctx, "SELECT count(*) FROM tasks").await;
    assert_eq!(rows[0][0], "33");

    assert_eq!(before, file_inventory(&repo), "no files created or removed");
}

/// MW-B1: all four edge kinds ingest; parent edges are stored child→parent.
#[tokio::test]
async fn edge_kinds() {
    let ctx = session(&["alpha"]);
    let kinds = sql_rows(&ctx, "SELECT DISTINCT kind FROM edges ORDER BY kind").await;
    let kinds: Vec<&str> = kinds.iter().map(|r| r[0].as_str()).collect();
    assert_eq!(kinds, ["discovered-from", "needs", "parent", "relates"]);

    let parent = sql_rows(
        &ctx,
        "SELECT dst_gid FROM edges WHERE kind='parent' AND src_gid='alpha#az-e9p2'",
    )
    .await;
    assert_eq!(parent, [["alpha#az-s4g0"]], "src is the child (DESIGN §4)");
}

/// The six-table SQL contract, including the waived column (MW-E2's
/// "queryable" made true), the log table (mw-3wnhhvp), and repos.present.
#[tokio::test]
async fn six_tables_queryable() {
    let ctx = session(&["alpha"]);
    let waived = sql_rows(&ctx, "SELECT id FROM tasks WHERE waived IS NOT NULL").await;
    assert_eq!(waived, [["az-w4v3"]]);

    let labels = sql_rows(
        &ctx,
        "SELECT label FROM labels WHERE gid='alpha#az-t5k1' ORDER BY label",
    )
    .await;
    assert_eq!(labels, [["p0"], ["perf"]]);

    let repos = sql_rows(&ctx, "SELECT repo, present FROM repos").await;
    assert_eq!(repos, [["alpha", "true"]]);

    let comments = sql_rows(&ctx, "SELECT count(*) FROM comments").await;
    assert!(comments[0][0].parse::<i64>().unwrap() >= 5);

    // log: the grammar's two shapes side by side — a dated transition with
    // note, and free text with NULL from/to (mw-3wnhhvp).
    let log = sql_rows(
        &ctx,
        "SELECT date, from_status, to_status, note FROM log \
         WHERE gid IN ('alpha#az-b10k','alpha#az-s4g0') ORDER BY gid",
    )
    .await;
    assert_eq!(
        log,
        [
            ["2026-08-02", "open", "blocked", "upstream release pending"],
            ["2026-08-01", "", "", "created"],
        ]
    );
}

/// `edges.resolved` derives from the loaded set: same-repo hits resolve,
/// absent repos don't — and loading beta alongside flips its edge to
/// resolved. Conservative by rule (MW-G5).
#[tokio::test]
async fn resolved_derivation() {
    let ctx = session(&["alpha"]);
    let rows = sql_rows(
        &ctx,
        "SELECT dst_gid, resolved FROM edges WHERE src_gid IN \
         ('alpha#az-n33d','alpha#az-x9b2','alpha#az-g4m8') AND kind='needs' ORDER BY dst_gid",
    )
    .await;
    assert_eq!(
        rows,
        [
            ["alpha#az-d0n3", "true"],
            ["beta#bz-c0r3", "false"],
            ["gamma#gm-zzz9", "false"],
        ]
    );

    let ctx = session(&["alpha", "beta"]);
    let rows = sql_rows(
        &ctx,
        "SELECT resolved FROM edges WHERE dst_gid='beta#bz-c0r3' AND kind='needs'",
    )
    .await;
    assert_eq!(rows, [["true"]], "loading beta resolves the edge");
}

/// MW-I2: unparseable files surface as status='invalid' rows with error
/// text — in the same tasks table every listing reads.
#[tokio::test]
async fn invalid_rows_visible() {
    let ctx = session(&["alpha-broken"]);
    let rows = sql_rows(
        &ctx,
        "SELECT id, error FROM tasks WHERE status='invalid' ORDER BY id",
    )
    .await;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0][0], "ax-brk9");
    assert!(!rows[0][1].is_empty());
    assert_eq!(rows[1][0], "ax-un10");
    assert!(rows[1][1].contains("duplicate"));
}

/// Comments carry file-position ord (1-based) per task.
#[tokio::test]
async fn comments_ord_by_file_position() {
    let ctx = session(&["alpha"]);
    let rows = sql_rows(
        &ctx,
        "SELECT ord, author FROM comments WHERE gid='alpha#az-c0m9' ORDER BY ord",
    )
    .await;
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0], ["1", "jon"]);
    assert_eq!(rows[3], ["4", "claude/b2277c19"]);
}

/// MW-G3: the portfolio is the same pipeline over N stores — the repo
/// column appears in every table and gids stay disambiguated.
#[tokio::test]
async fn union_two_repos_one_code_path() {
    let ctx = session(&["alpha", "beta"]);
    let repos = sql_rows(&ctx, "SELECT DISTINCT repo FROM tasks ORDER BY repo").await;
    assert_eq!(repos, [["alpha"], ["beta"]]);
    let count = sql_rows(&ctx, "SELECT count(*) FROM tasks WHERE repo='beta'").await;
    assert_eq!(count[0][0], "3");
}
