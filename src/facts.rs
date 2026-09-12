//! Lifecycle facts in Rust (FORMAT.md §Views: `events`, `facts`, and
//! `attention`'s `touched_since_move`) — the per-task scalars the pulse
//! block reads, computed from the parsed tasks under the same clock the
//! SQL session registers. The published SQL is the specification; the
//! pulse differential test holds the two equal.
//!
//! Every stamp goes through the same guard as the views: a conforming
//! stamp becomes an instant, anything else is an event with no `at` —
//! counted where the SQL counts it, invisible where the SQL needs a time.

use crate::parse::{ParsedTask, Status, Task};
use crate::store::RepoStore;
use crate::views::{stamp_secs, Clock};
use std::collections::BTreeSet;

/// Idle hours past which a `doing` task counts as stale in the pulse.
pub const STALE_DOING_H: f64 = 72.0;
/// Age hours past which a live task counts as past triage.
pub const TRIAGE_H: f64 = 14.0 * 24.0;

/// One task's lifecycle scalars. Instants are epoch seconds; durations
/// hours, `None` where the SQL is NULL (no conforming stamp to measure).
#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::struct_excessive_bools)] // the view's boolean columns, one field each
pub struct TaskFacts {
    /// `repo#id`.
    pub gid: String,
    /// The store's registry name.
    pub repo: String,
    /// Bare task id.
    pub id: String,
    /// Current status.
    pub status: Status,
    /// Open, doing or blocked.
    pub live: bool,
    /// The task's category, as written.
    pub category: Option<String>,
    /// `seq:` present.
    pub has_seq: bool,
    /// Carries `discovered-from`.
    pub discovered_from: bool,
    /// `created` through the guard.
    pub created_at: Option<i64>,
    /// Latest `→done`/`→dropped`, only while the status is terminal.
    pub terminal_at: Option<i64>,
    /// Latest transition with a conforming stamp.
    pub last_transition: Option<i64>,
    /// Latest conforming stamp on any event, `created` included.
    pub last_activity: Option<i64>,
    /// Created to terminal, or to now.
    pub age_h: Option<f64>,
    /// Now since the last activity (or creation).
    pub idle_h: Option<f64>,
    /// Created inside the window.
    pub filed_w: bool,
    /// Entered `doing` inside the window.
    pub started_w: bool,
    /// Close attempts logged inside the window.
    pub close_attempts_w: i64,
    /// Transitions into `open` from another status inside the window.
    pub reopens_w: i64,
    /// Transitions into `blocked` inside the window.
    pub blocks_w: i64,
    /// Comments dated inside the window.
    pub comments_w: i64,
    /// Distinct actors on events inside the window.
    pub actors_w: BTreeSet<String>,
    /// Distinct actors on events after the last transition (every actor
    /// when there is none) — `attention.touched_since_move`.
    pub touched_since_move: i64,
}

/// One `events` row, the columns the facts read.
struct Event {
    at: Option<i64>,
    actor: Option<String>,
    from: Option<Status>,
    to: Option<Status>,
    comment: bool,
    close_attempt: bool,
}

/// The `events` rows of one task: its `created`, every log line, every
/// comment — actors read as the SQL reads them (`claimed by …`,
/// `handoff by …`, comment authors).
fn events(t: &Task) -> Vec<Event> {
    let mut out = Vec::new();
    if let Some(created) = &t.created {
        out.push(Event {
            at: stamp_secs(created),
            actor: None,
            from: None,
            to: None,
            comment: false,
            close_attempt: false,
        });
    }
    for line in &t.log {
        let e = crate::parse::parse_log_line(line);
        let note = e.note.as_deref().unwrap_or_default();
        let actor = note
            .strip_prefix("claimed by ")
            .or_else(|| note.strip_prefix("handoff by "))
            .map(str::to_string);
        out.push(Event {
            at: e.date.as_deref().and_then(stamp_secs),
            actor,
            from: e.from,
            to: e.to,
            comment: false,
            close_attempt: e.to.is_none() && note.starts_with("close attempt"),
        });
    }
    for c in &t.comments {
        out.push(Event {
            at: stamp_secs(&c.date),
            actor: Some(c.author.clone()),
            from: None,
            to: None,
            comment: true,
            close_attempt: false,
        });
    }
    out
}

