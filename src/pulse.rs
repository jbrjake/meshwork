//! The `pulse` row in Rust (FORMAT.md §Views, `pulse`): the columns the
//! `prime` block renders, computed over the loaded stores under the same
//! clock the SQL session registers. `prime` is gated at 100 ms cold; a
//! query over the pulse costs hundreds of milliseconds to plan, so the
//! block is computed here from the tasks already parsed and the published
//! SQL stays the specification — `e2e::prime_pulse_matches_view` holds the
//! two equal, column by column, on every fixture store.
//!
//! Only the rendered columns live here. The rest of the row (`no_verify`,
//! lineage, mention totals) is a query away and nothing in `prime` reads it.

use crate::facts::{median, round1, TaskFacts, STALE_DOING_H, TRIAGE_H};
use crate::graph::GraphRow;
use crate::parse::{ParsedTask, Status};
use crate::registry::ForeignTask;
use crate::store::RepoStore;
use crate::views::Clock;
use std::collections::{BTreeMap, BTreeSet};

/// Category groups named on the queue line before collapsing to `+N`.
pub const TRIAGE_GROUPS: usize = 3;

/// The pulse columns `prime` renders, plus the ids the lines name.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Pulse {
    /// The repo this row describes.
    pub repo: String,
    /// Tasks currently open.
    pub open_n: i64,
    /// Tasks currently doing.
    pub doing_n: i64,
    /// Tasks currently blocked.
    pub blocked_n: i64,
    /// Tasks currently done.
    pub done_n: i64,
    /// Tasks currently dropped.
    pub dropped_n: i64,
    /// Created in the window.
    pub filed_w: i64,
    /// Of those, carrying `discovered-from`.
    pub filed_from_w: i64,
    /// Done, and closed in the window.
    pub done_w: i64,
    /// Dropped, and dropped in the window.
    pub dropped_w: i64,
    /// Distinct tasks that entered `doing` in the window.
    pub started_w: i64,
    /// `filed_w - done_w - dropped_w`.
    pub backlog_delta_w: i64,
    /// `doing` and idle 72 h or more.
    pub doing_stale: i64,
    /// Median age of open tasks, days, one decimal; `None` with no open task.
    pub open_age_med_d: Option<f64>,
    /// Live and 14 days old or more.
    pub past_triage: i64,
    /// Those, grouped by category (first two segments; `(none)`), most first.
    pub past_triage_groups: Vec<(String, i64)>,
    /// Ready by the graph view's predicate.
    pub ready_n: i64,
    /// Tasks something live transitively needs.
    pub unlockers: i64,
    /// Lanes with more than one member.
    pub lanes_multi: i64,
    /// Prerequisites placed behind a dependent.
    pub needs_behind_n: i64,
    /// Live tasks with an unmet need in another repo.
    pub blocked_foreign: i64,
    /// Live tasks with a need the loaded set cannot resolve.
    pub blocked_unresolved: i64,
    /// Live tasks another repo's live task needs.
    pub owed_foreign: i64,
    /// The task that unlocks the most, with its count.
    pub top_unlocker: Option<(String, i64)>,
    /// Asks addressed to this repo, owed by the view's rule.
    pub asks_in_open: i64,
    /// The oldest of those, in days, one decimal.
    pub asks_in_oldest_d: Option<f64>,
    /// This repo's own asks nobody has answered with a done task.
    pub asks_out_open: i64,
    /// The oldest of those, in days, one decimal.
    pub asks_out_oldest_d: Option<f64>,
    /// Close attempts logged in the window.
    pub close_attempts_w: i64,
    /// Transitions back to open in the window.
    pub reopens_w: i64,
    /// Transitions into blocked in the window.
    pub blocks_w: i64,
    /// Live tasks two or more actors touched since they last moved.
    pub thrash_n: i64,
    /// The most-touched of those, with its actor count.
    pub top_thrash: Option<(String, i64)>,
    /// Live tasks whose handoff still names a closed task.
    pub handoff_stale_n: i64,
}

/// The rollup key of a category: its first two segments.
#[must_use]
pub fn category_group(category: &str) -> &str {
    let mut end = category.len();
    for (n, (i, _)) in category.match_indices('/').enumerate() {
        if n == 1 {
            end = i;
            break;
        }
    }
    &category[..end]
}

/// Tokens of `text` on the alphabet `[a-z0-9#-]` — the mentions view's
/// tokenizer, everything else a separator.
fn tokens(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '#' || c == '-'))
        .filter(|t| !t.is_empty())
}

/// The gids `text` mentions, in first-appearance order: a token that is
/// a loaded gid, or one the mentioning task's own `repo#` completes —
/// never the task itself.
#[must_use]
pub fn mentioned_gids(
    text: &str,
    repo: &str,
    own_gid: &str,
    known: &BTreeSet<String>,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for tok in tokens(text) {
        let gid = if tok.contains('#') {
            tok.to_string()
        } else {
            format!("{repo}#{tok}")
        };
        if gid != own_gid && known.contains(&gid) && !out.contains(&gid) {
            out.push(gid);
        }
    }
    out
}

