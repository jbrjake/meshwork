//! The derived projection (FORMAT.md §Views, MW-S1): the one-row `clock`
//! table and the thirteen public views, registered from `FORMAT-views.sql`
//! verbatim into a session that serves a query — `q`, `portfolio q` — and
//! never into the gated `ready`/`prime` sessions, which must not pay the
//! planning cost of fifty view bodies they never read.

use datafusion::arrow::array::{
    ArrayRef, Date32Array, Int64Array, StringArray, TimestampNanosecondArray,
};
use datafusion::arrow::datatypes::{DataType, Field, Schema, TimeUnit};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::prelude::SessionContext;
use std::sync::Arc;

/// The reference SQL, published beside FORMAT.md and registered as is.
pub const SQL: &str = include_str!("../FORMAT-views.sql");

/// The thirteen contract names (FORMAT.md §Views), `clock` included —
/// what `q`'s error path and help list after the six tables.
pub const VIEWS: [&str; 13] = [
    "clock",
    "events",
    "spans",
    "facts",
    "graph",
    "asks",
    "mentions",
    "lineage",
    "attention",
    "sessions",
    "flow",
    "hazard",
    "pulse",
];

/// `[stats] window_days` when the store does not say (MW-S5).
pub const DEFAULT_WINDOW_DAYS: i64 = 7;

/// The `clock` row: where every age, idle and window is measured from.
#[derive(Debug, Clone, Copy)]
pub struct Clock {
    /// `now` as epoch seconds, UTC.
    pub now_secs: i64,
    /// `override` when `MESHWORK_TODAY` set it, else `system`.
    pub source: &'static str,
    /// The width of every `_w` column, in days.
    pub window_days: i64,
}

impl Clock {
    /// `MESHWORK_TODAY` in either conforming form — a minute stamp, or a
    /// date (midnight UTC) — else the wall clock at minute resolution.
    /// Anything else is refused: a nonconforming clock would silently
    /// shift every duration in the projection.
    ///
    /// # Errors
    /// The override is set and is not a conforming stamp.
    pub fn resolve(window_days: i64) -> Result<Self, String> {
        let stamp = crate::clock::stamp();
        let source = if std::env::var("MESHWORK_TODAY").is_ok_and(|v| !v.trim().is_empty()) {
            "override"
        } else {
            "system"
        };
        let now_secs = stamp_secs(&stamp).ok_or_else(|| {
            format!(
                "MESHWORK_TODAY must be YYYY-MM-DD or YYYY-MM-DDTHH:MMZ, got `{stamp}` — \
                 the clock would shift every duration in the derived projection"
            )
        })?;
        Ok(Self {
            now_secs,
            source,
            window_days,
        })
    }
}

/// Epoch seconds of a conforming stamp; `None` for any other text.
fn stamp_secs(stamp: &str) -> Option<i64> {
    let (date, minutes) = match stamp.len() {
        10 => (stamp, 0),
        17 => {
            let b = stamp.as_bytes();
            if b[10] != b'T' || b[13] != b':' || b[16] != b'Z' {
                return None;
            }
            let h: i64 = stamp[11..13].parse().ok()?;
            let m: i64 = stamp[14..16].parse().ok()?;
            if h > 23 || m > 59 {
                return None;
            }
            (&stamp[..10], h * 60 + m)
        }
        _ => return None,
    };
    if !date.bytes().enumerate().all(|(i, c)| {
        if i == 4 || i == 7 {
            c == b'-'
        } else {
            c.is_ascii_digit()
        }
    }) {
        return None;
    }
    Some(crate::clock::days_from_civil(date)? * 86_400 + minutes * 60)
}

