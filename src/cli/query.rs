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
    /// SQL over tasks / edges / labels / comments / repos.
    sql: String,
}

/// Rows a listing shows by default (MW-D2).
const LISTING_CAP: usize = 20;

/// The normative `ready` SQL (DESIGN §5, MW-B6) minus its `LIMIT 20`: the
/// cap is applied at render time because the `… and N more` marker needs
/// the true total (MW-D2). Semantics are otherwise verbatim.
const READY_SQL: &str = "\
SELECT t.id, t.title FROM tasks t
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
/// portfolio unions later (MW-G3).
fn local_session() -> Result<SessionContext, String> {
    let root = crate::cli::require_store_root()?;
    let store = crate::store::load_repo(&root).map_err(|e| e.to_string())?;
    crate::tables::session_for(&[store]).map_err(|e| e.to_string())
}

/// Execute SQL, returning column names + batches (schema survives empty
/// results).
fn run_query(ctx: &SessionContext, sql: &str) -> Result<(Vec<String>, Vec<RecordBatch>), String> {
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
            .map(|r| serde_json::json!({ "id": r[0], "title": r[1] }))
            .collect();
        crate::cli::emit_json(
            "ready",
            &serde_json::json!({ "total": total, "rows": shown }),
        );
    } else {
        for row in &rows[..cap] {
            println!("{}  {}", row[0], row[1]);
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
        let mut rows = Vec::new();
        for batch in &batches {
            for row in 0..batch.num_rows() {
                rows.push(serde_json::Value::Array(
                    (0..batch.num_columns())
                        .map(|col| cell_to_json(batch, col, row))
                        .collect(),
                ));
            }
        }
        crate::cli::emit_json(
            "q",
            &serde_json::json!({ "columns": columns, "rows": rows }),
        );
    } else {
        println!("{}", columns.join(" | "));
        let rows = string_rows(&batches);
        let n = rows.len();
        for row in rows {
            println!("{}", row.join(" | "));
        }
        println!("({n} rows)");
    }
    Ok(())
}

fn string_rows(batches: &[RecordBatch]) -> Vec<Vec<String>> {
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
