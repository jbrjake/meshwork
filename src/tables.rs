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

/// Build a `SessionContext` with the five-table contract registered:
/// `tasks`, `edges`, `labels`, `comments`, `repos` (DESIGN §4) — plus the
/// `category_matches` UDF (MW-B4), so filtering stays plain SQL.
///
/// # Errors
/// Only Arrow schema/registration failures — which would be a bug, not data.
pub fn session_for(stores: &[RepoStore]) -> DfResult<SessionContext> {
    let ctx = SessionContext::new();
    ctx.register_batch("tasks", tasks_batch(stores)?)?;
    ctx.register_batch("edges", edges_batch(stores)?)?;
    ctx.register_batch("labels", labels_batch(stores)?)?;
    ctx.register_batch("comments", comments_batch(stores)?)?;
    ctx.register_batch("repos", repos_batch(stores)?)?;
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

fn tasks_batch(stores: &[RepoStore]) -> DfResult<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
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
        Field::new("github", DataType::Int64, true),
        utf8(false, "path"),
        utf8(true, "error"),
    ]));

    let mut gid = Vec::new();
    let mut repo = Vec::new();
    let mut id = Vec::new();
    let mut title: Vec<Option<String>> = Vec::new();
    let mut status = Vec::new();
    let mut category: Vec<Option<String>> = Vec::new();
    let mut verify: Vec<Option<String>> = Vec::new();
    let mut waived: Vec<Option<String>> = Vec::new();
    let mut seq: Vec<Option<i64>> = Vec::new();
    let mut created: Vec<Option<String>> = Vec::new();
    let mut blocked_reason: Vec<Option<String>> = Vec::new();
    let mut github: Vec<Option<i64>> = Vec::new();
    let mut path = Vec::new();
    let mut error: Vec<Option<String>> = Vec::new();

    for store in stores {
        for entry in &store.entries {
            repo.push(store.repo.clone());
            path.push(format!("docs/meshwork/{}", entry.file_name));
            match &entry.parsed {
                ParsedTask::Valid(t) => {
                    gid.push(store.gid(&t.id));
                    id.push(t.id.clone());
                    title.push(Some(t.title.clone()));
                    status.push(t.status.as_str().to_string());
                    category.push(t.category.clone());
                    verify.push(t.verify.clone());
                    waived.push(t.waived.clone());
                    seq.push(t.seq);
                    created.push(t.created.clone());
                    blocked_reason.push(t.blocked_reason.clone());
                    github.push(t.github.and_then(|n| i64::try_from(n).ok()));
                    error.push(None);
                }
                ParsedTask::Invalid(inv) => {
                    gid.push(store.gid(&inv.id));
                    id.push(inv.id.clone());
                    title.push(None);
                    status.push("invalid".to_string());
                    category.push(None);
                    verify.push(None);
                    waived.push(None);
                    seq.push(None);
                    created.push(None);
                    blocked_reason.push(None);
                    github.push(None);
                    error.push(Some(inv.error.clone()));
                }
            }
        }
    }

    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(gid)),
        Arc::new(StringArray::from(repo)),
        Arc::new(StringArray::from(id)),
        Arc::new(StringArray::from(title)),
        Arc::new(StringArray::from(status)),
        Arc::new(StringArray::from(category)),
        Arc::new(StringArray::from(verify)),
        Arc::new(StringArray::from(waived)),
        Arc::new(Int64Array::from(seq)),
        Arc::new(StringArray::from(created)),
        Arc::new(StringArray::from(blocked_reason)),
        Arc::new(Int64Array::from(github)),
        Arc::new(StringArray::from(path)),
        Arc::new(StringArray::from(error)),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

/// Qualify an edge target: `repo#id` refs pass through, bare ids get the
/// declaring store's repo (DESIGN §4).
fn qualify(store: &RepoStore, target: &str) -> String {
    if target.contains('#') {
        target.to_string()
    } else {
        store.gid(target)
    }
}

fn edges_batch(stores: &[RepoStore]) -> DfResult<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        utf8(false, "src_gid"),
        utf8(false, "dst_gid"),
        utf8(false, "kind"),
        Field::new("resolved", DataType::Boolean, false),
    ]));

    // `resolved` = dst present in the loaded set (invalid rows count: the
    // file exists and its status blocks conservatively). Registry lookups
    // for absent repos arrive with the portfolio (MW-B3/G5, PLAN 2.3).
    let known: BTreeSet<String> = stores
        .iter()
        .flat_map(|s| {
            s.entries.iter().map(|e| match &e.parsed {
                ParsedTask::Valid(t) => s.gid(&t.id),
                ParsedTask::Invalid(inv) => s.gid(&inv.id),
            })
        })
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
                dst.push(qualify(store, target));
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
    ]));
    let mut gid = Vec::new();
    let mut ord = Vec::new();
    let mut date = Vec::new();
    let mut author = Vec::new();
    let mut text = Vec::new();
    for store in stores {
        for entry in &store.entries {
            if let ParsedTask::Valid(t) = &entry.parsed {
                for (i, c) in t.comments.iter().enumerate() {
                    gid.push(store.gid(&t.id));
                    ord.push(i64::try_from(i).unwrap_or(i64::MAX - 1) + 1);
                    date.push(c.date.clone());
                    author.push(c.author.clone());
                    text.push(c.text.clone());
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
