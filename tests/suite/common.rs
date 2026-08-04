//! Shared helpers for the suite: fixture paths, tempdir corpus copies, and
//! SQL row extraction (values as strings for terse assertions).

use datafusion::prelude::SessionContext;
use std::path::{Path, PathBuf};

pub fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

/// Run git in `dir`, isolated from the machine's config; panics on failure.
pub fn git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .expect("git runs");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Recursive copy — tests never mutate the committed corpus (DESIGN §13).
pub fn copy_dir(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), &to).unwrap();
        }
    }
}

/// Every file under `root`, repo-relative, sorted — for no-new-files checks.
pub fn file_inventory(root: &Path) -> Vec<String> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_dir() {
                walk(&entry.path(), root, out);
            } else {
                out.push(
                    entry
                        .path()
                        .strip_prefix(root)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                );
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort();
    out
}

/// Byte-compare `actual` against a committed golden (MW-J4). Regenerate
/// ONLY via `MESHWORK_BLESS=1 cargo test …` — the `--bless` flow — followed
/// by a reviewed git diff; never silently.
pub fn assert_golden(name: &str, actual: &str) {
    let path = fixtures_root().join("golden").join(name);
    if std::env::var_os("MESHWORK_BLESS").is_some() {
        std::fs::write(&path, actual).unwrap();
        eprintln!("blessed golden {name} — review the git diff before committing");
    }
    let expected = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!("missing golden {name}; generate with MESHWORK_BLESS=1 cargo test")
    });
    assert_eq!(
        expected, actual,
        "golden mismatch: {name} (if the change is intended: MESHWORK_BLESS=1 cargo test, then review the diff)"
    );
}

/// Run SQL, return all rows as strings (nulls render empty).
pub async fn sql_rows(ctx: &SessionContext, q: &str) -> Vec<Vec<String>> {
    let batches = ctx.sql(q).await.unwrap().collect().await.unwrap();
    let mut rows = Vec::new();
    for batch in &batches {
        for row in 0..batch.num_rows() {
            rows.push(
                (0..batch.num_columns())
                    .map(|col| {
                        datafusion::arrow::util::display::array_value_to_string(
                            batch.column(col),
                            row,
                        )
                        .unwrap()
                    })
                    .collect(),
            );
        }
    }
    rows
}
