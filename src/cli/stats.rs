//! `stats [--window <n>d]` and `portfolio stats` (mw-549rh9w): the derived
//! projection read as tables — every one a canned SELECT over a published
//! view, the `ready`/`search` pattern, never a language. Text prints the
//! pulse first, then the tables; `--json` carries each table as
//! `{columns, rows}` inside the standard envelope. A read: it orders
//! nothing and writes nothing.

use crate::cli::query::{q_payload, run_query};
use crate::pulse::Pulse;
use datafusion::prelude::SessionContext;
use serde_json::Value;
use std::fmt::Write as _;

#[derive(clap::Args)]
pub(crate) struct StatsArgs {
    /// Width of every windowed count, in days: 7d, 28d (default: the
    /// store's configured stats window, else 7).
    #[arg(long, value_name = "Nd")]
    window: Option<String>,
}

impl StatsArgs {
    /// The window in days: the flag when given, else `default`.
    pub(crate) fn window_days(&self, default: i64) -> Result<i64, String> {
        self.window.as_deref().map_or(Ok(default), parse_window)
    }
}

fn parse_window(text: &str) -> Result<i64, String> {
    match text.trim().trim_end_matches('d').parse::<i64>() {
        Ok(n) if n >= 1 => Ok(n),
        _ => Err(format!(
            "--window takes a whole number of days such as 7d or 28d, got `{text}`"
        )),
    }
}

/// The views the tables read, named so registration plans those and
/// their dependencies only.
const VIEWS_READ: &str = "pulse flow hazard spans graph lineage attention facts";
/// Weeks the flow table shows.
const WEEKS: usize = 6;
/// The hazard bins shown; the survival column multiplies through every
/// bin below each, shown or not.
const HAZARD_BINS: [i64; 17] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 21, 28];

const PULSE_SQL: &str = "SELECT * FROM pulse ORDER BY repo";

/// Calendar weeks, Monday-start, pooled over repos; one extra week so the
/// oldest shown still has a week-over-week delta.
const FLOW_WEEKLY_SQL: &str = "\
SELECT week, sum(filed) AS filed, sum(done) AS done, sum(dropped) AS dropped,
       sum(CASE WHEN day = week_end THEN backlog ELSE 0 END) AS backlog
FROM (SELECT date_trunc('week', CAST(day AS TIMESTAMP)) AS week, day, filed, done, dropped, backlog,
             max(day) OVER (PARTITION BY repo, date_trunc('week', CAST(day AS TIMESTAMP))) AS week_end
      FROM flow)
GROUP BY week ORDER BY week DESC LIMIT 7";

const HAZARD_SQL: &str = "\
SELECT age_d, sum(at_risk) AS at_risk, sum(done_in) AS done, sum(dropped_in) AS dropped
FROM hazard GROUP BY age_d ORDER BY age_d";

const SPANS_SQL: &str = "\
SELECT state, count(*) AS n, round(median(hours), 1) AS med_h,
       round(approx_percentile_cont(hours, 0.9), 1) AS p90_h, round(max(hours), 1) AS max_h
FROM spans GROUP BY state ORDER BY state";

/// Live lanes of two or more, largest first; `head` is the member that
/// unlocks the most.
const LANES_SQL: &str = "\
SELECT lane, size, head, ready, blocked_foreign FROM (
  SELECT lane, lane_size AS size, gid AS head,
         sum(CASE WHEN ready THEN 1 ELSE 0 END) OVER (PARTITION BY lane) AS ready,
         sum(CASE WHEN needs_open_foreign > 0 THEN 1 ELSE 0 END) OVER (PARTITION BY lane)
           AS blocked_foreign,
         row_number() OVER (PARTITION BY lane ORDER BY unlock DESC, gid) AS rn
  FROM graph WHERE lane_size > 1 AND status IN ('open', 'doing', 'blocked'))
