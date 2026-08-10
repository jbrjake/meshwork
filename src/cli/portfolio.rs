//! `portfolio ready / q` (PLAN 2.2, mw-9093): the single-repo pipeline fed
//! N stores — `tables::session_for` was built for 1..N from day one, so
//! the union is a loading concern, not a second query path (MW-G1/G3).
//! Registered-but-absent repos skip + report (MW-G5): stderr in text mode
//! (stdout stays pipeable), a `skipped` list in the JSON data. `next` and
//! `seq` land with sequence.md (PLAN 2.4) and error honestly until then.

use crate::cli::query::{print_q_text, q_payload, run_query, string_rows, READY_SQL};
use crate::registry::{self, SkippedRepo};
use datafusion::prelude::SessionContext;

#[derive(clap::Args)]
pub(crate) struct PortfolioArgs {
    #[command(subcommand)]
    action: PortfolioAction,
}

#[derive(clap::Subcommand)]
enum PortfolioAction {
    /// Ready across every registered repo.
    Ready,
    /// First task by the total ordering (sequence.md → registry → per-repo).
    Next,
    /// Raw SQL over the unioned portfolio tables.
    Q {
        /// SQL over the unioned six tables.
        sql: String,
    },
    /// Renumber seq weights when gaps exhaust (§15.2).
    Seq,
}

pub(crate) fn run(args: &PortfolioArgs, json: bool) -> Result<(), String> {
    match &args.action {
        PortfolioAction::Ready => ready(json),
        PortfolioAction::Q { sql } => q(sql, json),
        PortfolioAction::Next | PortfolioAction::Seq => {
            Err("portfolio next/seq land with sequence.md (PLAN 2.4); ready and q are live".into())
        }
    }
}

/// Registry → loaded stores → one `SessionContext` over the union.
fn union_session() -> Result<(SessionContext, Vec<SkippedRepo>), String> {
    let dir = registry::portfolio_dir()?;
    let reg = registry::load(&dir)?;
    let (stores, skipped) = registry::load_stores(&reg)?;
    // No foreign injection here: every resolvable repo is already loaded
    // whole; what the union can't load, a file lookup can't reach either.
    let ctx = crate::tables::session_for(&stores, &[]).map_err(|e| e.to_string())?;
    Ok((ctx, skipped))
}

/// Text-mode skip report — stderr, so piped stdout stays clean.
fn report_skips(skipped: &[SkippedRepo]) {
    for s in skipped {
        eprintln!("portfolio: skipped {} \u{2014} {}", s.repo, s.detail);
    }
}

/// JSON skip list: stable tokens only — `detail` carries machine-local
/// paths and stays out (golden outputs must not depend on the machine).
fn skips_json(skipped: &[SkippedRepo]) -> serde_json::Value {
    serde_json::Value::Array(
        skipped
            .iter()
            .map(|s| serde_json::json!({ "repo": s.repo, "reason": s.reason }))
            .collect(),
    )
}

/// Rows a listing shows by default (MW-D2) — the shared cap.
const LISTING_CAP: usize = 20;

fn ready(json: bool) -> Result<(), String> {
    let (ctx, skipped) = union_session()?;
    // The normative §5 ready SQL with the repo column joined in — the
    // predicate is untouched (one semantics, MW-G3).
    let sql = READY_SQL.replacen("SELECT t.id", "SELECT t.repo, t.id", 1);
    let (_, batches) = run_query(&ctx, &sql)?;
    let rows = string_rows(&batches);
    let total = rows.len();
    let cap = LISTING_CAP.min(total);

    if json {
        let shown: Vec<_> = rows[..cap]
            .iter()
            .map(|r| {
                serde_json::json!({ "repo": r[0], "id": r[1], "title": r[2],
                    "claimed_by": (!r[3].is_empty()).then(|| r[3].clone()) })
            })
            .collect();
        crate::cli::emit_json(
            "portfolio ready",
            &serde_json::json!({ "total": total, "skipped": skips_json(&skipped), "rows": shown }),
        );
    } else {
        report_skips(&skipped);
        for row in &rows[..cap] {
            let claim = if row[3].is_empty() {
                String::new()
            } else {
                format!("  [claimed: {}]", row[3])
            };
            println!("{}#{}  {}{claim}", row[0], row[1], row[2]);
        }
        if total > cap {
            println!(
                "\u{2026} and {} more (portfolio q for the rest)",
                total - cap
            );
        }
        if total == 0 {
            println!("nothing ready");
        }
    }
    Ok(())
}

fn q(sql: &str, json: bool) -> Result<(), String> {
    let (ctx, skipped) = union_session()?;
    let (columns, batches) = run_query(&ctx, sql)?;
    if json {
        let mut payload = q_payload(&columns, &batches);
        payload["skipped"] = skips_json(&skipped);
        crate::cli::emit_json("portfolio q", &payload);
    } else {
        report_skips(&skipped);
        print_q_text(&columns, &batches);
    }
    Ok(())
}
