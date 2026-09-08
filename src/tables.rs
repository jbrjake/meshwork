//! Rows → Arrow `MemTable`s → `DataFusion` `SessionContext` (DESIGN §4, the
//! sahjhan pattern; MW-A2/C1). One code path for 1..N stores: single-repo
//! commands pass one store, `portfolio` passes many (MW-G3).

use crate::parse::ParsedTask;
use crate::store::RepoStore;
use datafusion::arrow::array::{ArrayRef, BooleanArray, Int64Array, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::error::Result as DfResult;
use datafusion::prelude::SessionContext;
use std::collections::BTreeSet;
use std::sync::Arc;

/// The queryable table names, registration order — the single list the
/// `q` error path enumerates (mw-0ssk8dg).
pub const TABLES: [&str; 6] = ["tasks", "edges", "labels", "comments", "log", "repos"];

/// Every queryable table with its columns in projection order — what
/// `q --help` prints, so nobody reverse-engineers the graph with
/// `SELECT *` (mw-myas0dd). Pinned to the registered Arrow schemas by
/// `schema_listing_matches_registration`.
pub const SCHEMA: [(&str, &[&str]); 6] = [
    (
        "tasks",
        &[
            "gid",
            "repo",
            "id",
            "title",
            "status",
            "category",
            "verify",
            "waived",
            "seq",
            "created",
            "blocked_reason",
            "claimed_by",
            "github",
            "addressed_to",
            "path",
            "error",
            "body",
            "handoff",
            "parent",
        ],
    ),
    ("edges", &["src_gid", "dst_gid", "kind", "resolved"]),
    ("labels", &["gid", "label"]),
    (
        "comments",
        &["gid", "ord", "date", "author", "text", "hash"],
    ),
    (
        "log",
        &["gid", "ord", "date", "from_status", "to_status", "note"],
    ),
    ("repos", &["repo", "path", "remote", "present"]),
];

#[cfg(test)]
mod schema_tests {
    use super::*;

    /// The listing is the schema: every registered table's columns, in
    /// order, are exactly what `SCHEMA` says.
    #[test]
    fn schema_listing_matches_registration() {
        let ctx = session_for(&[], &[]).unwrap();
        let rt = tokio::runtime::Runtime::new().unwrap();
        for (table, columns) in SCHEMA {
            let names: Vec<String> = rt
                .block_on(ctx.table(table))
                .unwrap()
                .schema()
                .fields()
                .iter()
                .map(|f| f.name().clone())
                .collect();
            assert_eq!(names, columns, "{table}");
        }
        assert_eq!(SCHEMA.map(|(t, _)| t), TABLES);
    }
}

/// Build a `SessionContext` with the six-table contract registered:
/// `tasks`, `edges`, `labels`, `comments`, `log`, `repos` (DESIGN §4) —
/// plus the `category_matches` UDF (MW-B4), so filtering stays plain SQL.
///
/// `foreign` (mw-k7r5): registry-resolved cross-repo targets injected as
/// thin task rows so the frozen dep predicate sees them. Callers pass
/// TERMINAL statuses only — that is the one delta the predicate needs
/// (done/dropped satisfies a dep); an injected open task would leak into
/// listings, and NULL already blocks conservatively (MW-G5).
///
/// # Errors
/// Only Arrow schema/registration failures — which would be a bug, not data.
pub fn session_for(
    stores: &[RepoStore],
    foreign: &[crate::registry::ForeignTask],
) -> DfResult<SessionContext> {
    let ctx = SessionContext::new();
    ctx.register_batch(TABLES[0], tasks_batch(stores, foreign)?)?;
    ctx.register_batch(TABLES[1], edges_batch(stores, foreign)?)?;
    ctx.register_batch(TABLES[2], labels_batch(stores)?)?;
    ctx.register_batch(TABLES[3], comments_batch(stores)?)?;
    ctx.register_batch(TABLES[4], log_batch(stores)?)?;
    ctx.register_batch(TABLES[5], repos_batch(stores)?)?;
    ctx.register_udf(category_matches_udf());
    Ok(ctx)
}

/// Whole-segment category prefix match (MW-B4): `engine/spill` matches
/// `engine/spill/compaction` and `engine/spill` itself — never
/// `engine/spillover`, never mid-path. Empty prefix matches everything.
#[must_use]
pub fn category_matches(category: &str, prefix: &str) -> bool {
    prefix.is_empty()
        || category == prefix
        || (category.len() > prefix.len()
            && category.starts_with(prefix)
            && category.as_bytes()[prefix.len()] == b'/')
}

/// `category_matches(category, prefix)` as a SQL scalar UDF.
fn category_matches_udf() -> datafusion::logical_expr::ScalarUDF {
    use datafusion::logical_expr::{create_udf, ColumnarValue, Volatility};
    let fun = std::sync::Arc::new(|args: &[ColumnarValue]| {
        let arrays = ColumnarValue::values_to_arrays(args)?;
        let cats = arrays[0]
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| {
                datafusion::error::DataFusionError::Execution(
                    "category_matches: expected utf8 arguments".into(),
                )
            })?;
        let prefixes = arrays[1]
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| {
                datafusion::error::DataFusionError::Execution(
                    "category_matches: expected utf8 arguments".into(),
                )
            })?;
        let out: BooleanArray = cats
            .iter()
            .zip(prefixes.iter())
            .map(|(cat, prefix)| match (cat, prefix) {
                (Some(cat), Some(prefix)) => Some(category_matches(cat, prefix)),
                _ => None,
            })
            .collect();
        Ok(ColumnarValue::Array(std::sync::Arc::new(out)))
    });
    create_udf(
        "category_matches",
        vec![DataType::Utf8, DataType::Utf8],
        DataType::Boolean,
        Volatility::Immutable,
        fun,
    )
}

