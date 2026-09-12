//! `graph::` — the `graph` view's columns computed in Rust for the gated
//! verbs, held equal to the published SQL (FORMAT.md §Views) on every
//! fixture store. The SQL is the specification; the Rust is what `why`,
//! and later `ready`/`prime`, read without planning fifty view bodies.

use crate::common::{copy_dir, fixtures_root, sql_rows};
use meshwork::graph::{compute, GraphRow};
use meshwork::registry::ForeignTask;
use meshwork::store::{load_repo, RepoStore};
use meshwork::tables::session_for;
use meshwork::views::{register, Clock};

/// The view's columns, in FORMAT.md order.
const COLUMNS: &str = "gid, repo, id, status, seq, unlock, depth, place, inherit, needs_behind, \
                       lane, lane_size, needs_open, needs_unresolved, needs_open_foreign, \
                       needed_by_open, needed_by_foreign, live_children, discovered, ready";

/// A Rust row rendered the way the SQL rows come back: every value a
/// string, NULL as the empty string, booleans as `true`/`false`.
fn render(r: &GraphRow) -> Vec<String> {
    vec![
        r.gid.clone(),
        r.repo.clone(),
        r.id.clone(),
        r.status.clone(),
        r.seq.map_or(String::new(), |n| n.to_string()),
        r.unlock.to_string(),
        r.depth.to_string(),
        r.place.to_string(),
        r.inherit.to_string(),
        r.needs_behind.to_string(),
        r.lane.clone().unwrap_or_default(),
        r.lane_size.to_string(),
        r.needs_open.to_string(),
        r.needs_unresolved.to_string(),
        r.needs_open_foreign.to_string(),
        r.needed_by_open.to_string(),
        r.needed_by_foreign.to_string(),
        r.live_children.to_string(),
        r.discovered.to_string(),
        r.ready.to_string(),
    ]
}

async fn view_rows(stores: &[RepoStore], foreign: &[ForeignTask]) -> Vec<Vec<String>> {
    let ctx = session_for(stores, foreign).unwrap();
    // The graph view reads no clock; any conforming one registers it.
    let clock = Clock {
        now_secs: 1_786_000_000,
        source: "override",
        window_days: 7,
    };
    register(&ctx, &clock, Some("SELECT * FROM graph"))
        .await
        .unwrap();
    sql_rows(&ctx, &format!("SELECT {COLUMNS} FROM graph ORDER BY gid")).await
}

async fn assert_same(label: &str, stores: &[RepoStore], foreign: &[ForeignTask]) {
    let sql = view_rows(stores, foreign).await;
    let mut rust: Vec<Vec<String>> = compute(stores, foreign).iter().map(render).collect();
    rust.sort();
    assert!(!sql.is_empty(), "{label}: the view has rows");
    assert_eq!(rust.len(), sql.len(), "{label}: row count");
    for (r, s) in rust.iter().zip(&sql) {
        assert_eq!(r, s, "{label}: row {} (columns: {COLUMNS})", s[0]);
    }
}

fn foreign(gid: &str, status: &str) -> ForeignTask {
    let (repo, id) = gid.split_once('#').unwrap();
    ForeignTask {
        gid: gid.to_string(),
        repo: repo.to_string(),
        id: id.to_string(),
        status: status.to_string(),
        title: None,
        path: format!("/elsewhere/{id}.md"),
    }
}

/// The differential: every fixture store, the two unions, a damaged store
/// (needs cycle, parent cycle, dangling edge, unparseable file), and the
/// registry-resolved foreign row `q` injects — column by column.
#[tokio::test]
async fn graph_rust_matches_view() {
    let root = fixtures_root();
    let load = |rel: &str| load_repo(&root.join(rel)).unwrap();

    for rel in [
        "alpha",
        "beta",
        "conformance",
        "conformance/linked/left",
        "conformance/linked/right",
    ] {
        assert_same(rel, &[load(rel)], &[]).await;
    }
    assert_same("alpha+beta", &[load("alpha"), load("beta")], &[]).await;
    assert_same(
        "left+right",
        &[
            load("conformance/linked/left"),
            load("conformance/linked/right"),
        ],
        &[],
    )
    .await;

    // The damaged store, minus its duplicate-id twin: two files minting
    // one gid is store damage the SQL answers with multiplied joins and
    // the reader with whichever it keeps — lint's finding, not the view's.
    let dir = tempfile::tempdir().unwrap();
    let broken = dir.path().join("alpha-broken");
    copy_dir(&root.join("alpha-broken"), &broken);
    std::fs::remove_file(broken.join("docs/meshwork/ax-dup1-duplicate-id-second.md")).unwrap();
    let store = load_repo(&broken).unwrap();
    let rows = compute(std::slice::from_ref(&store), &[]);
    let cyc = rows.iter().find(|r| r.id == "ax-cyc1").unwrap();
    assert_eq!(cyc.depth, 64, "a cycle walks to the bound, as the SQL does");
    assert_same("alpha-broken", &[store], &[]).await;

    // Foreign rows: a resolved done target satisfies the need and counts
    // as nothing foreign-open; a resolved live one is foreign-open.
    let alpha = load("alpha");
    for status in ["done", "open"] {
        let f = [foreign("beta#bz-c0r3", status)];
        assert_same(
            &format!("alpha+foreign {status}"),
            std::slice::from_ref(&alpha),
            &f,
        )
        .await;
    }
    let rows = compute(
        std::slice::from_ref(&alpha),
        &[foreign("beta#bz-c0r3", "open")],
    );
    let x = rows.iter().find(|r| r.id == "az-x9b2").unwrap();
    assert_eq!(
        (x.needs_open, x.needs_unresolved, x.needs_open_foreign),
        (1, 0, 1)
    );
    assert!(
        !rows.iter().any(|r| r.gid == "beta#bz-c0r3"),
        "thin rows are not graph rows"
    );
}
