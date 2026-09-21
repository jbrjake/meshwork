//! The renderers behind `meshwork prime` (DESIGN §7b), split from
//! `prime.rs` at the 750-line ceiling: every section of the digest as
//! lines — headline counts, the category rollup, weather, the next-task
//! block, the inbox, also-ready, recently done, the advisory tail — and
//! the assembly that lays them out under a budget cut. `prime.rs` keeps
//! the run and the JSON emitter; nothing here reads the store or prints.

use crate::parse::{ParsedTask, Status, Task};
use crate::write::clamp_bytes;
use std::collections::{BTreeMap, BTreeSet};

/// Per-line clamp so one monster title can't eat the digest.
pub(super) const LINE_CLAMP: usize = 160;
/// Ready rows in the digest (DESIGN §7: top-10 = next + 9 also-ready).
pub(super) const READY_ROWS: usize = 10;
/// Rollup groups shown (§7b: top 5 by min seq; the rest is a loud +N).
pub(super) const ROLLUP_GROUPS: usize = 5;
/// Freshest frontier comments surfaced as weather.
const WEATHER_COMMENTS: usize = 4;
/// Body-head lines quoted in the next-task block.
const BODY_HEAD_LINES: usize = 3;
/// `handoff:` commentary lines rendered (voice, not a novel).
const VOICE_LINES: usize = 6;
/// Recently-done rows (§7b: last ~5, dated from log lines).
const DONE_ROWS: usize = 5;
/// Dependents named on a blocks-line before collapsing to +N.
const BLOCKS_NAMED: usize = 3;
/// Outbound asks listed before the footnote names `ready --all`.
const ASKS_OUT_ROWS: usize = 5;

/// A category's first two segments — `engine/spill/budget` rolls up into
/// `engine/spill` (§7b: display grain, not a model change).
fn group_of(category: &str) -> &str {
    crate::pulse::category_group(category)
}

fn is_live(status: Status) -> bool {
    matches!(status, Status::Open | Status::Doing | Status::Blocked)
}

/// Open dependents of `id` — what this task unblocks, for the blocks-line.
fn dependents<'a>(tasks: &[&'a Task], id: &str) -> Vec<&'a str> {
    tasks
        .iter()
        .filter(|t| is_live(t.status) && t.needs.iter().any(|n| n == id))
        .map(|t| t.id.as_str())
        .collect()
}

fn blocks_suffix(deps: &[&str]) -> String {
    if deps.is_empty() {
        return String::new();
    }
    let named: Vec<&str> = deps.iter().take(BLOCKS_NAMED).copied().collect();
    let extra = deps.len().saturating_sub(BLOCKS_NAMED);
    let more = if extra > 0 {
        format!(" +{extra}")
    } else {
        String::new()
    };
    format!("blocks: {}{}", named.join(", "), more)
}

/// Done-date from the last `→done` transition, read through the normative
/// log grammar (mw-3wnhhvp) — the de-facto substring parse is retired.
fn done_date(t: &Task) -> Option<String> {
    t.log.iter().rev().find_map(|l| {
        let e = crate::parse::parse_log_line(l);
        if e.to == Some(Status::Done) {
            e.date
        } else {
            None
        }
    })
}

/// Live tasks grouped by first-two category segments, ranked by min seq
/// (seq IS the priority primitive — there is no priority field), plus the
/// rendered line: top-5 groups, the rest a loud `+N more` (MW-D2).
pub(super) fn rollup<'a>(tasks: &[&'a Task]) -> (Vec<(&'a str, i64, usize)>, Option<String>) {
    let mut groups: BTreeMap<&str, (i64, usize)> = BTreeMap::new();
    for t in tasks.iter().filter(|t| is_live(t.status)) {
        if let Some(cat) = t.category.as_deref() {
            let entry = groups.entry(group_of(cat)).or_insert((i64::MAX, 0));
            entry.0 = entry.0.min(t.seq.unwrap_or(i64::MAX));
            entry.1 += 1;
        }
    }
    let mut ranked: Vec<(&str, i64, usize)> =
        groups.into_iter().map(|(g, (s, n))| (g, s, n)).collect();
    ranked.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(b.0)));
    let line = if ranked.is_empty() {
        None
    } else {
        let mut parts: Vec<String> = ranked
            .iter()
            .take(ROLLUP_GROUPS)
            .map(|(g, _, n)| format!("{g} {n}"))
            .collect();
        if ranked.len() > ROLLUP_GROUPS {
            parts.push(format!("+{} more", ranked.len() - ROLLUP_GROUPS));
        }
        Some(parts.join(" \u{b7} "))
    };
    (ranked, line)
}