WHERE rn = 1 ORDER BY size DESC, lane LIMIT 10";

const GRAVITY_SQL: &str = "\
SELECT gid, status, gravity, spawned_total, descendants_total, referrers_edges, mentioned_by
FROM lineage WHERE gravity > 0 ORDER BY gravity DESC, gid LIMIT 10";

const SPAWNERS_SQL: &str = "\
SELECT gid, status, spawned_w, spawned_total, spawned_live
FROM lineage WHERE spawned_w > 0 ORDER BY spawned_w DESC, gid LIMIT 10";

const TOUCHED_SQL: &str = "\
SELECT gid, status, touched_since_move, actors, round(idle_h, 1) AS idle_h
FROM attention WHERE touched_since_move > 0 ORDER BY touched_since_move DESC, gid LIMIT 10";

/// Live placement per repo; `collisions` is the surplus of seq'd tasks
/// over distinct seqs, `parked` the pulse's parked-in-place rule.
const PLACEMENT_SQL: &str = "\
SELECT repo, count(*) AS live, count(*) FILTER (WHERE has_seq) AS seqd,
       count(DISTINCT seq) AS distinct_seq,
       count(*) FILTER (WHERE has_seq) - count(DISTINCT seq) AS collisions,
       count(*) FILTER (WHERE has_seq AND idle_h >= 336) AS parked
FROM facts WHERE live GROUP BY repo ORDER BY repo";

/// One table of the report: its JSON key, its text title, and rows of
/// typed cells in the `q` shape.
struct Table {
    key: &'static str,
    title: String,
    columns: Vec<String>,
    rows: Vec<Vec<Value>>,
}

pub(crate) struct Report {
    window_days: i64,
    /// One repo or the union — decides whether ids drop their `repo#`.
    union: bool,
    tables: Vec<Table>,
}

/// `stats` over the local store.
pub(crate) fn run(args: &StatsArgs, json: bool) -> Result<(), String> {
    let (ctx, store) = crate::cli::query::local_session_store()?;
    let window_days = args.window_days(store.config.window_days())?;
    let report = collect(&ctx, window_days, false)?;
    emit(&report, "stats", json, None);
    Ok(())
}

/// Every table over an already-built session — the local store's, or the
/// union's for `portfolio stats`.
pub(crate) fn collect(
    ctx: &SessionContext,
    window_days: i64,
    union: bool,
) -> Result<Report, String> {
    let clock = crate::views::Clock::resolve(window_days)?;
    crate::views::register_blocking(ctx, &clock, VIEWS_READ)?;
    let pulse = query(
        ctx,
        "pulse",
        format!("pulse ({window_days}d window)"),
        PULSE_SQL,
    )?;
    let mentions = mentions_of(&pulse);
    let tables = vec![
        pulse,
        flow_weekly(query(
            ctx,
            "flow_weekly",
            "flow, weekly (Monday-start weeks)".into(),
            FLOW_WEEKLY_SQL,
        )?),
        hazard(query(
            ctx,
            "hazard",
            "close hazard (pooled, daily bins)".into(),
            HAZARD_SQL,
        )?),
        query(ctx, "spans", "spans by state (hours)".into(), SPANS_SQL)?,
        query(
            ctx,
            "lanes",
            "lanes (live needs components, largest first)".into(),
            LANES_SQL,
        )?,
        query(ctx, "top_gravity", "top 10 by gravity".into(), GRAVITY_SQL)?,
        query(
            ctx,
            "top_spawners",
            format!("top 10 spawners ({window_days}d)"),
            SPAWNERS_SQL,
        )?,
        query(
            ctx,
            "top_touched",
            "top 10 touched since last move".into(),
            TOUCHED_SQL,
        )?,
        mentions,
        query(
            ctx,
            "placement",
            "placement (live tasks)".into(),
            PLACEMENT_SQL,
        )?,
    ];
    Ok(Report {
        window_days,
        union,
        tables,
    })
}