fn clock_batch(clock: &Clock) -> datafusion::error::Result<RecordBatch> {
    let schema = Arc::new(Schema::new(vec![
        Field::new(
            "now",
            DataType::Timestamp(TimeUnit::Nanosecond, None),
            false,
        ),
        Field::new("today", DataType::Date32, false),
        Field::new("source", DataType::Utf8, false),
        Field::new("window_days", DataType::Int64, false),
    ]));
    let day = i32::try_from(clock.now_secs.div_euclid(86_400)).unwrap_or(0);
    let columns: Vec<ArrayRef> = vec![
        Arc::new(TimestampNanosecondArray::from(vec![
            clock.now_secs * 1_000_000_000,
        ])),
        Arc::new(Date32Array::from(vec![day])),
        Arc::new(StringArray::from(vec![clock.source])),
        Arc::new(Int64Array::from(vec![clock.window_days])),
    ];
    Ok(RecordBatch::try_new(schema, columns)?)
}

/// The layers several later views read. The engine inlines a view at every
/// reference, so `facts` inside `pulse` would otherwise be computed eight
/// times over; these are run once and re-registered as tables under the
/// same name before anything downstream is planned. Pure functions of the
/// six tables and `clock` (MW-S2), so the relation is the same either way.
const MATERIALIZE: [&str; 6] = ["events", "facts", "graph", "asks", "mentions", "lineage"];

/// One `CREATE VIEW` of the published SQL: its name and full statement.
struct Statement {
    name: String,
    sql: String,
}

/// The statements of the published SQL, comments stripped, in file order —
/// registration order matters, each view names the ones before it.
fn statements() -> Vec<Statement> {
    let body: String = SQL
        .lines()
        .filter(|l| !l.trim_start().starts_with("--"))
        .collect::<Vec<_>>()
        .join("\n");
    body.split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| Statement {
            name: s
                .trim_start_matches("CREATE VIEW ")
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .to_string(),
            sql: s.to_string(),
        })
        .collect()
}

fn names_in(text: &str, names: &[&str]) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let ident = |c: Option<&u8>| c.is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_');
    names
        .iter()
        .filter(|n| {
            lower.match_indices(*n).any(|(at, _)| {
                !ident(at.checked_sub(1).and_then(|i| lower.as_bytes().get(i)))
                    && !ident(lower.as_bytes().get(at + n.len()))
            })
        })
        .map(|n| (*n).to_string())
        .collect()
}

/// Does this SQL name one of the thirteen views? Only then is the
/// projection registered at all: a query over the six tables alone never
/// pays for it. Helper views are not contract and are reachable only
/// through the thirteen.
#[must_use]
pub fn mentioned(sql: &str) -> bool {
    !names_in(sql, &VIEWS).is_empty()
}

/// The statements a query needs: the thirteen it names, and transitively
/// every helper they read — in file order, so each is created after the
/// ones it references. `None` for the whole projection.
fn needed(all: &[Statement], sql: Option<&str>) -> Vec<usize> {
    let names: Vec<&str> = all.iter().map(|s| s.name.as_str()).collect();
    let Some(sql) = sql else {
        return (0..all.len()).collect();
    };
    let mut want: std::collections::BTreeSet<String> = names_in(sql, &VIEWS).into_iter().collect();
    let mut frontier: Vec<String> = want.iter().cloned().collect();
    while let Some(name) = frontier.pop() {
        if let Some(stmt) = all.iter().find(|s| s.name == name) {
            for dep in names_in(&stmt.sql, &names) {
                if dep != name && want.insert(dep.clone()) {
                    frontier.push(dep);
                }
            }
        }
    }
    (0..all.len())
        .filter(|i| want.contains(&all[*i].name))
        .collect()
}

async fn materialize(ctx: &SessionContext, name: &str) -> datafusion::error::Result<()> {
    let df = ctx.sql(&format!("SELECT * FROM {name}")).await?;
    let schema = Arc::new(df.schema().as_arrow().clone());
    let batches = df.collect().await?;
    ctx.deregister_table(name)?;
    let table = datafusion::datasource::MemTable::try_new(schema, vec![batches])?;
    ctx.register_table(name, Arc::new(table))?;
    Ok(())
}