/// The newest log entry that tells a session something — pure provenance
/// stamps (`imported from …`, bare `created`) are skipped, and a task with
/// nothing else gets no tail at all (mw-drrvpsg: the sazed pilot spent 8
/// weather lines on the identical import stamp).
fn substantive_log_tail(t: &Task) -> Option<&String> {
    t.log.iter().rev().find(|l| {
        let text = l
            .split_once(' ')
            .map_or(l.as_str(), |(_, rest)| rest)
            .trim();
        text != "created" && !text.starts_with("imported from ")
    })
}

/// Weather — all derived, never stored: doing with newest substantive log,
/// blocked with reasons, freshest comments across the active frontier (§7b).
pub(super) fn weather_lines(tasks: &[&Task], ready_ids: &BTreeSet<&str>) -> Vec<String> {
    let mut out = Vec::new();
    let today = crate::clock::today();
    for t in tasks.iter().filter(|t| t.status == Status::Doing) {
        let claim = t
            .claimed_by
            .as_deref()
            .map_or(String::new(), |c| format!(" [claimed: {c}]"));
        // mw-06j1wqe: age the rot in place — the digest must not keep
        // normalizing a doing list that only ever grows.
        let stale = t
            .last_activity_date()
            .and_then(|d| crate::clock::days_between(&d, &today))
            .filter(|a| *a >= crate::lint::STALE_DOING_DAYS)
            .map_or(String::new(), |a| format!(" [stale: {a}d]"));
        let tail = substantive_log_tail(t).map_or(String::new(), |l| format!(" \u{2014} {l}"));
        out.push(clamp_bytes(
            &format!("- doing {} {}{claim}{stale}{}", t.id, t.title, tail),
            LINE_CLAMP,
        ));
    }
    for t in tasks.iter().filter(|t| t.status == Status::Blocked) {
        let reason = t.blocked_reason.as_deref().unwrap_or_default();
        out.push(clamp_bytes(
            &format!("- blocked {} {} \u{2014} {reason}", t.id, t.title),
            LINE_CLAMP,
        ));
    }
    let mut fresh: Vec<(&str, &str, &str)> = tasks
        .iter()
        .filter(|t| {
            t.status == Status::Doing
                || t.status == Status::Blocked
                || ready_ids.contains(t.id.as_str())
        })
        .flat_map(|t| {
            t.comments.iter().map(|c| {
                (
                    c.date.as_str(),
                    t.id.as_str(),
                    c.text.lines().next().unwrap_or(""),
                )
            })
        })
        .collect();
    fresh.sort_by(|a, b| b.0.cmp(a.0).then(a.1.cmp(b.1)));
    for (date, id, text) in fresh.iter().take(WEATHER_COMMENTS) {
        out.push(clamp_bytes(&format!("- {date} {id}: {text}"), LINE_CLAMP));
    }
    out
}

/// Who wrote the voice, and how long ago (MW-S10): the newest minted
/// `<stamp> handoff [by <author>]` log line, rendered
/// `[handoff by <author>, Nd]`; a block with no such line — hand-written,
/// or older than the minting — renders `[handoff: unstamped]`, so the
/// reader knows it is looking at prose of unknown age.
fn handoff_tag(t: &Task) -> String {
    let minted = t.log.iter().rev().find_map(|entry| {
        let (stamp, rest) = entry.split_once(' ')?;
        let rest = rest.trim();
        if rest == "handoff" {
            Some((stamp, None))
        } else {
            rest.strip_prefix("handoff by ")
                .map(|a| (stamp, Some(a.trim())))
        }
    });
    let Some((stamp, author)) = minted else {
        return "  [handoff: unstamped]".to_string();
    };
    let age = crate::clock::days_between(stamp, &crate::clock::today())
        .map(|d| format!(", {}d", d.max(0)))
        .unwrap_or_default();
    match author {
        Some(a) => format!("  [handoff by {a}{age}]"),
        None => format!("  [handoff{age}]"),
    }
}