/// Hours between two instants — the engine's `to_unixtime` arithmetic.
#[must_use]
#[allow(clippy::cast_precision_loss)] // epoch seconds sit far inside f64's exact range
pub fn hours(from: i64, to: i64) -> f64 {
    (to - from) as f64 / 3600.0
}

fn facts_of(store: &RepoStore, t: &Task, clock: &Clock) -> TaskFacts {
    let since = clock.now_secs - clock.window_days * 86_400;
    let ev = events(t);
    let created_at = t.created.as_deref().and_then(stamp_secs);
    let terminal = matches!(t.status, Status::Done | Status::Dropped);
    let max_at =
        |pick: &dyn Fn(&Event) -> bool| ev.iter().filter(|e| pick(e)).filter_map(|e| e.at).max();
    let terminal_at =
        max_at(&|e| matches!(e.to, Some(Status::Done | Status::Dropped))).filter(|_| terminal);
    let last_transition = max_at(&|e| e.to.is_some());
    let last_activity = ev.iter().filter_map(|e| e.at).max();
    let age_h = created_at.map(|c| hours(c, terminal_at.unwrap_or(clock.now_secs)));
    let idle_h = last_activity
        .or(created_at)
        .map(|a| hours(a, clock.now_secs));

    let in_window = |e: &&Event| e.at.is_some_and(|at| at >= since);
    let windowed: Vec<&Event> = ev.iter().filter(in_window).collect();
    let count = |pick: &dyn Fn(&Event) -> bool| {
        i64::try_from(windowed.iter().filter(|e| pick(e)).count()).unwrap_or(i64::MAX)
    };
    let touched: BTreeSet<&str> = ev
        .iter()
        .filter(|e| match last_transition {
            Some(moved) => e.at.is_some_and(|at| at > moved),
            None => true,
        })
        .filter_map(|e| e.actor.as_deref())
        .collect();

    TaskFacts {
        gid: store.gid(&t.id),
        repo: store.repo.clone(),
        id: t.id.clone(),
        status: t.status,
        live: !terminal,
        category: t.category.clone(),
        has_seq: t.seq.is_some(),
        discovered_from: t.discovered_from.is_some(),
        created_at,
        terminal_at,
        last_transition,
        last_activity,
        age_h,
        idle_h,
        filed_w: created_at.is_some_and(|c| c >= since),
        started_w: windowed.iter().any(|e| e.to == Some(Status::Doing)),
        close_attempts_w: count(&|e| e.close_attempt),
        reopens_w: count(&|e| e.to == Some(Status::Open) && e.from.is_some()),
        blocks_w: count(&|e| e.to == Some(Status::Blocked)),
        comments_w: count(&|e| e.comment),
        actors_w: windowed.iter().filter_map(|e| e.actor.clone()).collect(),
        touched_since_move: i64::try_from(touched.len()).unwrap_or(i64::MAX),
    }
}

/// The facts of every valid task in the stores, in store and file order.
#[must_use]
pub fn compute(stores: &[RepoStore], clock: &Clock) -> Vec<TaskFacts> {
    let mut out = Vec::new();
    for store in stores {
        for entry in &store.entries {
            if let ParsedTask::Valid(t) = &entry.parsed {
                out.push(facts_of(store, t, clock));
            }
        }
    }
    out
}

/// `round(x, 1)` as the engine computes it.
#[must_use]
pub fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}

/// The engine's median: the middle value, or the mean of the two middles.
#[must_use]
pub fn median(values: &mut [f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f64::total_cmp);
    let n = values.len();
    Some(if n.is_multiple_of(2) {
        f64::midpoint(values[n / 2 - 1], values[n / 2])
    } else {
        values[n / 2]
    })
}
