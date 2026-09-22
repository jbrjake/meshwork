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
    /// Renumber seq weights when gaps exhaust.
    Seq,
    /// The derived projection as tables over the union, one pulse row per repo.
    Stats(super::stats::StatsArgs),
    /// The same literal text search as `search`, across every registered store.
    Search(super::search::SearchArgs),
}

pub(crate) fn run(args: &PortfolioArgs, json: bool) -> Result<(), String> {
    match &args.action {
        PortfolioAction::Ready => ready(json),
        PortfolioAction::Q { sql } => q(sql, json),
        PortfolioAction::Next => next(json),
        PortfolioAction::Seq => seq(json),
        PortfolioAction::Stats(stats_args) => stats(stats_args, json),
        PortfolioAction::Search(search_args) => search(search_args, json),
    }
}

/// `portfolio search` — `search`'s canned SQL over the union, hits grouped
/// by repo and named `repo#id`. A pure read (MW-S14): sequence.md is
/// never touched.
fn search(args: &super::search::SearchArgs, json: bool) -> Result<(), String> {
    let (ctx, p) = union_session(false)?;
    let hits = super::search::hits(&ctx, &args.term, true)?;
    if !json {
        report_skips(&p.skipped);
    }
    super::search::emit(
        &hits,
        args.all,
        json,
        "portfolio search",
        true,
        Some(("skipped", skips_json(&p.skipped))),
    );
    Ok(())
}

/// `portfolio stats` — a pure read like `q` (MW-S14): sequence.md is never
/// touched. The union has no config of its own, so the window is the flag
/// or the default; the inbound half of the asks counts is real here and
/// only here, because only the union sees every store.
fn stats(args: &super::stats::StatsArgs, json: bool) -> Result<(), String> {
    let (ctx, p) = union_session(false)?;
    let window_days = args.window_days(crate::views::DEFAULT_WINDOW_DAYS)?;
    let report = super::stats::collect(&ctx, window_days, true)?;
    if !json {
        report_skips(&p.skipped);
    }
    super::stats::emit(
        &report,
        "portfolio stats",
        json,
        Some(("skipped", skips_json(&p.skipped))),
    );
    Ok(())
}

/// Everything a portfolio verb starts from: registry, loaded stores, the
/// skip report — and, for the verbs that consume the overlay, the
/// autoprune already applied (mw-chcqk6g, owner-ruled: `ready`, `next`
/// and `seq` prune satisfied sequence.md entries; no flag, git diff in
/// the portfolio repo is the review surface). `q` is a pure read
/// (MW-S14): it never touches sequence.md, so its `pruned` is always
/// empty — kept on the JSON envelope for shape stability (MW-C3).
struct Portfolio {
    dir: PathBuf,
    reg: Registry,
    stores: Vec<RepoStore>,
    skipped: Vec<SkippedRepo>,
    pruned: Vec<PrunedEntry>,
}

/// Load the registry and every resolvable store; `prune` says whether
/// this verb consumes the overlay and may rewrite sequence.md.
fn load_portfolio(prune: bool) -> Result<Portfolio, String> {
    let dir = registry::portfolio_dir()?;
    let reg = registry::load(&dir)?;
    let (stores, skipped) = registry::load_stores(&reg)?;
    let pruned = if prune {
        crate::registry_hygiene::autoprune_sequence(&dir, &reg, &stores)?
    } else {
        Vec::new()
    };
    Ok(Portfolio {
        dir,
        reg,
        stores,
        skipped,
        pruned,
    })
}

/// The expanded ready SELECT both ordering-aware verbs run.
/// Columns: repo, id, seq, created, title, `claimed_by`, verify.
fn ordered_ready_sql() -> String {
    READY_SQL.replacen("SELECT t.id", "SELECT t.repo, t.id, t.seq, t.created", 1)
}

