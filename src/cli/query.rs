//! `ready` + `q` (PLAN 0.8): canned verbs are frozen SQL over the in-memory
//! tables (DESIGN §5 — the `ready` text is normative for MW-B6); `q` is the
//! same session, raw (MW-C1). Listings cap at 20 with an explicit marker
//! (MW-D2).

use datafusion::arrow::array::{Array, BooleanArray, Float64Array, Int64Array, StringArray};
use datafusion::arrow::datatypes::DataType;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::util::display::array_value_to_string;
use datafusion::prelude::SessionContext;

#[derive(clap::Args)]
pub(crate) struct ReadyArgs {
    /// Show every ready task instead of the first 20 (MW-D2).
    #[arg(long)]
    all: bool,
}

#[derive(clap::Args)]
pub(crate) struct QArgs {
    /// SQL over tasks / edges / labels / comments / log / repos.
    sql: String,
}

/// Rows a listing shows by default (MW-D2).
const LISTING_CAP: usize = 20;

/// The normative `ready` SQL (DESIGN §5, MW-B6) minus its `LIMIT 20`: the
/// cap is applied at render time because the `… and N more` marker needs
/// the true total (MW-D2). Semantics are otherwise verbatim.
pub(crate) const READY_SQL: &str = "\
SELECT t.id, t.title, t.claimed_by, t.verify FROM tasks t
WHERE t.status = 'open'
  AND NOT EXISTS (
    SELECT 1 FROM edges e
    LEFT JOIN tasks d ON e.dst_gid = d.gid
    WHERE e.src_gid = t.gid AND e.kind = 'needs'
      AND (d.status IS NULL OR d.status NOT IN ('done','dropped')))
  AND NOT EXISTS (
    SELECT 1 FROM edges c JOIN tasks ch ON c.src_gid = ch.gid
    WHERE c.dst_gid = t.gid AND c.kind = 'parent'
      AND ch.status IN ('open','doing','blocked'))
ORDER BY coalesce(t.seq, 999999), t.created";

/// Session over the current repo's store — the same single pipeline the
/// portfolio unions (MW-G3). Cross-repo `needs` targets resolve through
/// the registry by direct file lookup (mw-k7r5); only TERMINAL statuses
/// inject (a done/dropped dep is satisfied — the one delta the frozen
/// predicate needs; anything else already blocks conservatively as NULL,
/// and an injected open task would leak into listings).
fn local_session() -> Result<SessionContext, String> {
    let root = crate::cli::require_store_root()?;
    let store = crate::store::load_repo(&root).map_err(|e| e.to_string())?;
    let refs = crate::registry::foreign_refs(&[&store]);
    let mut foreign = Vec::new();
    if !refs.is_empty() {
        if let Some(registry) = crate::registry::quiet_load()? {
            let loaded = std::iter::once(store.repo.as_str()).collect();
            foreign = crate::registry::resolve_foreign(&registry, &refs, &loaded)
                .into_iter()
                .filter(|f| matches!(f.status.as_str(), "done" | "dropped"))
                .collect();
        }
    }
    crate::tables::session_for(&[store], &foreign).map_err(|e| e.to_string())
}

/// Execute SQL, returning column names + batches (schema survives empty
/// results). Shared with `portfolio` — same pipeline, N stores (MW-G3).
pub(crate) fn run_query(
    ctx: &SessionContext,
    sql: &str,
) -> Result<(Vec<String>, Vec<RecordBatch>), String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(async {
        let df = ctx.sql(sql).await?;
        let columns = df
            .schema()
            .fields()
            .iter()
            .map(|f| f.name().clone())
            .collect();
        let batches = df.collect().await?;
        Ok((columns, batches))
    })
    .map_err(|e: datafusion::error::DataFusionError| e.to_string())
}

/// Run SQL over the local store, rows as strings — for sibling verbs
/// (`blocked`, later `prime`) that share the canned-SQL pipeline.
pub(crate) fn sql_rows_local(sql: &str) -> Result<Vec<Vec<String>>, String> {
    let ctx = local_session()?;
    let (_, batches) = run_query(&ctx, sql)?;
    Ok(string_rows(&batches))
}

