//! `search <term>` (mw-5xdyxep, owner ask 2026-08-21) and `portfolio
//! search <term>` (mw-8ex271k, MW-M3): full-text search across everything
//! a session might remember writing — title, body, handoff, comment text,
//! log notes — archives included because they are loaded. The term is a
//! literal substring matched case-insensitively; never a pattern language
//! (REQUIREMENTS §3's query-DSL fence): the canned SQL filters with
//! `strpos`, quote-doubling the only escaping. The union verb runs the
//! same SQL over every registered store, hits grouped by repo.

use crate::write::clamp_bytes;
use datafusion::prelude::SessionContext;
use std::collections::BTreeMap;

#[derive(clap::Args)]
pub(crate) struct SearchArgs {
    /// Literal text to find (case-insensitive substring, not a pattern).
    pub(crate) term: String,
    /// Show every hit instead of the first 20.
    #[arg(long)]
    pub(crate) all: bool,
}

/// Snippet byte budget per matched field (MW-D5: bytes, never lines).
const SNIPPET_BYTES: usize = 160;

pub(crate) struct Hit {
    repo: String,
    id: String,
    title: String,
    status: String,
    /// (field, first matching line) — first match per field wins.
    matches: Vec<(&'static str, String)>,
}

pub(crate) fn run(args: &SearchArgs, json: bool) -> Result<(), String> {
    let (ctx, _) = crate::cli::query::local_session()?;
    let hits = hits(&ctx, &args.term, false)?;
    emit(&hits, args.all, json, "search", false, None);
    Ok(())
}

/// Every hit over the session's tables, ordered for listing: live tasks
/// before terminal ones (the archive is bulk history), then id — and
/// over the union, grouped by repo first. Stable and deterministic, no
/// relevance heuristics to drift.
pub(crate) fn hits(ctx: &SessionContext, term: &str, union: bool) -> Result<Vec<Hit>, String> {
    let term = term.trim();
    if term.is_empty() {
        return Err("search needs a non-empty term".to_string());
    }
    let lit = term.to_lowercase().replace('\'', "''");
    let has = |col: &str| format!("strpos(lower(coalesce({col},'')), '{lit}') > 0");

    let tasks = crate::cli::query::run_query(
        ctx,
        &format!(
            "SELECT t.gid, t.repo, t.id, t.title, t.status, t.body, t.handoff FROM tasks t \
             WHERE {} OR {} OR {}",
            has("t.title"),
            has("t.body"),
            has("t.handoff"),
        ),
    )?;
    let comments = crate::cli::query::run_query(
        ctx,
        &format!(
            "SELECT t.gid, t.repo, t.id, t.title, t.status, c.text \
             FROM comments c JOIN tasks t ON c.gid = t.gid \
             WHERE {} ORDER BY c.ord",
            has("c.text"),
        ),
    )?;
    let log = crate::cli::query::run_query(
        ctx,
        &format!(
            "SELECT t.gid, t.repo, t.id, t.title, t.status, l.note \
             FROM log l JOIN tasks t ON l.gid = t.gid \
             WHERE {} ORDER BY l.ord",
            has("l.note"),
        ),
    )?;

    let mut found: BTreeMap<String, Hit> = BTreeMap::new();
    for row in crate::cli::query::string_rows(&tasks.1) {
        // A title match needs no snippet — the hit line already shows it.
        upsert(&mut found, &row);
        add_match(&mut found, &row, "body", &row[5], term);
        add_match(&mut found, &row, "handoff", &row[6], term);
    }
    for row in crate::cli::query::string_rows(&comments.1) {
        add_match(&mut found, &row, "comment", &row[5], term);
    }
    for row in crate::cli::query::string_rows(&log.1) {
        add_match(&mut found, &row, "log", &row[5], term);
    }

    let mut ordered: Vec<Hit> = found.into_values().collect();
    ordered.sort_by(|a, b| {
        let group = |h: &Hit| (status_rank(&h.status), h.id.clone());
        if union {
            (a.repo.as_str(), group(a)).cmp(&(b.repo.as_str(), group(b)))
        } else {
            group(a).cmp(&group(b))
        }
    });
    Ok(ordered)
}

/// The listing: the MW-D2 cap with its marker unless `all`, live before
/// terminal; over the union each repo's hits under a header carrying the
/// repo's full count, rows named by `repo#id`. `extra` rides the JSON
/// data (the union's skip list).
pub(crate) fn emit(
    hits: &[Hit],
    all: bool,
    json: bool,
    verb: &str,
    union: bool,
    extra: Option<(&str, serde_json::Value)>,
) {
    let total = hits.len();
    let cap = if all {
        total
    } else {
        crate::cli::query::LISTING_CAP.min(total)
    };
    let name = |h: &Hit| {
        if union {
            format!("{}#{}", h.repo, h.id)
        } else {
            h.id.clone()
        }
    };

    if json {
        let rows: Vec<_> = hits[..cap]
            .iter()
            .map(|h| {
                let matches: Vec<_> = h
                    .matches
                    .iter()
                    .map(|(f, s)| serde_json::json!({ "field": f, "snippet": s }))
                    .collect();
                let mut row = serde_json::json!({ "id": h.id, "title": h.title,
                    "status": h.status, "matches": matches });
                if union {
                    row["gid"] = name(h).into();
                    row["repo"] = h.repo.clone().into();
                }
                row
            })
            .collect();
        let mut data = serde_json::json!({ "total": total, "rows": rows });
        if let Some((key, value)) = extra {
            data[key] = value;
        }
        crate::cli::emit_json(verb, &data);
        return;
    }

    let mut current_repo: Option<&str> = None;
    for h in &hits[..cap] {
        if union && current_repo != Some(h.repo.as_str()) {
            let n = hits.iter().filter(|x| x.repo == h.repo).count();
            let noun = if n == 1 { "hit" } else { "hits" };
            println!("{} ({n} {noun}):", crate::cli::sanitize(&h.repo));
            current_repo = Some(h.repo.as_str());
        }
        println!(
            "{}  {} [{}]",
            name(h),
            crate::cli::sanitize(&h.title),
            h.status
        );
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

/// Rows arrive as `[gid, repo, id, title, status, <field text>…]`.
fn upsert<'a>(hits: &'a mut BTreeMap<String, Hit>, row: &[String]) -> &'a mut Hit {
    hits.entry(row[0].clone()).or_insert_with(|| Hit {
        repo: row[1].clone(),
        id: row[2].clone(),
        title: row[3].clone(),
        status: row[4].clone(),
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
