//! `portfolio ready / q` (PLAN 2.2, mw-9093): the single-repo pipeline fed
//! N stores — `tables::session_for` was built for 1..N from day one, so
//! the union is a loading concern, not a second query path (MW-G1/G3).
//! Registered-but-absent repos skip + report (MW-G5): stderr in text mode
//! (stdout stays pipeable), a `skipped` list in the JSON data. `next`
//! overlays sequence.md (MW-G4, mw-jpbv); `seq` renumbers a repo's live
//! weights to gaps of 10 when a gap exhausts (§15.2, mw-908n9k2).

use crate::cli::query::{print_q_text, q_payload, run_query, string_rows, READY_SQL};
use crate::registry::{self, Registry, SkippedRepo};
use crate::registry_hygiene::PrunedEntry;
use crate::store::RepoStore;
use datafusion::prelude::SessionContext;
use std::path::PathBuf;

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
        PortfolioAction::Next => next(json),
        PortfolioAction::Seq => seq(json),
    }
}

/// Everything a portfolio verb starts from: registry, loaded stores, the
/// skip report — and the autoprune already applied (mw-chcqk6g,
/// owner-ruled: running any portfolio verb prunes satisfied sequence.md
/// entries; no flag, git diff in the portfolio repo is the review
/// surface).
struct Portfolio {
    dir: PathBuf,
    reg: Registry,
    stores: Vec<RepoStore>,
    skipped: Vec<SkippedRepo>,
    pruned: Vec<PrunedEntry>,
}

fn load_portfolio() -> Result<Portfolio, String> {
    let dir = registry::portfolio_dir()?;
    let reg = registry::load(&dir)?;
    let (stores, skipped) = registry::load_stores(&reg)?;
    let pruned = crate::registry_hygiene::autoprune_sequence(&dir, &reg, &stores)?;
    Ok(Portfolio {
        dir,
        reg,
        stores,
        skipped,
        pruned,
    })
}

/// `portfolio next` (MW-G4, mw-jpbv): the first READY task in the total
/// ordering — sequence.md entries in file order (non-ready and
/// unresolvable entries skipped, MW-G5), then unsequenced ready tasks by
/// repos.toml order, then per-repo seq/created/id. Total, deterministic.
fn next(json: bool) -> Result<(), String> {
    let p = load_portfolio()?;
    // Post-prune read: the overlay `next` walks is the surviving one.
    let sequence = registry::load_sequence(&p.dir)?;
    let (reg, skipped) = (&p.reg, &p.skipped);
    let ctx = crate::tables::session_for(&p.stores, &[]).map_err(|e| e.to_string())?;
    let sql = READY_SQL.replacen("SELECT t.id", "SELECT t.repo, t.id, t.seq, t.created", 1);
    let (_, batches) = run_query(&ctx, &sql)?;
    // Columns: repo, id, seq, created, title, claimed_by.
    let rows = string_rows(&batches);

    // Sequenced pass — canonicalize each ref (rename aliases resolve) and
    // take the first that is actually ready.
    let ready_by_gid: std::collections::BTreeMap<String, &Vec<String>> = rows
        .iter()
        .map(|r| (format!("{}#{}", r[0], r[1]), r))
        .collect();
    let sequenced_pick = sequence.iter().find_map(|target| {
        let (repo_part, id_part) = target.split_once('#')?;
        let canonical = reg
            .resolve(repo_part)
            .map_or(repo_part, |(e, _)| e.name.as_str());
        ready_by_gid.get(&format!("{canonical}#{id_part}")).copied()
    });

    // Fallback — repos.toml order, then per-repo seq, created, id.
    let repo_rank: std::collections::BTreeMap<&str, usize> = reg
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.name.as_str(), i))
        .collect();
    let fallback_pick = || {
        rows.iter().min_by_key(|r| {
            (
                repo_rank.get(r[0].as_str()).copied().unwrap_or(usize::MAX),
                r[2].parse::<i64>().unwrap_or(i64::MAX),
                r[3].clone(),
                r[1].clone(),
            )
        })
    };
    let (row, sequenced) = match sequenced_pick {
        Some(row) => (Some(row), true),
        None => (fallback_pick(), false),
    };

    if json {
        let data = row.map_or_else(
            || {
                serde_json::json!({ "repo": null, "id": null, "title": null,
                "claimed_by": null, "sequenced": false })
            },
            |r| {
                serde_json::json!({ "repo": r[0], "id": r[1], "title": r[4],
                    "claimed_by": (!r[5].is_empty()).then(|| r[5].clone()),
                    "sequenced": sequenced })
            },
        );
        let mut data = data;
        data["skipped"] = skips_json(skipped);
        data["pruned"] = pruned_json(&p.pruned);
        crate::cli::emit_json("portfolio next", &data);
    } else {
        report_skips(skipped);
        report_pruned(&p.pruned);
        match row {
            Some(r) => {
                let claim = if r[5].is_empty() {
                    String::new()
                } else {
                    format!("  [claimed: {}]", r[5])
                };
                println!("{}#{}  {}{claim}", r[0], r[1], r[4]);
            }
            None => println!("nothing ready"),
        }
    }
    Ok(())
}

