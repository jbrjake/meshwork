//! The pulse block of `prime` (MW-S6): the repo's `pulse` row as up to
//! five weather lines — flow, queue, graph, asks, friction — every count
//! with its denominator, a line whose every count is zero omitted, each
//! line byte-clamped. The numbers are `pulse::compute`'s, equal to the
//! view by the differential test; this file only renders them. Also the
//! budget dance: when the digest would overflow, also-ready shrinks to a
//! floor, then the friction line goes, then the graph line — each cut
//! loud — before the digest's truncation tail is the last resort.

use crate::pulse::{Pulse, TRIAGE_GROUPS};
use crate::write::clamp_bytes;
use std::fmt::Write as _;

/// Per-line clamp, the digest's.
const LINE_CLAMP: usize = 160;
/// Also-ready rows the budget dance keeps at minimum.
pub(crate) const ALSO_READY_FLOOR: usize = 3;
/// What a cut line says instead of its numbers.
const CUT: &str = "cut (6KB budget)";

/// What the budget dance has given up so far.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Cuts {
    /// The inbox collapsed to its count, oldest age and the listing verb.
    pub inbox: bool,
    /// Also-ready rows shown.
    pub also_ready: usize,
    /// The friction line replaced by its cut marker.
    pub friction: bool,
    /// The graph line replaced by its cut marker.
    pub graph: bool,
}

fn any_nonzero(counts: &[i64]) -> bool {
    counts.iter().any(|c| *c != 0)
}

fn days(d: f64) -> String {
    format!("{d:.1}d")
}

fn flow(p: &Pulse, window_days: i64) -> Option<String> {
    any_nonzero(&[p.filed_w, p.filed_from_w, p.done_w, p.dropped_w]).then(|| {
        let live = p.open_n + p.doing_n + p.blocked_n;
        format!(
            "- flow {window_days}d: filed {} ({} from other tasks) \u{b7} done {} \u{b7} dropped {} \
             \u{b7} backlog {live} ({:+})",
            p.filed_w, p.filed_from_w, p.done_w, p.dropped_w, p.backlog_delta_w
        )
    })
}

fn queue(p: &Pulse) -> Option<String> {
    any_nonzero(&[
        p.open_n,
        p.doing_n,
        p.blocked_n,
        p.ready_n,
        p.doing_stale,
        p.past_triage,
    ])
    .then(|| {
        let mut s = format!(
            "- queue: ready {} of {} open \u{b7} doing {} ({} stale) \u{b7} blocked {}",
            p.ready_n, p.open_n, p.doing_n, p.doing_stale, p.blocked_n
        );
        if let Some(d) = p.open_age_med_d {
            let _ = write!(s, " \u{b7} open age med {}", days(d));
        }
        let _ = write!(s, " \u{b7} {} past 14d", p.past_triage);
        let named: Vec<String> = p
            .past_triage_groups
            .iter()
            .take(TRIAGE_GROUPS)
            .map(|(g, n)| format!("{g} {n}"))
            .collect();
        if !named.is_empty() {
            let _ = write!(s, ": {}", named.join(" \u{b7} "));
            if p.past_triage_groups.len() > TRIAGE_GROUPS {
                let _ = write!(s, " \u{b7} +{}", p.past_triage_groups.len() - TRIAGE_GROUPS);
            }
        }
        s
    })
}

fn graph(p: &Pulse) -> Option<String> {
    any_nonzero(&[
        p.lanes_multi,
        p.unlockers,
        p.needs_behind_n,
        p.blocked_foreign,
        p.blocked_unresolved,
        p.owed_foreign,
    ])
    .then(|| {
        let mut s = format!("- graph: {} lanes", p.lanes_multi);
        if let Some((id, n)) = &p.top_unlocker {
            let _ = write!(s, " \u{b7} unlocks most {id} ({n})");
        }
        let _ = write!(
            s,
            " \u{b7} needs-behind {} \u{b7} blocked on foreign {} \u{b7} unresolved {} \u{b7} owed to others {}",
            p.needs_behind_n, p.blocked_foreign, p.blocked_unresolved, p.owed_foreign
        );
        s
    })
}

fn asks(p: &Pulse) -> Option<String> {
    let half = |label: &str, n: i64, oldest: Option<f64>| {
        (n > 0).then(|| {
            let age = oldest.map_or(String::new(), |d| format!(" (oldest {})", days(d)));
            format!("{label} {n}{age}")
        })
    };
    let parts: Vec<String> = [
        half("owed to me", p.asks_in_open, p.asks_in_oldest_d),
        half("owed by me", p.asks_out_open, p.asks_out_oldest_d),
    ]
    .into_iter()
    .flatten()
    .collect();
    (!parts.is_empty()).then(|| format!("- asks: {}", parts.join(" \u{b7} ")))
}

