//! `meshwork prime` (PLAN 1.5 + mw-a8tv; MW-D3/D5, DESIGN §7b): the ≤6KB
//! session-start digest, now the materialized handoff. Headline counts +
//! top-5 category rollup by min seq; the pulse block and the weather
//! derived from the active frontier; the next task led by its `handoff:`
//! commentary; also-ready with blocks-lines; recently done from dated log
//! lines; one fixed footer. Bytes enforced, truncation loud. The
//! renderers live in `prime_render.rs`; this file reads the store, runs
//! the queries, fits the budget and emits. Hand-written HANDOFF files are
//! dead — this is the view.

use super::prime_render::{
    advisory_lines, also_ready_lines, assemble, counts_line, invalid_ids, next_block_lines,
    recent_dones, rollup, weather_lines, Digest, LINE_CLAMP, READY_ROWS, ROLLUP_GROUPS,
};
use crate::parse::{ParsedTask, Task};
use crate::store::load_repo;
use crate::write::clamp_bytes;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

/// The whole digest budget (MW-D3: 6KB ≈ 1.5K tokens at 4 bytes/token).
const BUDGET: usize = 6144;
/// The last line, always: what to do next with the digest itself.
const FOOTER: &str =
    "re-run meshwork prime when the question changes; load the meshwork skill before filing";
/// Visible marker when the budget forces a cut.
const TAIL: &str = "… truncated (6KB budget)";

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

pub(crate) fn run(json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let store = load_repo(&root).map_err(|e| e.to_string())?;

    // Ready via the normative SQL (single source of queue truth), over the
    // session a query would see: this store plus its terminal foreign
    // rows — the same inputs the Rust-side pulse reads (MW-S7).
    let foreign = super::query::terminal_foreign(&store)?;
    let ctx = crate::tables::session_for(std::slice::from_ref(&store), &foreign)
        .map_err(|e| e.to_string())?;
    let (_, batches) = super::query::run_query(&ctx, super::query::READY_SQL)?;
    let ready = super::query::string_rows(&batches);
    let ready_ids: BTreeSet<&str> = ready.iter().map(|r| r[0].as_str()).collect();
    let clock = crate::views::Clock::resolve(store.config.window_days())?;

    let tasks: Vec<&Task> = store
        .entries
        .iter()
        .filter_map(|e| match &e.parsed {
            ParsedTask::Valid(t) => Some(t.as_ref()),
            ParsedTask::Invalid(_) => None,
        })
        .collect();
    let invalid_ids = invalid_ids(&store);
    let invalid = invalid_ids.len();

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &tasks {
        *counts.entry(t.status.as_str()).or_default() += 1;
    }

    let (ranked, rollup_line) = rollup(&tasks);
    let weather = weather_lines(&tasks, &ready_ids);
    let dones = recent_dones(&tasks);
    let today = crate::clock::today();

    // One union read serves the inbox and the asks line (MW-S6); with no
    // registry the store answers for itself, as its own query would.
    let union = crate::addressed::union();
    let inbox = union
        .as_deref()
        .map_or_else(Vec::new, |s| crate::addressed::inbox_of(s, &store.repo));
    let mut pulse =
        crate::pulse::compute(std::slice::from_ref(&store), &foreign, &clock, &store.repo);
    if let Some(stores) = &union {
        let (a, b, c, d) = crate::pulse::asks(stores, &clock, &store.repo);
        pulse.asks_in_open = a;
        pulse.asks_in_oldest_d = b;
        pulse.asks_out_open = c;
        pulse.asks_out_oldest_d = d;
    }
    let next_task = ready
        .first()
        .and_then(|r| tasks.iter().find(|t| t.id == r[0]).copied());
    let cited = cited_by_next(&store, &foreign, next_task);
    let next_block = next_block_lines(&tasks, &ready, &cited, &store.repo);
    let also_ready = also_ready_lines(&tasks, &ready);

    if json {
        emit_prime_json(&PrimeJson {
            counts: &counts,
            ready: &ready,
            rollup: &ranked,
            weather: &weather,
            pulse: &pulse,
            next: next_task,
            cites: &cited,
            dones: &dones,
            inbox: &inbox,
            today: &today,
            provenance: provenance_line(&root).as_deref(),
        });
        return Ok(());
    }

    // Assemble lines under the byte budget — the pulse's cuts first, the
    // loud tail last. The headline carries the inbox's silence as a
    // number (mw-r6g9bhe): an ask nobody answers ages in every session's
    // first line.
    let mut headline = counts_line(&counts, invalid, &store.repo);
    if let Some(tail) = crate::addressed::headline_tail(&inbox, &today) {
        let _ = write!(headline, " \u{b7} {tail}");
    }
    let digest = Digest {
        headline,
        provenance: provenance_line(&root).map(|p| clamp_bytes(&p, LINE_CLAMP)),
        rollup: rollup_line.map(|r| clamp_bytes(&r, LINE_CLAMP)),
        pulse: &pulse,
        window_days: store.config.window_days(),
        weather: &weather,
        inbox: &inbox,
        today: &today,
        repo: &store.repo,
        next_block: &next_block,
        also_ready: &also_ready,
        ready_total: ready.len(),
        dones: &dones,
        advisories: advisory_lines(&store, &tasks, &invalid_ids),
    };
    // The tail and the footer are reserved off the top: the footer is the
    // one line that must outlive every cut (mw-d539ppk).
    let reserved = TAIL.len() + 1 + FOOTER.len() + 1;
    let lines = super::pulse::fit(BUDGET - reserved, also_ready.len(), |cuts| {
        assemble(&digest, cuts)
    });

    let mut out = String::new();
    for line in &lines {
        if out.len() + line.len() + 1 > BUDGET - reserved {
            let _ = writeln!(out, "{TAIL}");
            break;
        }
        let _ = writeln!(out, "{line}");
    }
    let _ = writeln!(out, "{FOOTER}");
    // One choke point for the whole digest — this text also lands in
    // hook-injected agent context (mw-8fmsws3).
    print!("{}", crate::cli::sanitize(&out));
    Ok(())
}