fn utf8(nullable: bool, name: &str) -> Field {
    Field::new(name, DataType::Utf8, nullable)
}

fn tasks_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        utf8(false, "gid"),
        utf8(false, "repo"),
        utf8(false, "id"),
        utf8(true, "title"),
        utf8(false, "status"),
        utf8(true, "category"),
        utf8(true, "verify"),
        utf8(true, "waived"),
        Field::new("seq", DataType::Int64, true),
        utf8(true, "created"),
        utf8(true, "blocked_reason"),
        utf8(true, "claimed_by"),
        Field::new("github", DataType::Int64, true),
        utf8(true, "addressed_to"),
        utf8(false, "path"),
        utf8(true, "error"),
        // Appended last (mw-getx732, mw-5xdyxep, then mw-0ssk8dg) so
        // readers indexing the format-1 column order are undisturbed.
        utf8(true, "body"),
        utf8(true, "handoff"),
        utf8(true, "parent"),
    ]))
}

/// Column builders for `tasks`, one Vec per column, in schema order.
#[derive(Default)]
struct TaskCols {
    gid: Vec<String>,
    repo: Vec<String>,
    id: Vec<String>,
    title: Vec<Option<String>>,
    status: Vec<String>,
    category: Vec<Option<String>>,
    verify: Vec<Option<String>>,
    waived: Vec<Option<String>>,
    seq: Vec<Option<i64>>,
    created: Vec<Option<String>>,
    blocked_reason: Vec<Option<String>>,
    claimed_by: Vec<Option<String>>,
    github: Vec<Option<i64>>,
    addressed_to: Vec<Option<String>>,
    path: Vec<String>,
    error: Vec<Option<String>>,
    body: Vec<Option<String>>,
    handoff: Vec<Option<String>>,
    parent: Vec<Option<String>>,
}

impl TaskCols {
    fn push_valid(&mut self, store: &RepoStore, t: &crate::parse::Task) {
        self.gid.push(store.gid(&t.id));
        self.id.push(t.id.clone());
        self.title.push(Some(t.title.clone()));
        self.status.push(t.status.as_str().to_string());
        self.category.push(t.category.clone());
        self.verify.push(t.verify.clone());
        self.waived.push(t.waived.clone());
        self.seq.push(t.seq);
        self.created.push(t.created.clone());
        self.blocked_reason.push(t.blocked_reason.clone());
        self.claimed_by.push(t.claimed_by.clone());
        self.github
            .push(t.github.and_then(|n| i64::try_from(n).ok()));
        self.addressed_to.push(t.to.clone());
        self.error.push(None);
        // Parsed-and-empty is `''`; only unparsed rows are NULL.
        self.body.push(Some(t.description.clone()));
        self.handoff.push(t.handoff.clone());
        // One edge kind, child-points-up, so the column is well-defined
        // (mw-0ssk8dg); the edges row remains the normative projection.
        self.parent.push(t.parent.clone());
    }

    /// All-NULL optional columns; `error`/`status` mark the row invalid.
    fn push_invalid(&mut self, store: &RepoStore, inv: &crate::parse::Invalid) {
        self.gid.push(store.gid(&inv.id));
        self.id.push(inv.id.clone());
        self.title.push(None);
        self.status.push("invalid".to_string());
        self.error.push(Some(inv.error.clone()));
        self.push_absent_optionals();
    }