/// The next-task block: the `handoff:` voice first, mechanics after (§7b);
/// `cited` names the closed tasks the voice still refers to (MW-S6).
pub(super) fn next_block_lines(
    tasks: &[&Task],
    ready: &[Vec<String>],
    cited: &[String],
    repo: &str,
) -> Vec<String> {
    let mut out = Vec::new();
    let Some(row) = ready.first() else {
        return out;
    };
    let Some(t) = tasks.iter().find(|t| t.id == row[0]) else {
        return out;
    };
    out.push(clamp_bytes(
        &format!("next \u{2192} {} {}", row[0], row[1]),
        LINE_CLAMP,
    ));
    if let Some(voice) = t.handoff.as_deref() {
        for line in voice
            .lines()
            .filter(|l| !l.trim().is_empty())
            .take(VOICE_LINES)
        {
            out.push(clamp_bytes(
                &format!("  \u{bb} {}", line.trim()),
                LINE_CLAMP,
            ));
        }
        out.push(clamp_bytes(&handoff_tag(t), LINE_CLAMP));
        out.extend(super::pulse::cites_line(cited, repo));
    }
    let deps = dependents(tasks, &t.id);
    let mut meta: Vec<String> = Vec::new();
    if let Some(cat) = t.category.as_deref() {
        meta.push(format!("[{cat}]"));
    }
    if !deps.is_empty() {
        meta.push(blocks_suffix(&deps));
    }
    if !meta.is_empty() {
        out.push(clamp_bytes(
            &format!("  {}", meta.join(" \u{b7} ")),
            LINE_CLAMP,
        ));
    }
    if let Some(v) = t.verify.as_deref().filter(|v| !v.trim().is_empty()) {
        out.push(clamp_bytes(&format!("  verify: {v}"), LINE_CLAMP));
    } else {
        // mw-6wdpz1b: the gap is the next action, never silence — start
        // refuses until the done-test exists.
        out.push(
            "  verify: (unset \u{2014} needs-verify: write the done-test, then start)".to_string(),
        );
    }
    if !t.docs.is_empty() {
        out.push(clamp_bytes(
            &format!("  docs: {}", t.docs.join(" \u{b7} ")),
            LINE_CLAMP,
        ));
    }
    for line in t
        .description
        .lines()
        .filter(|l| !l.trim().is_empty())
        .take(BODY_HEAD_LINES)
    {
        out.push(clamp_bytes(&format!("  | {line}"), LINE_CLAMP));
    }
    let tail_n = t.comments.len().min(2);
    if tail_n > 0 {
        out.push(format!("  comments (last {tail_n}):"));
        for c in t.comments.iter().rev().take(2).rev() {
            let first = c.text.lines().next().unwrap_or("");
            out.push(clamp_bytes(
                &format!("  - {} [{}] {}", c.date, c.author, first),
                LINE_CLAMP,
            ));
        }
    }
    out
}

/// Incoming asks (mw-hfvtx0s) — between weather and next: they inform
/// the session before it commits to a task, but never displace next.
/// Each row carries its age; the headline carries the oldest.
fn inbox_lines(
    inbox: &[crate::addressed::Ask],
    today: &str,
    repo: &str,
    collapsed: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    if inbox.is_empty() {
        return out;
    }
    // Whole, or a pointer (mw-0a084qy): every ask while it fits; when it
    // cannot, the count, the oldest age and the exact statement that
    // lists them — never a `… and N more` with nothing to run. The
    // statement is not clamped: cut, it would run nothing.
    if collapsed {
        let s = if inbox.len() == 1 { "" } else { "s" };
        let oldest = crate::addressed::oldest_age_days(inbox, today)
            .map_or(String::new(), |d| format!(", oldest {d}d"));
        out.push(format!(
            "addressed to this repo: {} ask{s}{oldest} \u{2014} {}",
            inbox.len(),
            crate::addressed::list_statement(repo)
        ));
        return out;
    }
    out.push(format!("addressed to this repo ({}):", inbox.len()));
    for a in inbox {
        let age = crate::addressed::age_suffix(a, today);
        let answered = crate::addressed::answered_suffix(a.answer.as_ref(), " \u{b7} ");
        out.push(clamp_bytes(
            &format!("- {} {}{age}{answered}", a.gid, a.title),
            LINE_CLAMP,
        ));
    }
    out
}