/// The closed tasks the next task's handoff still names (MW-S6) — the
/// mention pass over one block, resolved against every row the query
/// session would carry.
fn cited_by_next(
    store: &crate::store::RepoStore,
    foreign: &[crate::registry::ForeignTask],
    next: Option<&Task>,
) -> Vec<String> {
    let Some((t, voice)) = next.and_then(|t| t.handoff.as_deref().map(|h| (t, h))) else {
        return Vec::new();
    };
    let statuses = crate::pulse::statuses(std::slice::from_ref(store), foreign);
    crate::pulse::cited_closed(voice, &store.repo, &store.gid(&t.id), &statuses)
}

/// The digest's sections, bundled for the JSON emitter — one view, one arg.
struct PrimeJson<'a> {
    counts: &'a BTreeMap<&'a str, usize>,
    ready: &'a [Vec<String>],
    rollup: &'a [(&'a str, i64, usize)],
    weather: &'a [String],
    pulse: &'a crate::pulse::Pulse,
    next: Option<&'a Task>,
    cites: &'a [String],
    dones: &'a [(String, &'a str, &'a str)],
    inbox: &'a [crate::addressed::Ask],
    today: &'a str,
    provenance: Option<&'a str>,
}

fn emit_prime_json(v: &PrimeJson) {
    let ready_rows: Vec<_> = v
        .ready
        .iter()
        .take(READY_ROWS)
        .map(|r| serde_json::json!({ "id": r[0], "title": r[1] }))
        .collect();
    let rollup_rows: Vec<_> = v
        .rollup
        .iter()
        .take(ROLLUP_GROUPS)
        .map(|(g, s, n)| {
            serde_json::json!({ "group": g, "open": n,
                "min_seq": if *s == i64::MAX { None } else { Some(*s) } })
        })
        .collect();
    let next_row = v.next.map(|t| {
        serde_json::json!({ "id": t.id, "title": t.title, "handoff": t.handoff,
            "verify": t.verify, "docs": t.docs, "category": t.category,
            "cites": v.cites })
    });
    let done_rows: Vec<_> = v
        .dones
        .iter()
        .map(|(d, id, title)| serde_json::json!({ "date": d, "id": id, "title": title }))
        .collect();
    let addressed: Vec<_> = v
        .inbox
        .iter()
        .map(|a| {
            serde_json::json!({ "gid": a.gid, "title": a.title,
                "created": a.created, "age_days": a.age_days(v.today) })
        })
        .collect();
    let asks = serde_json::json!({
        "unanswered": v.inbox.len(),
        "oldest_days": crate::addressed::oldest_age_days(v.inbox, v.today),
    });
    crate::cli::emit_json(
        "prime",
        &serde_json::json!({
            "counts": v.counts, "provenance": v.provenance,
            "ready_total": v.ready.len(), "ready": ready_rows,
            "rollup": rollup_rows, "rollup_total": v.rollup.len(),
            "weather": v.weather, "pulse": super::pulse::json(v.pulse),
            "next": next_row, "recently_done": done_rows,
            "addressed": addressed, "asks": asks,
        }),
    );
}