fn friction(p: &Pulse, window_days: i64) -> Option<String> {
    any_nonzero(&[
        p.close_attempts_w,
        p.reopens_w,
        p.blocks_w,
        p.thrash_n,
        p.handoff_stale_n,
    ])
    .then(|| {
        let thrash = p
            .top_thrash
            .as_ref()
            .map_or(String::new(), |(id, n)| format!(" ({id}: {n} actors)"));
        format!(
            "- friction {window_days}d: close attempts {} \u{b7} reopens {} \u{b7} blocks {} \
             \u{b7} thrash {}{thrash} \u{b7} handoffs citing closed tasks {}",
            p.close_attempts_w, p.reopens_w, p.blocks_w, p.thrash_n, p.handoff_stale_n
        )
    })
}

/// The block, in order, clamped; a cut line keeps its label and says so.
pub(crate) fn lines(p: &Pulse, window_days: i64, cuts: &Cuts) -> Vec<String> {
    let cut = |line: Option<String>, label: &str, cut: bool| {
        line.map(|l| if cut { format!("- {label}: {CUT}") } else { l })
    };
    [
        flow(p, window_days),
        queue(p),
        cut(graph(p), "graph", cuts.graph),
        asks(p),
        cut(friction(p, window_days), "friction", cuts.friction),
    ]
    .into_iter()
    .flatten()
    .map(|l| clamp_bytes(&l, LINE_CLAMP))
    .collect()
}

/// `cites N closed tasks (ids…) — handoff may be stale`, for the next block.
pub(crate) fn cites_line(cited: &[String], repo: &str) -> Option<String> {
    if cited.is_empty() {
        return None;
    }
    let prefix = format!("{repo}#");
    let mut named: Vec<&str> = cited
        .iter()
        .take(3)
        .map(|g| g.strip_prefix(&prefix).unwrap_or(g))
        .collect();
    let more = cited.len().saturating_sub(3);
    let extra;
    if more > 0 {
        extra = format!("+{more}");
        named.push(&extra);
    }
    let noun = if cited.len() == 1 { "task" } else { "tasks" };
    Some(clamp_bytes(
        &format!(
            "  cites {} closed {noun} ({}) \u{2014} handoff may be stale",
            cited.len(),
            named.join(", ")
        ),
        LINE_CLAMP,
    ))
}

/// The row as JSON, the view's column names, plus the ids the lines name.
pub(crate) fn json(p: &Pulse) -> serde_json::Value {
    let pair = |v: &Option<(String, i64)>, key: &str| {
        v.as_ref()
            .map(|(id, n)| serde_json::json!({ "id": id, key: n }))
    };
    serde_json::json!({
        "open_n": p.open_n, "doing_n": p.doing_n, "blocked_n": p.blocked_n,
        "done_n": p.done_n, "dropped_n": p.dropped_n,
        "filed_w": p.filed_w, "filed_from_w": p.filed_from_w, "done_w": p.done_w,
        "dropped_w": p.dropped_w, "started_w": p.started_w, "backlog_delta_w": p.backlog_delta_w,
        "doing_stale": p.doing_stale, "open_age_med_d": p.open_age_med_d, "past_triage": p.past_triage,
        "past_triage_groups": p.past_triage_groups.iter()
            .map(|(g, n)| serde_json::json!({ "group": g, "n": n })).collect::<Vec<_>>(),
        "ready_n": p.ready_n, "unlockers": p.unlockers, "lanes_multi": p.lanes_multi,
        "needs_behind_n": p.needs_behind_n, "blocked_foreign": p.blocked_foreign,
        "blocked_unresolved": p.blocked_unresolved, "owed_foreign": p.owed_foreign,
        "top_unlocker": pair(&p.top_unlocker, "unlock"),
        "asks_in_open": p.asks_in_open, "asks_in_oldest_d": p.asks_in_oldest_d,
        "asks_out_open": p.asks_out_open, "asks_out_oldest_d": p.asks_out_oldest_d,
        "close_attempts_w": p.close_attempts_w, "reopens_w": p.reopens_w, "blocks_w": p.blocks_w,
        "thrash_n": p.thrash_n, "top_thrash": pair(&p.top_thrash, "actors"),
        "handoff_stale_n": p.handoff_stale_n,
    })
}

fn size(lines: &[String]) -> usize {
    lines.iter().map(|l| l.len() + 1).sum()
}

/// Build the digest under `budget` bytes, giving up the least first: the
/// inbox list collapses to a pointer that still carries its count, age
/// and verb; then also-ready down to the floor, then friction, then
/// graph. What still overflows after that is the caller's truncation tail.
pub(crate) fn fit(
    budget: usize,
    also_ready: usize,
    build: impl Fn(&Cuts) -> Vec<String>,
) -> Vec<String> {
    let mut cuts = Cuts {
        also_ready,
        ..Cuts::default()
    };
    let steps: [fn(&mut Cuts); 4] = [
        |c| c.inbox = true,
        |c| c.also_ready = c.also_ready.min(ALSO_READY_FLOOR),
        |c| c.friction = true,
        |c| c.graph = true,
    ];
    let mut lines = build(&cuts);
    for step in steps {
        if size(&lines) <= budget {
            break;
        }
        step(&mut cuts);
        lines = build(&cuts);
    }
    lines
}
