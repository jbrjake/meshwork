//! `meshwork prime` (PLAN 1.5; MW-D3/D5): the ≤6KB session-start digest
//! that replaces reading TODO.md+HANDOFF.md — ready top-10, in-progress
//! with the last log line, blocked with reasons, counts. The budget is
//! bytes, enforced, with truncation made visible.

use crate::parse::{ParsedTask, Status};
use crate::store::load_repo;
use crate::write::clamp_bytes;
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// The whole digest budget (MW-D3: 6KB ≈ 1.5K tokens at 4 bytes/token).
const BUDGET: usize = 6144;
/// Per-line clamp so one monster title can't eat the digest.
const LINE_CLAMP: usize = 160;
/// Ready rows in the digest (DESIGN §7: top-10).
const READY_ROWS: usize = 10;
/// Visible marker when the budget forces a cut.
const TAIL: &str = "… truncated (6KB budget, MW-D3)";

pub(crate) fn run(json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let store = load_repo(&root).map_err(|e| e.to_string())?;

    // Ready via the normative SQL (single source of queue truth).
    let ready = super::query::sql_rows_local(super::query::READY_SQL)?;

    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut doing: Vec<(String, String, Option<String>)> = Vec::new();
    let mut blocked: Vec<(String, String, String)> = Vec::new();
    for entry in &store.entries {
        match &entry.parsed {
            ParsedTask::Valid(t) => {
                *counts.entry(t.status.as_str()).or_default() += 1;
                match t.status {
                    Status::Doing => {
                        doing.push((t.id.clone(), t.title.clone(), t.log.last().cloned()));
                    }
                    Status::Blocked => blocked.push((
                        t.id.clone(),
                        t.title.clone(),
                        t.blocked_reason.clone().unwrap_or_default(),
                    )),
                    _ => {}
                }
            }
            ParsedTask::Invalid(_) => *counts.entry("invalid").or_default() += 1,
        }
    }

    let counts_line = {
        let mut parts: Vec<String> = Vec::new();
        for key in ["open", "doing", "blocked", "done", "dropped", "invalid"] {
            if let Some(n) = counts.get(key) {
                parts.push(format!("{n} {key}"));
            }
        }
        format!("{} — {}", store.repo, parts.join(", "))
    };

    if json {
        emit_prime_json(&counts, &ready, &doing, &blocked);
        return Ok(());
    }

    // Assemble lines, then enforce the byte budget with a loud tail.
    let mut lines: Vec<String> = vec![counts_line];
    lines.push(format!(
        "ready ({}, top {}):",
        ready.len(),
        READY_ROWS.min(ready.len())
    ));
    for r in ready.iter().take(READY_ROWS) {
        lines.push(clamp_bytes(&format!("- {} {}", r[0], r[1]), LINE_CLAMP));
    }
    if ready.len() > READY_ROWS {
        lines.push(format!(
            "… and {} more (ready --all)",
            ready.len() - READY_ROWS
        ));
    }
    if !doing.is_empty() {
        lines.push(format!("in progress ({}):", doing.len()));
        for (id, title, log) in &doing {
            let tail = log.as_deref().map_or(String::new(), |l| format!(" — {l}"));
            lines.push(clamp_bytes(&format!("- {id} {title}{tail}"), LINE_CLAMP));
        }
    }
    if !blocked.is_empty() {
        lines.push(format!("blocked ({}):", blocked.len()));
        for (id, title, reason) in &blocked {
            lines.push(clamp_bytes(
                &format!("- {id} {title} — {reason}"),
                LINE_CLAMP,
            ));
        }
    }
    if let Some(n) = counts.get("invalid") {
        lines.push(format!("! {n} invalid file(s) — run lint"));
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
    doing: &[(String, String, Option<String>)],
    blocked: &[(String, String, String)],
) {
    let ready_rows: Vec<_> = ready
        .iter()
        .take(READY_ROWS)
        .map(|r| serde_json::json!({ "id": r[0], "title": r[1] }))
        .collect();
    let doing_rows: Vec<_> = doing
        .iter()
        .map(|(id, title, log)| serde_json::json!({ "id": id, "title": title, "last_log": log }))
        .collect();
    let blocked_rows: Vec<_> = blocked
        .iter()
        .map(
            |(id, title, reason)| serde_json::json!({ "id": id, "title": title, "reason": reason }),
        )
        .collect();
    crate::cli::emit_json(
        "prime",
        &serde_json::json!({
            "counts": counts, "ready_total": ready.len(),
            "ready": ready_rows, "doing": doing_rows, "blocked": blocked_rows,
        }),
    );
}