/// Print the report: the envelope with every table under `tables`, or
/// the text sections. `extra` rides the JSON data (the union's skips).
pub(crate) fn emit(report: &Report, verb: &str, json: bool, extra: Option<(&str, Value)>) {
    if json {
        let mut tables = serde_json::Map::new();
        for t in &report.tables {
            tables.insert(
                t.key.to_string(),
                serde_json::json!({ "columns": t.columns, "rows": t.rows }),
            );
        }
        let mut data = serde_json::json!({ "window_days": report.window_days, "tables": tables });
        if let Some((key, value)) = extra {
            data[key] = value;
        }
        crate::cli::emit_json(verb, &data);
    } else {
        print!("{}", crate::cli::sanitize(&render(report)));
    }
}

fn query(
    ctx: &SessionContext,
    key: &'static str,
    title: String,
    sql: &str,
) -> Result<Table, String> {
    let (columns, batches) = run_query(ctx, sql)?;
    let payload = q_payload(&columns, &batches);
    let rows = payload["rows"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(|r| r.as_array().cloned().unwrap_or_default())
                .collect()
        })
        .unwrap_or_default();
    Ok(Table {
        key,
        title,
        columns,
        rows,
    })
}

fn column(t: &Table, name: &str) -> Option<usize> {
    t.columns.iter().position(|c| c == name)
}

fn int_at(row: &[Value], at: Option<usize>) -> i64 {
    at.and_then(|i| row.get(i))
        .and_then(Value::as_i64)
        .unwrap_or(0)
}

fn float_at(row: &[Value], at: Option<usize>) -> Option<f64> {
    at.and_then(|i| row.get(i)).and_then(Value::as_f64)
}

fn round3(x: f64) -> Value {
    Value::from((x * 1000.0).round() / 1000.0)
}

/// Week-over-week deltas: each shown week against the one before it; the
/// extra row the query fetched serves the oldest shown, then goes.
fn flow_weekly(t: Table) -> Table {
    let (week, filed, done, dropped, backlog) = (
        column(&t, "week"),
        column(&t, "filed"),
        column(&t, "done"),
        column(&t, "dropped"),
        column(&t, "backlog"),
    );
    let mut rows = Vec::new();
    for (i, row) in t.rows.iter().take(WEEKS).enumerate() {
        let older = t.rows.get(i + 1);
        let delta = |at: Option<usize>| {
            older.map_or(Value::Null, |o| {
                Value::from(int_at(row, at) - int_at(o, at))
            })
        };
        let day = week
            .and_then(|i| row.get(i))
            .and_then(Value::as_str)
            .map(|s| s.chars().take(10).collect::<String>())
            .unwrap_or_default();
        rows.push(vec![
            Value::from(day),
            Value::from(int_at(row, filed)),
            delta(filed),
            Value::from(int_at(row, done)),
            delta(done),
            Value::from(int_at(row, dropped)),
            delta(dropped),
            Value::from(int_at(row, backlog)),
        ]);
    }
    Table {
        key: t.key,
        title: t.title,
        columns: [
            "week",
            "filed",
            "filed_delta",
            "done",
            "done_delta",
            "dropped",
            "dropped_delta",
            "backlog",
        ]
        .map(String::from)
        .to_vec(),
        rows,
    }
}

