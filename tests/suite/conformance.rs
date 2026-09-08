//! `conformance::` — the FORMAT.md §Views corpus (mw-sg8phqk): every public
//! view over the conformance store at two clock stamps, and over the linked
//! two-store union, produced by the real binary (`q --json`, `portfolio q
//! --json`), rendered by the section's portable rules, and byte-compared to
//! `fixtures/conformance/**/expected/views/<view>.json`. Regenerate with
//! `MESHWORK_BLESS=1 cargo test conformance::views_golden`, then review the
//! diff against FORMAT.md §Views — the corpus exists to catch the case where
//! the spec and the binary disagree.

use crate::common::{copy_dir, fixtures_root, git};
use assert_cmd::Command;
use std::path::{Path, PathBuf};

/// The thirteen views and the sort keys FORMAT.md §Views pins for the corpus.
const VIEWS: [(&str, &str); 13] = [
    ("clock", "now"),
    ("events", "gid, ord, kind, stamp"),
    ("spans", "gid, entered, state"),
    ("facts", "gid"),
    ("graph", "gid"),
    ("asks", "gid"),
    ("mentions", "src_gid, field, ref_gid"),
    ("lineage", "gid"),
    ("attention", "gid"),
    ("sessions", "actor, repo"),
    ("flow", "repo, day"),
    ("hazard", "repo, age_d"),
    ("pulse", "repo"),
];
/// A minute stamp, and a date-only stamp (midnight UTC) — both conforming
/// forms of `MESHWORK_TODAY`, so the clock is exercised in each.
const STAMP_A: &str = "2026-08-10T12:00Z";
const STAMP_B: &str = "2026-09-01";
const WINDOW_DAYS: i64 = 7;

fn meshwork(dir: &Path, stamp: &str) -> Command {
    let mut cmd = Command::cargo_bin("meshwork").unwrap();
    cmd.current_dir(dir)
        .env("HOME", dir)
        .env_remove("MESHWORK_PORTFOLIO")
        .env("MESHWORK_TODAY", stamp);
    cmd
}

/// FORMAT.md §Views rendering: every cell a string — text as-is, integers
/// as digits, booleans `true`/`false`, NULL empty, floats with exactly four
/// decimals. Timestamps and dates arrive from `q --json` already as text.
fn cell(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::Null => String::new(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.as_i64().map_or_else(
            || format!("{:.4}", n.as_f64().unwrap_or(0.0)),
            |i| i.to_string(),
        ),
        serde_json::Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn render(view: &str, stamp: &str, js: &str) -> String {
    let v: serde_json::Value =
        serde_json::from_str(js).unwrap_or_else(|e| panic!("q --json over `{view}`: {e}\n{js}"));
    let rows: Vec<Vec<String>> = v["data"]["rows"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|r| r.as_array().expect("row").iter().map(cell).collect())
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({
        "view": view,
        "clock": stamp,
        "window_days": WINDOW_DAYS,
        "columns": v["data"]["columns"],
        "rows": rows,
    }))
    .unwrap()
        + "\n"
}

/// Compare (or, under `MESHWORK_BLESS`, rewrite) one expected file; returns
/// the view name on drift.
fn check(expected_dir: &Path, view: &str, actual: &str) -> Option<String> {
    let path = expected_dir.join(format!("{view}.json"));
    if std::env::var_os("MESHWORK_BLESS").is_some() {
        std::fs::create_dir_all(expected_dir).unwrap();
        std::fs::write(&path, actual).unwrap();
        eprintln!(
            "blessed {} — review the diff against FORMAT.md §Views",
            path.display()
        );
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing {}; generate with MESHWORK_BLESS=1 cargo test conformance::views_golden",
            path.display()
        )
    });
    (expected != actual).then(|| view.to_string())
}

fn run_views(
    mut cmd: impl FnMut() -> Command,
    verb: &[&str],
    stamp: &str,
    expected_dir: &Path,
    label: &str,
    drift: &mut Vec<String>,
) {
    for (view, order) in VIEWS {
        let sql = format!("SELECT * FROM {view} ORDER BY {order}");
        let mut args: Vec<&str> = verb.to_vec();
        args.extend([sql.as_str(), "--json"]);
        let out = cmd().args(&args).output().unwrap();
        assert!(
            out.status.success(),
            "{label}: `{sql}` at {stamp} failed — views not registered?\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let actual = render(view, stamp, &String::from_utf8(out.stdout).unwrap());
        if let Some(v) = check(expected_dir, view, &actual) {
            drift.push(format!("{label}/{v}"));
        }
    }
}

/// A tempdir copy of a fixture store, git-initialised so the binary finds it.
fn store_copy(dir: &Path, fixture: &Path, name: &str) -> PathBuf {
    let repo = dir.join(name);
    copy_dir(fixture, &repo);
    git(&repo, &["init", "-q"]);
    repo
}

/// MW-S1/S3/S16: the derived projection over the corpus, at both conforming
/// clock forms, and over the union — the only place the cross-repo columns
/// are non-zero and therefore the only test that a reader got them right.
#[test]
fn views_golden() {
    let dir = tempfile::tempdir().unwrap();
    let corpus = fixtures_root().join("conformance");
    let repo = store_copy(dir.path(), &corpus, "conformance");
    let mut drift = Vec::new();

    for (stamp, sub) in [(STAMP_A, ""), (STAMP_B, STAMP_B)] {
        let expected = corpus.join("expected").join("views").join(sub);
        run_views(
            || meshwork(&repo, stamp),
            &["q"],
            stamp,
            &expected,
            &format!("conformance@{stamp}"),
            &mut drift,
        );
    }

    // The linked union: `left` asks `right`, needs it, and cites it.
    let linked = corpus.join("linked");
    for name in ["left", "right"] {
        store_copy(dir.path(), &linked.join(name), name);
    }
    let portfolio = dir.path().join("portfolio");
    copy_dir(&linked.join("portfolio"), &portfolio);
    std::fs::write(
        portfolio.join("repos.local.toml"),
        format!(
            "[paths]\nleft = \"{}\"\nright = \"{}\"\n",
            dir.path().join("left").display(),
            dir.path().join("right").display()
        ),
    )
    .unwrap();
    run_views(
        || {
            let mut c = meshwork(dir.path(), STAMP_A);
            c.env("MESHWORK_PORTFOLIO", &portfolio);
            c
        },
        &["portfolio", "q"],
        STAMP_A,
        &linked.join("expected").join("views"),
        "linked",
        &mut drift,
    );

    assert!(
        drift.is_empty(),
        "views drift from the blessed corpus: {drift:?} (if FORMAT.md §Views sides with the \
         new output: MESHWORK_BLESS=1 cargo test conformance::views_golden, then review the diff)"
    );
}