/// This repo's own asks (MW-L5) — owed elsewhere, so out of the worklist
/// and listed apart with the addressee, the age and the answer's state.
/// Cut with the inbox: one line with the count, the oldest age and the
/// verb that lists them.
fn asks_out_lines(
    asks: &[crate::addressed::Outbound],
    today: &str,
    collapsed: bool,
) -> Vec<String> {
    let mut out = Vec::new();
    if asks.is_empty() {
        return out;
    }
    if collapsed {
        let oldest = asks
            .iter()
            .filter_map(|a| a.age_days(today))
            .max()
            .map_or(String::new(), |d| format!(", oldest {d}d"));
        out.push(format!(
            "asks out: {}{oldest} \u{2014} ready --all",
            asks.len()
        ));
        return out;
    }
    out.push(format!("asks out ({}):", asks.len()));
    for a in asks.iter().take(ASKS_OUT_ROWS) {
        let age = a.age_suffix(today);
        let answered = crate::addressed::answered_suffix(a.answer.as_ref(), " \u{b7} ");
        out.push(clamp_bytes(
            &format!("- {} \u{2192} {}{age} {}{answered}", a.id, a.to, a.title),
            LINE_CLAMP,
        ));
    }
    if asks.len() > ASKS_OUT_ROWS {
        out.push(format!(
            "\u{2026} and {} more (ready --all)",
            asks.len() - ASKS_OUT_ROWS
        ));
    }
    out
}

/// Also-ready one-liners with blocks-lines.
pub(super) fn also_ready_lines(tasks: &[&Task], ready: &[Vec<String>]) -> Vec<String> {
    let mut out = Vec::new();
    for r in ready.iter().take(READY_ROWS).skip(1) {
        let deps = dependents(tasks, &r[0]);
        let mut suffix = if deps.is_empty() {
            String::new()
        } else {
            format!(" \u{2192} {}", blocks_suffix(&deps))
        };
        if r[3].is_empty() {
            suffix.push_str(" [needs-verify]"); // mw-6wdpz1b: loud, never hidden
        }
        out.push(clamp_bytes(
            &format!("- {} {}{}", r[0], r[1], suffix),
            LINE_CLAMP,
        ));
    }
    out
}

/// Recently done, newest first, dated from `→done` log lines.
pub(super) fn recent_dones<'a>(tasks: &[&'a Task]) -> Vec<(String, &'a str, &'a str)> {
    let mut dones: Vec<(String, &str, &str)> = tasks
        .iter()
        .filter(|t| t.status == Status::Done)
        .filter_map(|t| done_date(t).map(|d| (d, t.id.as_str(), t.title.as_str())))
        .collect();
    dones.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
    dones.truncate(DONE_ROWS);
    dones
}

pub(super) fn counts_line(counts: &BTreeMap<&str, usize>, invalid: usize, repo: &str) -> String {
    let mut parts: Vec<String> = Vec::new();
    for key in ["open", "doing", "blocked", "done", "dropped"] {
        if let Some(n) = counts.get(key) {
            parts.push(format!("{n} {key}"));
        }
    }
    if invalid > 0 {
        parts.push(format!("{invalid} invalid"));
    }
    format!("{repo} — {}", parts.join(", "))
}