/// The discrete hazards per bin and the survival curve — the product of
/// `1 - done/at_risk` over every bin up to and including each one.
fn hazard(t: Table) -> Table {
    let (age, at_risk, done, dropped) = (
        column(&t, "age_d"),
        column(&t, "at_risk"),
        column(&t, "done"),
        column(&t, "dropped"),
    );
    let mut survival = 1.0_f64;
    let mut rows = Vec::new();
    for row in &t.rows {
        let (bin, risk, d, x) = (
            int_at(row, age),
            int_at(row, at_risk),
            int_at(row, done),
            int_at(row, dropped),
        );
        #[allow(clippy::cast_precision_loss)]
        let (h_done, h_drop) = if risk > 0 {
            (d as f64 / risk as f64, x as f64 / risk as f64)
        } else {
            (0.0, 0.0)
        };
        survival *= 1.0 - h_done;
        if HAZARD_BINS.contains(&bin) {
            rows.push(vec![
                Value::from(bin),
                Value::from(risk),
                Value::from(d),
                Value::from(x),
                round3(h_done),
                round3(h_drop),
                round3(survival),
            ]);
        }
    }
    Table {
        key: t.key,
        title: t.title,
        columns: [
            "age_d",
            "at_risk",
            "done",
            "dropped",
            "h_done",
            "h_dropped",
            "survival",
        ]
        .map(String::from)
        .to_vec(),
        rows,
    }
}

/// The mention-health counts summed over the pulse rows.
fn mentions_of(pulse: &Table) -> Table {
    let sum = |name: &str| {
        let at = column(pulse, name);
        Value::from(pulse.rows.iter().map(|r| int_at(r, at)).sum::<i64>())
    };
    Table {
        key: "mentions",
        title: "mentions".into(),
        columns: ["handoffs_citing_closed", "live_unlinked", "unlinked"]
            .map(String::from)
            .to_vec(),
        rows: vec![vec![
            sum("handoff_stale_n"),
            sum("implicit_live_n"),
            sum("unlinked_mentions"),
        ]],
    }
}

/// A pulse row as the struct `prime`'s weather lines render; the
/// Rust-side extras (top ids, triage groups) stay empty.
fn pulse_from(t: &Table, row: &[Value]) -> Pulse {
    let int = |name: &str| int_at(row, column(t, name));
    let float = |name: &str| float_at(row, column(t, name));
    Pulse {
        repo: column(t, "repo")
            .and_then(|i| row.get(i))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        open_n: int("open_n"),
        doing_n: int("doing_n"),
        blocked_n: int("blocked_n"),
        done_n: int("done_n"),
        dropped_n: int("dropped_n"),
        filed_w: int("filed_w"),
        filed_from_w: int("filed_from_w"),
        done_w: int("done_w"),
        dropped_w: int("dropped_w"),
        started_w: int("started_w"),
        backlog_delta_w: int("backlog_delta_w"),
        doing_stale: int("doing_stale"),
        open_age_med_d: float("open_age_med_d"),
        past_triage: int("past_triage"),
        ready_n: int("ready_n"),
        unlockers: int("unlockers"),
        lanes_multi: int("lanes_multi"),
        needs_behind_n: int("needs_behind_n"),
        blocked_foreign: int("blocked_foreign"),
        blocked_unresolved: int("blocked_unresolved"),
        owed_foreign: int("owed_foreign"),
        asks_in_open: int("asks_in_open"),
        asks_in_oldest_d: float("asks_in_oldest_d"),
        asks_out_open: int("asks_out_open"),
        asks_out_oldest_d: float("asks_out_oldest_d"),
        close_attempts_w: int("close_attempts_w"),
        reopens_w: int("reopens_w"),
        blocks_w: int("blocks_w"),
        thrash_n: int("thrash_n"),
        handoff_stale_n: int("handoff_stale_n"),
        ..Pulse::default()
    }
}

/// The per-repo pulse table `portfolio stats` prints: the counts a
/// session reads first, one row per repo and a total.
const PULSE_UNION_COLUMNS: [(&str, &str); 12] = [
    ("repo", "repo"),
    ("open_n", "open"),
    ("doing_n", "doing"),
    ("blocked_n", "blocked"),
    ("ready_n", "ready"),
    ("filed_w", "filed"),
    ("done_w", "done"),
    ("dropped_w", "dropped"),
    ("backlog_delta_w", "backlog_delta"),
    ("past_triage", "past_14d"),
    ("asks_in_open", "asks_in"),
    ("asks_out_open", "asks_out"),
];

