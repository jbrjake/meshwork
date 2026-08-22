//! `search <term>` (mw-5xdyxep, owner ask 2026-08-21): full-text search
//! across everything a session might remember writing — title, body,
//! handoff, comment text, log notes — archives included because they are
//! loaded. The term is a literal substring matched case-insensitively;
//! never a pattern language (REQUIREMENTS §3's query-DSL fence): the
//! canned SQL filters with `strpos`, quote-doubling the only escaping.

use crate::write::clamp_bytes;
use std::collections::BTreeMap;

#[derive(clap::Args)]
pub(crate) struct SearchArgs {
    /// Literal text to find (case-insensitive substring, not a pattern).
    term: String,
    /// Show every hit instead of the first 20.
    #[arg(long)]
    all: bool,
}

/// Snippet byte budget per matched field (MW-D5: bytes, never lines).
const SNIPPET_BYTES: usize = 160;

struct Hit {
    title: String,
    status: String,
    /// (field, first matching line) — first match per field wins.
    matches: Vec<(&'static str, String)>,
}

pub(crate) fn run(args: &SearchArgs, json: bool) -> Result<(), String> {
    let term = args.term.trim();
    if term.is_empty() {
        return Err("search needs a non-empty term".to_string());
    }
    let lit = term.to_lowercase().replace('\'', "''");
    let has = |col: &str| format!("strpos(lower(coalesce({col},'')), '{lit}') > 0");

    let (ctx, _) = crate::cli::query::local_session()?;
    let tasks = crate::cli::query::run_query(
        &ctx,
        &format!(
            "SELECT t.id, t.title, t.status, t.body, t.handoff FROM tasks t \
             WHERE {} OR {} OR {}",
            has("t.title"),
            has("t.body"),
            has("t.handoff"),
        ),
    )?;
    let comments = crate::cli::query::run_query(
        &ctx,
        &format!(
            "SELECT t.id, t.title, t.status, c.text \
             FROM comments c JOIN tasks t ON c.gid = t.gid \
             WHERE {} ORDER BY c.ord",
            has("c.text"),
        ),
    )?;
    let log = crate::cli::query::run_query(
        &ctx,
        &format!(
            "SELECT t.id, t.title, t.status, l.note \
             FROM log l JOIN tasks t ON l.gid = t.gid \
             WHERE {} ORDER BY l.ord",
            has("l.note"),
        ),
    )?;

    let mut hits: BTreeMap<String, Hit> = BTreeMap::new();
    for row in crate::cli::query::string_rows(&tasks.1) {
        // A title match needs no snippet — the hit line already shows it.
        upsert(&mut hits, &row);
        add_match(&mut hits, &row, "body", &row[3], term);
        add_match(&mut hits, &row, "handoff", &row[4], term);
    }
    for row in crate::cli::query::string_rows(&comments.1) {
        add_match(&mut hits, &row, "comment", &row[3], term);
    }
    for row in crate::cli::query::string_rows(&log.1) {
        add_match(&mut hits, &row, "log", &row[3], term);
    }

    // Live tasks first (the archive is bulk history), then id — stable
    // and deterministic, no relevance heuristics to drift.
    let mut ordered: Vec<(String, Hit)> = hits.into_iter().collect();
    ordered.sort_by_key(|(id, h)| (status_rank(&h.status), id.clone()));

    let total = ordered.len();
    let cap = if args.all {
        total
    } else {
        crate::cli::query::LISTING_CAP.min(total)
    };

    if json {
        let rows: Vec<_> = ordered[..cap]
            .iter()
            .map(|(id, h)| {
                let matches: Vec<_> = h
                    .matches
                    .iter()
                    .map(|(f, s)| serde_json::json!({ "field": f, "snippet": s }))
                    .collect();
                serde_json::json!({ "id": id, "title": h.title,
                    "status": h.status, "matches": matches })
            })
            .collect();
        crate::cli::emit_json(
            "search",
            &serde_json::json!({ "total": total, "rows": rows }),
        );
    } else {
        for (id, h) in &ordered[..cap] {
            println!("{id}  {} [{}]", crate::cli::sanitize(&h.title), h.status);
            for (field, snippet) in &h.matches {
                println!("    {field}: {}", crate::cli::sanitize(snippet));
            }
        }
        if total > cap {
            println!("… and {} more (use --all)", total - cap);
        }
        if total == 0 {
            println!("no matches");
        }
    }
    Ok(())
}

/// Rows arrive as `[id, title, status, <field text>]`.
fn upsert<'a>(hits: &'a mut BTreeMap<String, Hit>, row: &[String]) -> &'a mut Hit {
    hits.entry(row[0].clone()).or_insert_with(|| Hit {
        title: row[1].clone(),
        status: row[2].clone(),
        matches: Vec::new(),
    })
}

fn add_match(
    hits: &mut BTreeMap<String, Hit>,
    row: &[String],
    field: &'static str,
    text: &str,
    term: &str,
) {
    let Some(line) = matching_line(text, term) else {
        return;
    };
    let hit = upsert(hits, row);
    if hit.matches.iter().all(|(f, _)| *f != field) {
        hit.matches.push((field, line));
    }
}

fn contains_ci(haystack: &str, term: &str) -> bool {
    haystack.to_lowercase().contains(&term.to_lowercase())
}

/// First line containing the term, trimmed and byte-capped.
fn matching_line(text: &str, term: &str) -> Option<String> {
    text.lines()
        .find(|l| contains_ci(l, term))
        .map(|l| clamp_bytes(l.trim(), SNIPPET_BYTES))
}

fn status_rank(status: &str) -> u8 {
    match status {
        "open" => 0,
        "doing" => 1,
        "blocked" => 2,
        "done" => 3,
        "dropped" => 4,
        _ => 5,
    }
}