pub(crate) fn ready(args: &ReadyArgs, json: bool) -> Result<(), String> {
    let ctx = local_session()?;
    let (_, batches) = run_query(&ctx, READY_SQL)?;
    let rows = string_rows(&batches);
    let total = rows.len();
    let cap = if args.all {
        total
    } else {
        LISTING_CAP.min(total)
    };

    if json {
        let shown: Vec<_> = rows[..cap]
            .iter()
            .map(|r| {
                serde_json::json!({ "id": r[0], "title": r[1],
                    "claimed_by": (!r[2].is_empty()).then(|| r[2].clone()),
                    "needs_verify": r[3].is_empty() })
            })
            .collect();
        crate::cli::emit_json(
            "ready",
            &serde_json::json!({ "total": total, "rows": shown }),
        );
    } else {
        for row in &rows[..cap] {
            // An open task carrying a claim is a merge artifact — annotate,
            // never hide: claims are advisory (mw-tb6gdr9).
            let claim = if row[2].is_empty() {
                String::new()
            } else {
                format!("  [claimed: {}]", row[2])
            };
            // Verify-less capture stays visible AND loud (mw-6wdpz1b):
            // writing the done-test is the task's next action.
            let gap = if row[3].is_empty() {
                "  [needs-verify]"
            } else {
                ""
            };
            println!("{}  {}{claim}{gap}", row[0], row[1]);
        }
        if total > cap {
            println!("… and {} more (use --all)", total - cap);
        }
        if total == 0 {
            println!("nothing ready");
        }
    }
    Ok(())
}

pub(crate) fn q(args: &QArgs, json: bool) -> Result<(), String> {
    let ctx = local_session()?;
    let (columns, batches) = run_query(&ctx, &args.sql)?;

    if json {
        crate::cli::emit_json("q", &q_payload(&columns, &batches));
    } else {
        print_q_text(&columns, &batches);
    }
    Ok(())
}

/// The `q` JSON data shape — `{columns, rows}` with typed cells (MW-C3).
pub(crate) fn q_payload(columns: &[String], batches: &[RecordBatch]) -> serde_json::Value {
    let mut rows = Vec::new();
    for batch in batches {
        for row in 0..batch.num_rows() {
            rows.push(serde_json::Value::Array(
                (0..batch.num_columns())
                    .map(|col| cell_to_json(batch, col, row))
                    .collect(),
            ));
        }
    }
    serde_json::json!({ "columns": columns, "rows": rows })
}

/// The `q` text rendering: pipe-joined header, rows, count.
pub(crate) fn print_q_text(columns: &[String], batches: &[RecordBatch]) {
    println!("{}", columns.join(" | "));
    let rows = string_rows(batches);
    let n = rows.len();
    for row in rows {
        println!("{}", row.join(" | "));
    }
    println!("({n} rows)");
}

pub(crate) fn string_rows(batches: &[RecordBatch]) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for batch in batches {
        for row in 0..batch.num_rows() {
            rows.push(
                (0..batch.num_columns())
                    .map(|col| array_value_to_string(batch.column(col), row).unwrap_or_default())
                    .collect(),
            );
        }
    }
    rows
}

/// Typed JSON cell: strings, integers, floats, and bools survive as their
/// JSON types (MW-C3's stable shape); anything exotic renders as a string.
fn cell_to_json(batch: &RecordBatch, col: usize, row: usize) -> serde_json::Value {
    let array = batch.column(col);
    if array.is_null(row) {
        return serde_json::Value::Null;
    }
    match array.data_type() {
        DataType::Utf8 => array
            .as_any()
            .downcast_ref::<StringArray>()
            .map_or(serde_json::Value::Null, |a| a.value(row).into()),
        DataType::Int64 => array
            .as_any()
            .downcast_ref::<Int64Array>()
            .map_or(serde_json::Value::Null, |a| a.value(row).into()),
        DataType::Boolean => array
            .as_any()
            .downcast_ref::<BooleanArray>()
            .map_or(serde_json::Value::Null, |a| a.value(row).into()),
        DataType::Float64 => array
            .as_any()
            .downcast_ref::<Float64Array>()
            .map_or(serde_json::Value::Null, |a| a.value(row).into()),
        _ => array_value_to_string(array, row).unwrap_or_default().into(),
    }
}