    /// Registry-resolved cross-repo target (mw-k7r5): a thin row — gid,
    /// repo, id, status, title, and the resolved file's absolute path.
    fn push_foreign(&mut self, f: &crate::registry::ForeignTask) {
        self.gid.push(f.gid.clone());
        self.repo.push(f.repo.clone());
        self.id.push(f.id.clone());
        self.title.push(f.title.clone());
        self.status.push(f.status.clone());
        self.path.push(f.path.clone());
        self.error.push(None);
        self.push_absent_optionals();
    }

    /// The columns thin/invalid rows always leave NULL.
    fn push_absent_optionals(&mut self) {
        self.category.push(None);
        self.verify.push(None);
        self.waived.push(None);
        self.seq.push(None);
        self.created.push(None);
        self.blocked_reason.push(None);
        self.claimed_by.push(None);
        self.github.push(None);
        self.addressed_to.push(None);
        self.body.push(None);
        self.handoff.push(None);
        self.parent.push(None);
    }

    fn into_columns(self) -> Vec<ArrayRef> {
        vec![
            Arc::new(StringArray::from(self.gid)),
            Arc::new(StringArray::from(self.repo)),
            Arc::new(StringArray::from(self.id)),
            Arc::new(StringArray::from(self.title)),
            Arc::new(StringArray::from(self.status)),
            Arc::new(StringArray::from(self.category)),
            Arc::new(StringArray::from(self.verify)),
            Arc::new(StringArray::from(self.waived)),
            Arc::new(Int64Array::from(self.seq)),
            Arc::new(StringArray::from(self.created)),
            Arc::new(StringArray::from(self.blocked_reason)),
            Arc::new(StringArray::from(self.claimed_by)),
            Arc::new(Int64Array::from(self.github)),
            Arc::new(StringArray::from(self.addressed_to)),
            Arc::new(StringArray::from(self.path)),
            Arc::new(StringArray::from(self.error)),
            Arc::new(StringArray::from(self.body)),
            Arc::new(StringArray::from(self.handoff)),
            Arc::new(StringArray::from(self.parent)),
        ]
    }
}

fn tasks_batch(
    stores: &[RepoStore],
    foreign: &[crate::registry::ForeignTask],
) -> DfResult<RecordBatch> {
    let mut cols = TaskCols::default();
    for store in stores {
        for entry in &store.entries {
            cols.repo.push(store.repo.clone());
            cols.path.push(format!("docs/meshwork/{}", entry.file_name));
            match &entry.parsed {
                ParsedTask::Valid(t) => cols.push_valid(store, t),
                ParsedTask::Invalid(inv) => cols.push_invalid(store, inv),
            }
        }
    }
    for f in foreign {
        cols.push_foreign(f);
    }
    Ok(RecordBatch::try_new(tasks_schema(), cols.into_columns())?)
}

/// Qualify an edge target: `repo#id` refs pass through, bare ids get the
/// declaring store's repo (DESIGN §4). Shared with the addressed join.
#[must_use]
pub fn qualify_ref(store: &RepoStore, target: &str) -> String {
    if target.contains('#') {
        target.to_string()
    } else {
        store.gid(target)
    }
}

