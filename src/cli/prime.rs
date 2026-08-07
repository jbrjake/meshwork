//! `meshwork prime` (PLAN 1.5 + mw-a8tv; MW-D3/D5, DESIGN §7b): the ≤6KB
//! session-start digest, now the materialized handoff. Headline counts +
//! top-5 category rollup by min seq; weather derived from the active
//! frontier (doing with last log, blocked with reasons, freshest comments);
//! the next task led by its `handoff:` commentary; also-ready with
//! blocks-lines; recently done from dated log lines. Bytes enforced,
//! truncation loud. Hand-written HANDOFF files are dead — this is the view.

use crate::parse::{ParsedTask, Status, Task};
use crate::store::load_repo;
use crate::write::clamp_bytes;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// The whole digest budget (MW-D3: 6KB ≈ 1.5K tokens at 4 bytes/token).
const BUDGET: usize = 6144;
/// Per-line clamp so one monster title can't eat the digest.
const LINE_CLAMP: usize = 160;
/// Ready rows in the digest (DESIGN §7: top-10 = next + 9 also-ready).
const READY_ROWS: usize = 10;
/// Rollup groups shown (§7b: top 5 by min seq; the rest is a loud +N).
const ROLLUP_GROUPS: usize = 5;
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
/// Visible marker when the budget forces a cut.
const TAIL: &str = "… truncated (6KB budget, MW-D3)";