fn render(report: &Report) -> String {
    let mut out = String::new();
    for t in &report.tables {
        match t.key {
            "pulse" => pulse_section(&mut out, t, report),
            "mentions" => {
                let row = t.rows.first().cloned().unwrap_or_default();
                let _ = writeln!(
                    out,
                    "mentions: handoffs citing closed tasks {} \u{b7} live\u{2192}live unlinked {} \
                     \u{b7} unlinked in all {}",
                    cell_text(row.first(), ""),
                    cell_text(row.get(1), ""),
                    cell_text(row.get(2), "")
                );
            }
            "placement" => {
                section(&mut out, t, report);
                out.push_str("  placed by: seq or unranked \u{2014} bands are not built\n");
            }
            _ => section(&mut out, t, report),
        }
    }
    out
}

fn pulse_section(out: &mut String, t: &Table, report: &Report) {
    let _ = writeln!(out, "{}:", t.title);
    if report.union {
        let picks: Vec<Option<usize>> = PULSE_UNION_COLUMNS
            .iter()
            .map(|(name, _)| column(t, name))
            .collect();
        let mut rows: Vec<Vec<Value>> = t
            .rows
            .iter()
            .map(|r| {
                picks
                    .iter()
                    .map(|at| at.and_then(|i| r.get(i)).cloned().unwrap_or(Value::Null))
                    .collect()
            })
            .collect();
        let mut total: Vec<Value> = vec![Value::from("total")];
        for at in 1..PULSE_UNION_COLUMNS.len() {
            total.push(Value::from(
                rows.iter()
                    .map(|r| r.get(at).and_then(Value::as_i64).unwrap_or(0))
                    .sum::<i64>(),
            ));
        }
        rows.push(total);
        let columns: Vec<String> = PULSE_UNION_COLUMNS
            .iter()
            .map(|(_, label)| (*label).to_string())
            .collect();
        table_text(out, &columns, &rows, "");
    } else {
        let cuts = super::pulse::Cuts::default();
        let lines = t
            .rows
            .first()
            .map(|row| super::pulse::lines(&pulse_from(t, row), report.window_days, &cuts))
            .unwrap_or_default();
        if lines.is_empty() {
            out.push_str("  (nothing counted)\n");
        }
        for line in lines {
            let _ = writeln!(out, "{line}");
        }
    }
}

fn section(out: &mut String, t: &Table, report: &Report) {
    let _ = writeln!(out, "{}:", t.title);
    if t.rows.is_empty() {
        out.push_str("  (none)\n");
        return;
    }
    let prefix = if report.union {
        String::new()
    } else {
        repo_prefix(t)
    };
    table_text(out, &t.columns, &t.rows, &prefix);
}

/// The one repo's `repo#`, stripped from ids in single-store text; the
/// union keeps every id whole.
fn repo_prefix(t: &Table) -> String {
    t.columns
        .iter()
        .position(|c| c == "gid" || c == "lane")
        .and_then(|i| {
            t.rows
                .first()
                .and_then(|r| r.get(i))
                .and_then(Value::as_str)
        })
        .and_then(|gid| gid.split_once('#').map(|(repo, _)| format!("{repo}#")))
        .unwrap_or_default()
}