/// Every `tasks` row's status by gid — valid, invalid and foreign alike,
/// the set a mention resolves against.
#[must_use]
pub fn statuses(stores: &[RepoStore], foreign: &[ForeignTask]) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for store in stores {
        for entry in &store.entries {
            match &entry.parsed {
                ParsedTask::Valid(t) => {
                    out.insert(store.gid(&t.id), t.status.as_str().to_string());
                }
                ParsedTask::Invalid(inv) => {
                    out.insert(store.gid(&inv.id), "invalid".to_string());
                }
            }
        }
    }
    for f in foreign {
        out.insert(f.gid.clone(), f.status.clone());
    }
    out
}

fn is_terminal(status: &str) -> bool {
    matches!(status, "done" | "dropped")
}

/// The closed tasks `text` still names — `mentions` with a terminal
/// `ref_status` — as gids in first-appearance order.
#[must_use]
pub fn cited_closed(
    text: &str,
    repo: &str,
    own_gid: &str,
    statuses: &BTreeMap<String, String>,
) -> Vec<String> {
    let known: BTreeSet<String> = statuses.keys().cloned().collect();
    mentioned_gids(text, repo, own_gid, &known)
        .into_iter()
        .filter(|g| statuses.get(g).is_some_and(|s| is_terminal(s)))
        .collect()
}

/// The asks half of the row for `repo` over these stores, by the view's
/// rule: an ask is owed while it is non-terminal and no `done` task
/// answers it (a dropped answer never counts; an open one is an intent).
/// Its age is the view's `age_h` for a live task — creation to now.
/// Returns `(in_open, in_oldest_d, out_open, out_oldest_d)`.
#[must_use]
pub fn asks(
    stores: &[RepoStore],
    clock: &Clock,
    repo: &str,
) -> (i64, Option<f64>, i64, Option<f64>) {
    let mut done_answers: BTreeSet<String> = BTreeSet::new();
    for store in stores {
        for entry in &store.entries {
            if let ParsedTask::Valid(t) = &entry.parsed {
                if t.status == Status::Done {
                    if let Some(a) = &t.answers {
                        done_answers.insert(crate::tables::qualify_ref(store, a));
                    }
                }
            }
        }
    }
    let (mut in_n, mut out_n) = (0, 0);
    let (mut in_max, mut out_max): (Option<f64>, Option<f64>) = (None, None);
    for store in stores {
        for entry in &store.entries {
            let ParsedTask::Valid(t) = &entry.parsed else {
                continue;
            };
            let Some(to) = t.to.as_deref() else {
                continue;
            };
            let gid = store.gid(&t.id);
            if matches!(t.status, Status::Done | Status::Dropped) || done_answers.contains(&gid) {
                continue;
            }
            let age = t
                .created
                .as_deref()
                .and_then(crate::views::stamp_secs)
                .map(|c| crate::facts::hours(c, clock.now_secs));
            let to_repo = to.split('#').next().unwrap_or(to);
            if to_repo == repo {
                in_n += 1;
                in_max = in_max.into_iter().chain(age).reduce(f64::max);
            }
            if store.repo == repo {
                out_n += 1;
                out_max = out_max.into_iter().chain(age).reduce(f64::max);
            }
        }
    }
    let days = |h: Option<f64>| h.map(|h| round1(h / 24.0));
    (in_n, days(in_max), out_n, days(out_max))
}

fn count<T>(items: &[T], pick: impl Fn(&T) -> bool) -> i64 {
    i64::try_from(items.iter().filter(|x| pick(x)).count()).unwrap_or(i64::MAX)
}

/// `p_refs`: every task of `repo`, live or not, whose handoff names a
/// closed task — the SQL counts the source whatever its status.
fn handoff_stale(stores: &[RepoStore], repo: &str, statuses: &BTreeMap<String, String>) -> i64 {
    let mut n = 0;
    for store in stores.iter().filter(|s| s.repo == repo) {
        for entry in &store.entries {
            if let ParsedTask::Valid(t) = &entry.parsed {
                if let Some(h) = &t.handoff {
                    if !cited_closed(h, &store.repo, &store.gid(&t.id), statuses).is_empty() {
                        n += 1;
                    }
                }
            }
        }
    }
    n
}