/// Store provenance, one line (mw-3jwwh5d): HEAD short-sha, uncommitted
/// task-file edits, commits ahead of upstream — status and rev-list scoped
/// to docs/meshwork/, local refs only (zero network, MW-J6). Any git
/// failure (no repo, unborn HEAD, no upstream) degrades to omission —
/// the digest never fails over a nicety (MW-D5).
fn provenance_line(root: &std::path::Path) -> Option<String> {
    let git = |args: &[&str]| -> Option<String> {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    let sha = git(&["rev-parse", "--short", "HEAD"])?;
    let mut line = format!("store @ {sha}");
    let dirty =
        git(&["status", "--porcelain", "--", "docs/meshwork"]).map_or(0, |s| s.lines().count());
    if dirty > 0 {
        let s = if dirty == 1 { "" } else { "s" };
        let _ = write!(line, " \u{b7} {dirty} uncommitted task edit{s}");
    }
    if let Some(ahead) = git(&[
        "rev-list",
        "--count",
        "@{upstream}..HEAD",
        "--",
        "docs/meshwork",
    ])
    .and_then(|s| s.parse::<u64>().ok())
    .filter(|n| *n > 0)
    {
        let _ = write!(line, " \u{b7} {ahead} ahead of upstream");
    }
    Some(line)
}

/// A category's first two segments — `engine/spill/budget` rolls up into
/// `engine/spill` (§7b: display grain, not a model change).
fn group_of(category: &str) -> &str {
    match category.match_indices('/').nth(1) {
        Some((i, _)) => &category[..i],
        None => category,
    }
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
fn rollup<'a>(tasks: &[&'a Task]) -> (Vec<(&'a str, i64, usize)>, Option<String>) {
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

/// Weather — all derived, never stored: doing with last log, blocked with
/// reasons, freshest comments across the active frontier (§7b).
fn weather_lines(tasks: &[&Task], ready_ids: &BTreeSet<&str>) -> Vec<String> {
    let mut out = Vec::new();
    for t in tasks.iter().filter(|t| t.status == Status::Doing) {
        let claim = t
            .claimed_by
            .as_deref()
            .map_or(String::new(), |c| format!(" [claimed: {c}]"));
        let tail = t
            .log
            .last()
            .map_or(String::new(), |l| format!(" \u{2014} {l}"));
        out.push(clamp_bytes(
            &format!("- doing {} {}{claim}{}", t.id, t.title, tail),
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

/// The next-task block: the `handoff:` voice first, mechanics after (§7b).
fn next_block_lines(tasks: &[&Task], ready: &[Vec<String>]) -> Vec<String> {
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
    if let Some(v) = t.verify.as_deref() {
        out.push(clamp_bytes(&format!("  verify: {v}"), LINE_CLAMP));
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

/// Also-ready one-liners with blocks-lines.
fn also_ready_lines(tasks: &[&Task], ready: &[Vec<String>]) -> Vec<String> {
    let mut out = Vec::new();
    for r in ready.iter().take(READY_ROWS).skip(1) {
        let deps = dependents(tasks, &r[0]);
        let suffix = if deps.is_empty() {
            String::new()
        } else {
            format!(" \u{2192} {}", blocks_suffix(&deps))
        };
        out.push(clamp_bytes(
            &format!("- {} {}{}", r[0], r[1], suffix),
            LINE_CLAMP,
        ));
    }
    out
}

/// Recently done, newest first, dated from `→done` log lines.
fn recent_dones<'a>(tasks: &[&'a Task]) -> Vec<(String, &'a str, &'a str)> {
    let mut dones: Vec<(String, &str, &str)> = tasks
        .iter()
        .filter(|t| t.status == Status::Done)
        .filter_map(|t| done_date(t).map(|d| (d, t.id.as_str(), t.title.as_str())))
        .collect();
    dones.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
    dones.truncate(DONE_ROWS);
    dones
}

fn counts_line(counts: &BTreeMap<&str, usize>, invalid: usize, repo: &str) -> String {
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

pub(crate) fn run(json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let store = load_repo(&root).map_err(|e| e.to_string())?;

    // Ready via the normative SQL (single source of queue truth).
    let ready = super::query::sql_rows_local(super::query::READY_SQL)?;
    let ready_ids: BTreeSet<&str> = ready.iter().map(|r| r[0].as_str()).collect();

    let tasks: Vec<&Task> = store
        .entries
        .iter()
        .filter_map(|e| match &e.parsed {
            ParsedTask::Valid(t) => Some(t.as_ref()),
            ParsedTask::Invalid(_) => None,
        })
        .collect();
    let invalid = store.entries.len() - tasks.len();

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &tasks {
        *counts.entry(t.status.as_str()).or_default() += 1;
    }

    let (ranked, rollup_line) = rollup(&tasks);
    let mut weather = weather_lines(&tasks, &ready_ids);
    let mut next_block = next_block_lines(&tasks, &ready);
    let mut also_ready = also_ready_lines(&tasks, &ready);
    let dones = recent_dones(&tasks);

    if json {
        let next_task = ready
            .first()
            .and_then(|r| tasks.iter().find(|t| t.id == r[0]).copied());
        emit_prime_json(
            &counts,
            &ready,
            &ranked,
            &weather,
            next_task,
            &dones,
            provenance_line(&root).as_deref(),
        );
        return Ok(());
    }

    // Assemble lines, then enforce the byte budget with a loud tail.
    let mut lines: Vec<String> = vec![counts_line(&counts, invalid, &store.repo)];
    if let Some(p) = provenance_line(&root) {
        lines.push(clamp_bytes(&p, LINE_CLAMP));
    }
    if let Some(r) = rollup_line {
        lines.push(clamp_bytes(&r, LINE_CLAMP));
    }
    if !weather.is_empty() {
        lines.push("weather:".to_string());
        lines.append(&mut weather);
    }
    lines.append(&mut next_block);
    if !also_ready.is_empty() {
        lines.push(format!(
            "also ready ({} more, top {}):",
            ready.len().saturating_sub(1),
            also_ready.len()
        ));
        lines.append(&mut also_ready);
    }
    if ready.len() > READY_ROWS {
        lines.push(format!(
            "\u{2026} and {} more (ready --all)",
            ready.len() - READY_ROWS
        ));
    }
    if !dones.is_empty() {
        lines.push("recently done:".to_string());
        for (date, id, title) in &dones {
            lines.push(clamp_bytes(&format!("- {date} {id} {title}"), LINE_CLAMP));
        }
    }
    if invalid > 0 {
        lines.push(format!("! {invalid} invalid file(s) \u{2014} run lint"));
    }

    let mut out = String::new();
    for line in &lines {
        if out.len() + line.len() + 1 > BUDGET - (TAIL.len() + 1) {
            let _ = writeln!(out, "{TAIL}");
            break;
        }
        let _ = writeln!(out, "{line}");
    }
    print!("{out}");
    Ok(())
}

fn emit_prime_json(
    counts: &BTreeMap<&str, usize>,
    ready: &[Vec<String>],
    rollup: &[(&str, i64, usize)],
    weather: &[String],
    next: Option<&Task>,
    dones: &[(String, &str, &str)],
    provenance: Option<&str>,
) {
    let ready_rows: Vec<_> = ready
        .iter()
        .take(READY_ROWS)
        .map(|r| serde_json::json!({ "id": r[0], "title": r[1] }))
        .collect();
    let rollup_rows: Vec<_> = rollup
        .iter()
        .take(ROLLUP_GROUPS)
        .map(|(g, s, n)| {
            serde_json::json!({ "group": g, "open": n,
                "min_seq": if *s == i64::MAX { None } else { Some(*s) } })
        })
        .collect();
    let next_row = next.map(|t| {
        serde_json::json!({ "id": t.id, "title": t.title, "handoff": t.handoff,
            "verify": t.verify, "docs": t.docs, "category": t.category })
    });
    let done_rows: Vec<_> = dones
        .iter()
        .map(|(d, id, title)| serde_json::json!({ "date": d, "id": id, "title": title }))
        .collect();
    crate::cli::emit_json(
        "prime",
        &serde_json::json!({
            "counts": counts, "provenance": provenance,
            "ready_total": ready.len(), "ready": ready_rows,
            "rollup": rollup_rows, "rollup_total": rollup.len(),
            "weather": weather, "next": next_row, "recently_done": done_rows,
        }),
    );
}