/// The `!` advisory tail: invalid files BY NAME (mw-p6atpxh — a bare
/// count sat unactioned for two sessions while the broken task vanished
/// from ready), plus verifies edited after this clone approved them
/// (mw-yyf1bab) — surfaced before the session commits to a task; lint
/// carries the detail.
pub(super) fn advisory_lines(
    store: &crate::store::RepoStore,
    tasks: &[&Task],
    invalid: &[&str],
) -> Vec<String> {
    let mut out = Vec::new();
    if !invalid.is_empty() {
        out.push(clamp_bytes(
            &format!(
                "! {} invalid: {} \u{2014} run lint",
                invalid.len(),
                invalid.join(", ")
            ),
            LINE_CLAMP,
        ));
    }
    let changed = crate::lint_verify::changed_since_approval(store, tasks);
    if !changed.is_empty() {
        let ids: Vec<&str> = changed.iter().map(|(t, _)| t.id.as_str()).collect();
        out.push(clamp_bytes(
            &format!(
                "! verify changed since approval: {} \u{2014} lint shows the diff",
                ids.join(", ")
            ),
            LINE_CLAMP,
        ));
    }
    out
}

/// Recovered ids of unparseable files, so the advisory names what it
/// counts (mw-p6atpxh).
pub(super) fn invalid_ids(store: &crate::store::RepoStore) -> Vec<&str> {
    store
        .entries
        .iter()
        .filter_map(|e| match &e.parsed {
            ParsedTask::Invalid(inv) => Some(inv.id.as_str()),
            ParsedTask::Valid(_) => None,
        })
        .collect()
}

/// The digest's rendered pieces, assembled once per budget cut.
pub(super) struct Digest<'a> {
    pub(super) headline: String,
    pub(super) provenance: Option<String>,
    pub(super) rollup: Option<String>,
    pub(super) pulse: &'a crate::pulse::Pulse,
    pub(super) window_days: i64,
    pub(super) weather: &'a [String],
    pub(super) inbox: &'a [crate::addressed::Ask],
    pub(super) asks_out: &'a [crate::addressed::Outbound],
    pub(super) today: &'a str,
    pub(super) repo: &'a str,
    pub(super) next_block: &'a [String],
    pub(super) also_ready: &'a [String],
    pub(super) ready_total: usize,
    pub(super) dones: &'a [(String, &'a str, &'a str)],
    pub(super) advisories: Vec<String>,
}

/// Every line of the digest in order, under the given cuts: the pulse
/// leads the weather; also-ready shows what the cut allows and says so.
pub(super) fn assemble(d: &Digest, cuts: &super::pulse::Cuts) -> Vec<String> {
    let mut lines: Vec<String> = vec![d.headline.clone()];
    lines.extend(d.provenance.clone());
    lines.extend(d.rollup.clone());
    let mut pulse_lines = super::pulse::lines(d.pulse, d.window_days, cuts);
    if !pulse_lines.is_empty() || !d.weather.is_empty() {
        lines.push("weather:".to_string());
        lines.append(&mut pulse_lines);
        lines.extend(d.weather.iter().cloned());
    }
    lines.append(&mut inbox_lines(d.inbox, d.today, d.repo, cuts.inbox));
    lines.append(&mut asks_out_lines(d.asks_out, d.today, cuts.inbox));
    lines.extend(d.next_block.iter().cloned());
    let shown = d.also_ready.len().min(cuts.also_ready);
    if shown > 0 {
        let budgeted = if shown < d.also_ready.len() {
            " \u{2014} 6KB budget"
        } else {
            ""
        };
        lines.push(format!(
            "also ready ({} more, top {shown}{budgeted}):",
            d.ready_total.saturating_sub(1)
        ));
        lines.extend(d.also_ready.iter().take(shown).cloned());
    }
    if d.ready_total > shown + 1 {
        lines.push(format!(
            "\u{2026} and {} more (ready --all)",
            d.ready_total - shown - 1
        ));
    }
    if !d.dones.is_empty() {
        lines.push("recently done:".to_string());
        for (date, id, title) in d.dones {
            lines.push(clamp_bytes(&format!("- {date} {id} {title}"), LINE_CLAMP));
        }
    }
    lines.extend(d.advisories.iter().cloned());
    lines
}