/// The past-triage tasks by category group, most first, then by name.
fn triage_groups<'a>(past: impl Iterator<Item = &'a TaskFacts>) -> Vec<(String, i64)> {
    let mut groups: BTreeMap<&str, i64> = BTreeMap::new();
    for f in past {
        *groups
            .entry(f.category.as_deref().map_or("(none)", category_group))
            .or_default() += 1;
    }
    let mut out: Vec<(String, i64)> = groups
        .into_iter()
        .map(|(g, n)| (g.to_string(), n))
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

/// The row for `repo` over these stores — the same inputs the query
/// session sees (`tables::session_for`): loaded stores plus the foreign
/// thin rows.
#[must_use]
pub fn compute(stores: &[RepoStore], foreign: &[ForeignTask], clock: &Clock, repo: &str) -> Pulse {
    let all = crate::facts::compute(stores, clock);
    let graph = crate::graph::compute(stores, foreign);
    let statuses = statuses(stores, foreign);
    let mine: Vec<&TaskFacts> = all.iter().filter(|f| f.repo == repo).collect();
    let rows: Vec<&GraphRow> = graph.iter().filter(|r| r.repo == repo).collect();
    let since = clock.now_secs - clock.window_days * 86_400;

    let by_status = |s: Status| count(&mine, |f| f.status == s);
    let mut open_ages: Vec<f64> = mine
        .iter()
        .filter(|f| f.status == Status::Open)
        .filter_map(|f| f.age_h)
        .collect();
    let stale_doing =
        |f: &&TaskFacts| f.status == Status::Doing && f.idle_h.is_some_and(|h| h >= STALE_DOING_H);
    let past = |f: &&TaskFacts| f.live && f.age_h.is_some_and(|h| h >= TRIAGE_H);
    let past_triage_groups = triage_groups(mine.iter().filter(|f| past(f)).copied());

    let filed_w = count(&mine, |f| f.filed_w);
    let done_w = count(&mine, |f| {
        f.status == Status::Done && f.terminal_at.is_some_and(|t| t >= since)
    });
    let dropped_w = count(&mine, |f| {
        f.status == Status::Dropped && f.terminal_at.is_some_and(|t| t >= since)
    });

    let live_row = |r: &&GraphRow| matches!(r.status.as_str(), "open" | "doing" | "blocked");
    let lanes: BTreeSet<&str> = rows
        .iter()
        .filter(|r| r.lane_size > 1)
        .filter_map(|r| r.lane.as_deref())
        .collect();
    let top_unlocker = rows
        .iter()
        .filter(|r| r.unlock > 0)
        .max_by(|a, b| a.unlock.cmp(&b.unlock).then_with(|| b.id.cmp(&a.id)))
        .map(|r| (r.id.clone(), r.unlock));
    let top_thrash = mine
        .iter()
        .filter(|f| f.live && f.touched_since_move >= 2)
        .max_by(|a, b| {
            a.touched_since_move
                .cmp(&b.touched_since_move)
                .then_with(|| b.id.cmp(&a.id))
        })
        .map(|f| (f.id.clone(), f.touched_since_move));

    let handoff_stale_n = handoff_stale(stores, repo, &statuses);
    let (asks_in_open, asks_in_oldest_d, asks_out_open, asks_out_oldest_d) =
        asks(stores, clock, repo);

    Pulse {
        repo: repo.to_string(),
        open_n: by_status(Status::Open),
        doing_n: by_status(Status::Doing),
        blocked_n: by_status(Status::Blocked),
        done_n: by_status(Status::Done),
        dropped_n: by_status(Status::Dropped),
        filed_w,
        filed_from_w: count(&mine, |f| f.filed_w && f.discovered_from),
        done_w,
        dropped_w,
        started_w: count(&mine, |f| f.started_w),
        backlog_delta_w: filed_w - done_w - dropped_w,
        doing_stale: count(&mine, stale_doing),
        open_age_med_d: median(&mut open_ages).map(|h| round1(h / 24.0)),
        past_triage: count(&mine, past),
        past_triage_groups,
        ready_n: count(&rows, |r| r.ready),
        unlockers: count(&rows, |r| r.unlock > 0),
        lanes_multi: i64::try_from(lanes.len()).unwrap_or(i64::MAX),
        needs_behind_n: count(&rows, |r| r.needs_behind),
        blocked_foreign: count(&rows, |r| live_row(r) && r.needs_open_foreign > 0),
        blocked_unresolved: count(&rows, |r| live_row(r) && r.needs_unresolved > 0),
        owed_foreign: count(&rows, |r| live_row(r) && r.needed_by_foreign > 0),
        top_unlocker,
        asks_in_open,
        asks_in_oldest_d,
        asks_out_open,
        asks_out_oldest_d,
        close_attempts_w: mine.iter().map(|f| f.close_attempts_w).sum(),
        reopens_w: mine.iter().map(|f| f.reopens_w).sum(),
        blocks_w: mine.iter().map(|f| f.blocks_w).sum(),
        thrash_n: count(&mine, |f| f.live && f.touched_since_move >= 2),
        top_thrash,
        handoff_stale_n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_take_two_segments() {
        assert_eq!(category_group("engine/spill/budget"), "engine/spill");
        assert_eq!(category_group("engine/spill"), "engine/spill");
        assert_eq!(category_group("docs"), "docs");
    }

    #[test]
    fn mentions_tokenize_like_the_view() {
        let known: BTreeSet<String> = ["a#x-1", "a#y-2", "b#z-3"]
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        let got = mentioned_gids(
            "Start from x-1, then b#z-3; X-1 again (y-2).",
            "a",
            "a#y-2",
            &known,
        );
        assert_eq!(got, ["a#x-1", "b#z-3"]);
        assert!(mentioned_gids("nothing here", "a", "a#q", &known).is_empty());
    }
}