/// Header, rows, columns padded to their widest cell; numbers right-align.
/// With a `repo#` to strip, the id columns are headed `id`.
fn table_text(out: &mut String, columns: &[String], rows: &[Vec<Value>], strip: &str) {
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            columns
                .iter()
                .enumerate()
                .map(|(i, name)| {
                    cell_text(r.get(i), name)
                        .trim_start_matches(strip)
                        .to_string()
                })
                .collect()
        })
        .collect();
    let columns: Vec<String> = columns
        .iter()
        .map(|c| {
            if !strip.is_empty() && c == "gid" {
                "id".to_string()
            } else {
                c.clone()
            }
        })
        .collect();
    let columns = &columns;
    let widths: Vec<usize> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            cells
                .iter()
                .map(|r| r[i].chars().count())
                .chain(std::iter::once(c.chars().count()))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let numeric: Vec<bool> = columns
        .iter()
        .enumerate()
        .map(|(i, _)| rows.iter().any(|r| r.get(i).is_some_and(Value::is_number)))
        .collect();
    let line = |cells: &[String]| {
        let mut s = String::from(" ");
        for (i, cell) in cells.iter().enumerate() {
            let w = widths[i];
            if numeric[i] {
                let _ = write!(s, " {cell:>w$}");
            } else {
                let _ = write!(s, " {cell:<w$}");
            }
        }
        s.trim_end().to_string()
    };
    let _ = writeln!(out, "{}", line(columns));
    for r in &cells {
        let _ = writeln!(out, "{}", line(r));
    }
}

/// A cell as text: NULL empty, deltas signed, rates to three places,
/// other fractions to one, everything else as JSON prints it.
fn cell_text(v: Option<&Value>, name: &str) -> String {
    match v {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) if name.ends_with("_delta") => n
            .as_i64()
            .map_or_else(|| n.to_string(), |i| format!("{i:+}")),
        Some(Value::Number(n)) => match (n.as_i64(), n.as_f64()) {
            (Some(i), _) => i.to_string(),
            (None, Some(f)) if matches!(name, "h_done" | "h_dropped" | "survival") => {
                format!("{f:.3}")
            }
            (None, Some(f)) => format!("{f:.1}"),
            (None, None) => n.to_string(),
        },
        Some(Value::Bool(b)) => b.to_string(),
        Some(other) => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_parses_days_only() {
        assert_eq!(parse_window("7d").unwrap(), 7);
        assert_eq!(parse_window("28").unwrap(), 28);
        assert!(parse_window("0d").is_err());
        assert!(parse_window("week").is_err());
    }

    #[test]
    fn survival_multiplies_through_every_bin() {
        let t = Table {
            key: "hazard",
            title: String::new(),
            columns: ["age_d", "at_risk", "done", "dropped"]
                .map(String::from)
                .to_vec(),
            rows: vec![
                vec![0.into(), 10.into(), 5.into(), 0.into()],
                vec![1.into(), 4.into(), 2.into(), 1.into()],
                vec![15.into(), 2.into(), 1.into(), 0.into()],
                vec![21.into(), 1.into(), 0.into(), 0.into()],
            ],
        };
        let h = hazard(t);
        let survival: Vec<f64> = h.rows.iter().map(|r| r[6].as_f64().unwrap()).collect();
        // Bin 15 is not shown but still discounts bin 21's survival.
        assert_eq!(survival, vec![0.5, 0.25, 0.125]);
        assert_eq!(h.rows.len(), 3);
        assert_eq!(h.rows[1][5], round3(0.25));
    }

    #[test]
    fn weekly_deltas_use_the_extra_row() {
        let row = |w: &str, f: i64| vec![Value::from(w), f.into(), 0.into(), 0.into(), 3.into()];
        let t = Table {
            key: "flow_weekly",
            title: String::new(),
            columns: ["week", "filed", "done", "dropped", "backlog"]
                .map(String::from)
                .to_vec(),
            rows: vec![row("2026-08-10T00:00:00", 4), row("2026-08-03T00:00:00", 1)],
        };
        let f = flow_weekly(t);
        assert_eq!(f.rows.len(), 2);
        assert_eq!(f.rows[0][0], "2026-08-10");
        assert_eq!(f.rows[0][2], 3);
        assert_eq!(f.rows[1][2], Value::Null);
        assert_eq!(cell_text(Some(&f.rows[0][2]), "filed_delta"), "+3");
    }
}