/// Sort expanded ready rows into the MW-G4 total ordering — sequence.md
/// entries in file order (non-ready and unresolvable entries skipped,
/// MW-G5), then unsequenced rows by repos.toml order, then per-repo
/// seq/created/id. `next` is row 0; `ready` presents the whole ordering —
/// sharing this sort is what keeps the two verbs from ever disagreeing
/// (mw-0vw7nj0). Returns whether row 0 came from sequence.md.
fn sort_total(rows: &mut [Vec<String>], sequence: &[String], reg: &Registry) -> bool {
    // Canonicalize each sequence ref (rename aliases resolve); first
    // occurrence wins.
    let mut seq_idx: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for (i, target) in sequence.iter().enumerate() {
        if let Some((repo_part, id_part)) = target.split_once('#') {
            let canonical = reg
                .resolve(repo_part)
                .map_or(repo_part, |(e, _)| e.name.as_str());
            seq_idx.entry(format!("{canonical}#{id_part}")).or_insert(i);
        }
    }
    let repo_rank: std::collections::BTreeMap<&str, usize> = reg
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.name.as_str(), i))
        .collect();
    rows.sort_by_key(|r| {
        (
            seq_idx
                .get(&format!("{}#{}", r[0], r[1]))
                .copied()
                .unwrap_or(usize::MAX),
            repo_rank.get(r[0].as_str()).copied().unwrap_or(usize::MAX),
            r[2].parse::<i64>().unwrap_or(i64::MAX),
            r[3].clone(),
            r[1].clone(),
        )
    });
    rows.first()
        .is_some_and(|r| seq_idx.contains_key(&format!("{}#{}", r[0], r[1])))
}

/// `portfolio next` (MW-G4, mw-jpbv): the first READY task in the total
/// ordering. Total, deterministic — row 0 of the shared sort.
fn next(json: bool) -> Result<(), String> {
    let p = load_portfolio(true)?;
    // Post-prune read: the overlay `next` walks is the surviving one.
    let sequence = registry::load_sequence(&p.dir)?;
    let skipped = &p.skipped;
    let ctx = crate::tables::session_for(&p.stores, &[]).map_err(|e| e.to_string())?;
    let (_, batches) = run_query(&ctx, &ordered_ready_sql())?;
    let mut rows = string_rows(&batches);
    let sequenced = sort_total(&mut rows, &sequence, &p.reg);
    let row = rows.first();

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
    let p = load_portfolio(true)?;
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
fn union_session(prune: bool) -> Result<(SessionContext, Portfolio), String> {
    let p = load_portfolio(prune)?;
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
    let (ctx, p) = union_session(true)?;
    // The normative §5 ready SQL with the repo column joined in — the
    // predicate is untouched (one semantics, MW-G3); presentation is the
    // MW-G4 total ordering `next` picks row 0 from.
    let sequence = registry::load_sequence(&p.dir)?;
    let (_, batches) = run_query(&ctx, &ordered_ready_sql())?;
    let mut rows = string_rows(&batches);
    sort_total(&mut rows, &sequence, &p.reg);
    let total = rows.len();
    let cap = LISTING_CAP.min(total);

    if json {
        let shown: Vec<_> = rows[..cap]
            .iter()
            .map(|r| {
                serde_json::json!({ "repo": r[0], "id": r[1], "title": r[4],
                    "claimed_by": (!r[5].is_empty()).then(|| r[5].clone()) })
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
            let claim = if row[5].is_empty() {
                String::new()
            } else {
                format!("  [claimed: {}]", row[5])
            };
            println!("{}#{}  {}{claim}", row[0], row[1], row[4]);
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
    // A pure read (MW-S14): the overlay is never rewritten under a query.
    let (ctx, p) = union_session(false)?;
    if crate::views::mentioned(sql) {
        // The union has no config of its own: the default window (MW-S5).
        let clock = crate::views::Clock::resolve(crate::views::DEFAULT_WINDOW_DAYS)?;
        crate::views::register_blocking(&ctx, &clock, sql)?;
    }
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