/// `portfolio seq` (§15.2, mw-908n9k2): repo-level renumber when a gap
/// exhausts. A repo triggers when two adjacent live seq weights have no
/// integer between them (a midpoint insert is impossible); its live
/// seq-bearing tasks then renumber to 10, 20, 30… in current order
/// (seq, created, id — the `next` fallback order, so nothing observable
/// reorders). Unseq'd and terminal tasks are untouched; a weight already
/// on its target value is not rewritten (minimal diffs, MW-I1's spirit).
fn seq(json: bool) -> Result<(), String> {
    let p = load_portfolio()?;
    let mut renumbered = Vec::new();
    for store in &p.stores {
        let mut live: Vec<(i64, String, String, &str)> = store
            .entries
            .iter()
            .filter_map(|e| match &e.parsed {
                crate::parse::ParsedTask::Valid(t)
                    if !matches!(
                        t.status,
                        crate::parse::Status::Done | crate::parse::Status::Dropped
                    ) =>
                {
                    t.seq.map(|s| {
                        (
                            s,
                            t.created.clone().unwrap_or_default(),
                            t.id.clone(),
                            e.file_name.as_str(),
                        )
                    })
                }
                _ => None,
            })
            .collect();
        live.sort();
        if !live.windows(2).any(|w| w[1].0 - w[0].0 <= 1) {
            continue;
        }
        let mut rewritten = 0usize;
        for (rank, (old, _, _, file_name)) in live.iter().enumerate() {
            let new = i64::try_from(rank + 1).map_err(|e| e.to_string())? * 10;
            if *old == new {
                continue;
            }
            let path = crate::store::tasks_dir(&store.root).join(file_name);
            let text =
                std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            let out = crate::edit::set_scalar(&text, "seq", Some(&new.to_string()))
                .map_err(|e| format!("{}: {e}", path.display()))?;
            std::fs::write(&path, out).map_err(|e| format!("{}: {e}", path.display()))?;
            rewritten += 1;
        }
        renumbered.push((store.repo.clone(), rewritten, live.len()));
    }

    if json {
        let list: Vec<_> = renumbered
            .iter()
            .map(|(repo, rewritten, total)| {
                serde_json::json!({ "repo": repo, "rewritten": rewritten, "total": total })
            })
            .collect();
        crate::cli::emit_json(
            "portfolio seq",
            &serde_json::json!({ "renumbered": list,
                "skipped": skips_json(&p.skipped), "pruned": pruned_json(&p.pruned) }),
        );
    } else {
        report_skips(&p.skipped);
        report_pruned(&p.pruned);
        for (repo, rewritten, total) in &renumbered {
            println!("{repo}: seq renumbered \u{2014} {rewritten} of {total} rewritten");
        }
        if renumbered.is_empty() {
            println!("no exhausted gaps");
        }
    }
    Ok(())
}

/// Loaded portfolio → one `SessionContext` over the union.
fn union_session() -> Result<(SessionContext, Portfolio), String> {
    let p = load_portfolio()?;
    // No foreign injection here: every resolvable repo is already loaded
    // whole; what the union can't load, a file lookup can't reach either.
    let ctx = crate::tables::session_for(&p.stores, &[]).map_err(|e| e.to_string())?;
    Ok((ctx, p))
}

/// Text-mode skip report — stderr, so piped stdout stays clean.
fn report_skips(skipped: &[SkippedRepo]) {
    for s in skipped {
        eprintln!("portfolio: skipped {} \u{2014} {}", s.repo, s.detail);
    }
}

/// Text-mode autoprune report — stderr, like the skip report: a state
/// change the operator should see, kept out of pipeable stdout.
fn report_pruned(pruned: &[PrunedEntry]) {
    for e in pruned {
        eprintln!(
            "portfolio: pruned {} ({}) from sequence.md",
            e.target, e.status
        );
    }
}

/// JSON autoprune list — mirrors the stderr report structurally.
fn pruned_json(pruned: &[PrunedEntry]) -> serde_json::Value {
    serde_json::Value::Array(
        pruned
            .iter()
            .map(|e| serde_json::json!({ "ref": e.target, "status": e.status }))
            .collect(),
    )
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
    let (ctx, p) = union_session()?;
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
            &serde_json::json!({ "total": total, "skipped": skips_json(&p.skipped),
                "pruned": pruned_json(&p.pruned), "rows": shown }),
        );
    } else {
        report_skips(&p.skipped);
        report_pruned(&p.pruned);
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
    let (ctx, p) = union_session()?;
    let (columns, batches) = run_query(&ctx, sql)?;
    if json {
        let mut payload = q_payload(&columns, &batches);
        payload["skipped"] = skips_json(&p.skipped);
        payload["pruned"] = pruned_json(&p.pruned);
        crate::cli::emit_json("portfolio q", &payload);
    } else {
        report_skips(&p.skipped);
        report_pruned(&p.pruned);
        print_q_text(&columns, &batches);
    }
    Ok(())
}
