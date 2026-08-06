//! `start` / `block --reason` / `drop` / `reopen` (PLAN 0.6; MW-E1/E3):
//! one-frontmatter-line status edits plus a dated log append — never a
//! full-file rewrite (MW-I1).

use crate::edit::{append_section_entry, set_scalar};
use crate::parse::{parse_task_file, ParsedTask, Status};
use crate::store::find_task_file;
use crate::write::yaml_scalar;

#[derive(clap::Args)]
pub(crate) struct IdArg {
    /// Task id (e.g. az-k7f3).
    pub(crate) id: String,
}

#[derive(clap::Args)]
pub(crate) struct BlockArgs {
    /// Task id (e.g. az-k7f3).
    id: String,
    /// Blocker + unblock condition — required; a bare "blocked" helps no
    /// one at session start (MW-E1).
    #[arg(long, required = true, value_name = "TEXT")]
    reason: String,
}

pub(crate) fn start(args: &IdArg, json: bool) -> Result<(), String> {
    transition(
        "start",
        &args.id,
        &[Status::Open],
        Status::Doing,
        None,
        json,
    )
}

pub(crate) fn block(args: &BlockArgs, json: bool) -> Result<(), String> {
    transition(
        "block",
        &args.id,
        &[Status::Open, Status::Doing],
        Status::Blocked,
        Some(&args.reason),
        json,
    )
}

pub(crate) fn drop(args: &IdArg, json: bool) -> Result<(), String> {
    transition(
        "drop",
        &args.id,
        &[Status::Open, Status::Doing, Status::Blocked],
        Status::Dropped,
        None,
        json,
    )
}

pub(crate) fn reopen(args: &IdArg, json: bool) -> Result<(), String> {
    // The missing inverse: without it every unblock is a hand-edit (§6).
    transition(
        "reopen",
        &args.id,
        &[Status::Blocked, Status::Doing, Status::Done],
        Status::Open,
        None,
        json,
    )
}

fn transition(
    verb: &str,
    id: &str,
    allowed_from: &[Status],
    to: Status,
    reason: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("docs").join("meshwork");
    let Some(path) = find_task_file(&tasks_dir, id) else {
        return Err(format!("{id} not found in {}", tasks_dir.display()));
    };
    let task = match parse_task_file(&path) {
        ParsedTask::Valid(t) => t,
        ParsedTask::Invalid(inv) => {
            return Err(format!(
                "{id} is invalid ({}) — repair it (lint --fix) before transitioning",
                inv.error
            ))
        }
    };
    if !allowed_from.contains(&task.status) {
        return Err(format!(
            "cannot {verb} {id}: status is {}, needs one of [{}]",
            task.status.as_str(),
            allowed_from
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut text = set_scalar(&text, "status", Some(to.as_str()))?;
    match (to, reason) {
        (Status::Blocked, Some(reason)) => {
            text = set_scalar(&text, "blocked-reason", Some(&yaml_scalar(reason)))?;
        }
        // Leaving blocked clears the reason but keeps the key — matching
        // the normative example's empty `blocked-reason:` line (DESIGN §2).
        (_, _) if task.status == Status::Blocked && task.blocked_reason.is_some() => {
            text = set_scalar(&text, "blocked-reason", None)?;
        }
        _ => {}
    }

    let today = crate::clock::today();
    let mut entry = format!("{today} {}→{}", task.status.as_str(), to.as_str());
    if let Some(reason) = reason {
        use std::fmt::Write as _;
        let _ = write!(entry, " — {reason}");
    }
    let text = append_section_entry(&text, "log", &entry);
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    // Terminal tasks live in archive/; reopen brings the file back
    // (mw-45e2qf4 — the graph never notices, only the directory does).
    let terminal = matches!(to, Status::Done | Status::Dropped);
    crate::store::relocate_for_status(&path, terminal).map_err(|e| e.to_string())?;

    if json {
        crate::cli::emit_json(
            verb,
            &serde_json::json!({
                "id": id, "from": task.status.as_str(), "to": to.as_str(),
                "reason": reason,
            }),
        );
    } else {
        println!("{id} {}→{}", task.status.as_str(), to.as_str());
    }
    Ok(())
}