fn edges_batch(
    stores: &[RepoStore],
    foreign: &[crate::registry::ForeignTask],
) -> DfResult<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        utf8(false, "src_gid"),
        utf8(false, "dst_gid"),
        utf8(false, "kind"),
        Field::new("resolved", DataType::Boolean, false),
    ]));

    // `resolved` = dst present in the loaded set (invalid rows count: the
    // file exists and its status blocks conservatively), plus registry-
    // resolved foreign targets (mw-k7r5) — those rows exist in `tasks` too.
    let known: BTreeSet<String> = stores
        .iter()
        .flat_map(|s| {
            s.entries.iter().map(|e| match &e.parsed {
                ParsedTask::Valid(t) => s.gid(&t.id),
                ParsedTask::Invalid(inv) => s.gid(&inv.id),
            })
        })
        .chain(foreign.iter().map(|f| f.gid.clone()))
        .collect();

    let mut src = Vec::new();
    let mut dst = Vec::new();
    let mut kind = Vec::new();
    for store in stores {
        for entry in &store.entries {
            let ParsedTask::Valid(t) = &entry.parsed else {
                continue;
            };
            let src_gid = store.gid(&t.id);
            let mut push = |target: &str, k: &str| {
                src.push(src_gid.clone());
                dst.push(qualify_ref(store, target));
                kind.push(k.to_string());
            };
            for n in &t.needs {
                push(n, "needs");
            }
            if let Some(p) = &t.parent {
                push(p, "parent"); // src = the child (DESIGN §4)
            }
            if let Some(d) = &t.discovered_from {
                push(d, "discovered-from");
            }
            for r in &t.relates {
                push(r, "relates");
            }
            if let Some(a) = &t.answers {
                push(a, "answers"); // never gates ready (mw-hfvtx0s)
            }
        }
    }
    let resolved: Vec<bool> = dst.iter().map(|d| known.contains(d)).collect();

    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(src)),
        Arc::new(StringArray::from(dst)),
        Arc::new(StringArray::from(kind)),
        Arc::new(BooleanArray::from(resolved)),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn labels_batch(stores: &[RepoStore]) -> DfResult<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![utf8(false, "gid"), utf8(false, "label")]));
    let mut gid = Vec::new();
    let mut label = Vec::new();
    for store in stores {
        for entry in &store.entries {
            if let ParsedTask::Valid(t) = &entry.parsed {
                for l in &t.labels {
                    gid.push(store.gid(&t.id));
                    label.push(l.clone());
                }
            }
        }
    }
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(gid)),
        Arc::new(StringArray::from(label)),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn comments_batch(stores: &[RepoStore]) -> DfResult<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        utf8(false, "gid"),
        Field::new("ord", DataType::Int64, false),
        utf8(false, "date"),
        utf8(false, "author"),
        utf8(false, "text"),
        utf8(false, "hash"),
    ]));
    let mut gid = Vec::new();
    let mut ord = Vec::new();
    let mut date = Vec::new();
    let mut author = Vec::new();
    let mut text = Vec::new();
    let mut hash = Vec::new();
    for store in stores {
        for entry in &store.entries {
            if let ParsedTask::Valid(t) = &entry.parsed {
                for (i, c) in t.comments.iter().enumerate() {
                    gid.push(store.gid(&t.id));
                    ord.push(i64::try_from(i).unwrap_or(i64::MAX - 1) + 1);
                    date.push(c.date.clone());
                    author.push(c.author.clone());
                    text.push(c.text.clone());
                    hash.push(c.hash());
                }
            }
        }
    }
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(gid)),
        Arc::new(Int64Array::from(ord)),
        Arc::new(StringArray::from(date)),
        Arc::new(StringArray::from(author)),
        Arc::new(StringArray::from(text)),
        Arc::new(StringArray::from(hash)),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

/// `## log` entries through the normative grammar (mw-3wnhhvp): transition
/// lines carry from/to, free text keeps them NULL — the queryable half of
/// MW-E3's durable record (blocked-duration, cycle time, activity feeds).
fn log_batch(stores: &[RepoStore]) -> DfResult<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        utf8(false, "gid"),
        Field::new("ord", DataType::Int64, false),
        utf8(true, "date"),
        utf8(true, "from_status"),
        utf8(true, "to_status"),
        utf8(true, "note"),
    ]));
    let mut gid = Vec::new();
    let mut ord = Vec::new();
    let mut date: Vec<Option<String>> = Vec::new();
    let mut from: Vec<Option<String>> = Vec::new();
    let mut to: Vec<Option<String>> = Vec::new();
    let mut note: Vec<Option<String>> = Vec::new();
    for store in stores {
        for entry in &store.entries {
            if let ParsedTask::Valid(t) = &entry.parsed {
                for (i, line) in t.log.iter().enumerate() {
                    let e = crate::parse::parse_log_line(line);
                    gid.push(store.gid(&t.id));
                    ord.push(i64::try_from(i).unwrap_or(i64::MAX - 1) + 1);
                    date.push(e.date);
                    from.push(e.from.map(|s| s.as_str().to_string()));
                    to.push(e.to.map(|s| s.as_str().to_string()));
                    note.push(e.note);
                }
            }
        }
    }
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(gid)),
        Arc::new(Int64Array::from(ord)),
        Arc::new(StringArray::from(date)),
        Arc::new(StringArray::from(from)),
        Arc::new(StringArray::from(to)),
        Arc::new(StringArray::from(note)),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

fn repos_batch(stores: &[RepoStore]) -> DfResult<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        utf8(false, "repo"),
        utf8(false, "path"),
        utf8(true, "remote"),
        Field::new("present", DataType::Boolean, false),
    ]));
    // Loaded stores are present by definition; registry-known-but-absent
    // repos join this table when the portfolio lands (MW-G2/G5, PLAN 2.x).
    let repo: Vec<String> = stores.iter().map(|s| s.repo.clone()).collect();
    let path: Vec<String> = stores
        .iter()
        .map(|s| s.root.display().to_string())
        .collect();
    let remote: Vec<Option<String>> = stores.iter().map(|_| None).collect();
    let present: Vec<bool> = stores.iter().map(|_| true).collect();
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(repo)),
        Arc::new(StringArray::from(path)),
        Arc::new(StringArray::from(remote)),
        Arc::new(BooleanArray::from(present)),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}