/// Register `clock` and the views a query needs — every view when `sql`
/// is `None` — into `ctx`, which already carries the six tables.
///
/// # Errors
/// A view body fails to plan — a bug in the published SQL against this
/// engine, named by its first line.
pub async fn register(
    ctx: &SessionContext,
    clock: &Clock,
    sql: Option<&str>,
) -> Result<(), String> {
    ctx.register_batch("clock", clock_batch(clock).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    let all = statements();
    for i in needed(&all, sql) {
        let stmt = &all[i];
        let at = |e: datafusion::error::DataFusionError| {
            format!(
                "registering the derived projection: {e}\n  in: {}",
                stmt.sql.lines().next().unwrap_or_default()
            )
        };
        ctx.sql(&stmt.sql).await.map_err(at)?;
        if MATERIALIZE.contains(&stmt.name.as_str()) {
            materialize(ctx, &stmt.name).await.map_err(at)?;
        }
    }
    Ok(())
}

/// `register` from synchronous code: its own runtime, like every query.
///
/// # Errors
/// As `register`, or the runtime failing to build.
pub fn register_blocking(ctx: &SessionContext, clock: &Clock, sql: &str) -> Result<(), String> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    rt.block_on(register(ctx, clock, Some(sql)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conforming_stamps_only() {
        assert_eq!(stamp_secs("1970-01-01"), Some(0));
        assert_eq!(stamp_secs("1970-01-02T01:30Z"), Some(86_400 + 5_400));
        assert_eq!(stamp_secs("2026-09-01"), stamp_secs("2026-09-01T00:00Z"));
        for bad in [
            "2026-08-06T21:47-04:00",
            "2026-08-06T21:47:00Z",
            "2026-08-06 21:47Z",
            "fixed",
            "",
            "2026-13-01",
        ] {
            assert_eq!(stamp_secs(bad), None, "{bad}");
        }
    }

    #[test]
    fn mentions_whole_words_only() {
        assert!(mentioned("SELECT * FROM pulse"));
        assert!(mentioned("select repo, open_n from PULSE p"));
        assert!(mentioned(
            "SELECT count(*) FROM facts f JOIN tasks t ON t.gid = f.gid"
        ));
        assert!(!mentioned(
            "SELECT * FROM tasks WHERE title LIKE '%pulsed%'"
        ));
        assert!(!mentioned("SELECT gid, created FROM tasks"));
        assert!(!mentioned("SELECT * FROM g_lanes"));
    }

    #[test]
    fn published_sql_is_thirteen_views_plus_helpers() {
        let all = statements();
        let names: Vec<&str> = all.iter().map(|s| s.name.as_str()).collect();
        for v in VIEWS.iter().filter(|v| **v != "clock") {
            assert!(names.contains(v), "{v} missing from FORMAT-views.sql");
        }
        assert!(!names.contains(&"clock"), "clock is a table, never a view");
        for m in MATERIALIZE {
            assert!(names.contains(&m), "{m} is materialized but not published");
        }
    }

    #[test]
    fn needed_is_the_transitive_closure_in_file_order() {
        let all = statements();
        let name = |i: &usize| all[*i].name.as_str();
        let facts: Vec<&str> = needed(&all, Some("SELECT * FROM facts"))
            .iter()
            .map(name)
            .collect();
        assert_eq!(facts, ["events", "spans", "f_tr", "f_bl", "f_dg", "facts"]);
        let none = needed(&all, Some("SELECT * FROM tasks"));
        assert!(none.is_empty());
        assert_eq!(needed(&all, None).len(), all.len());
        let pulse: Vec<&str> = needed(&all, Some("SELECT * FROM pulse"))
            .iter()
            .map(name)
            .collect();
        for dep in ["facts", "graph", "asks", "mentions", "lineage", "attention"] {
            assert!(pulse.contains(&dep), "pulse reads {dep}");
        }
        for not in ["flow", "hazard", "sessions"] {
            assert!(!pulse.contains(&not), "pulse never reads {not}");
        }
    }
}
