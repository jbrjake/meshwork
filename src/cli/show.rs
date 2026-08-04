//! `meshwork show <id>` (PLAN 0.5): the full single-task view — level two
//! of the flat two-level disclosure (MW-D1). Comments cap at last-3 with an
//! explicit `… and N more` marker (MW-K4/D2); `--comments` opts out.

use crate::parse::{parse_task_file, ParsedTask, Task};
use crate::store::find_task_file;

#[derive(clap::Args)]
pub(crate) struct ShowArgs {
    /// Task id (e.g. az-k7f3).
    id: String,
    /// Render all comments instead of the last 3 (MW-K4).
    #[arg(long)]
    comments: bool,
}

const COMMENT_CAP: usize = 3;

pub(crate) fn run(args: &ShowArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("meshwork").join("tasks");
    let Some(path) = find_task_file(&tasks_dir, &args.id) else {
        return Err(format!("{} not found in {}", args.id, tasks_dir.display()));
    };
    let rel = format!(
        "meshwork/tasks/{}",
        path.file_name().unwrap_or_default().to_string_lossy()
    );

    match parse_task_file(&path) {
        ParsedTask::Valid(task) => {
            let shown_from = if args.comments {
                0
            } else {
                task.comments.len().saturating_sub(COMMENT_CAP)
            };
            if json {
                emit_json(&task, &rel, shown_from);
            } else {
                render_text(&task, &rel, shown_from);
            }
            Ok(())
        }
        // Invalid rows stay loud in every path that reads them (MW-I2).
        ParsedTask::Invalid(inv) => {
            if json {
                crate::cli::emit_json(
                    "show",
                    &serde_json::json!({
                        "id": inv.id, "status": "invalid",
                        "error": inv.error, "path": rel,
                    }),
                );
                Ok(())
            } else {
                Err(format!("{rel}: INVALID — {}", inv.error))
            }
        }
    }
}

fn render_text(t: &Task, rel: &str, shown_from: usize) {
    println!("{} — {} [{}]", t.id, t.title, t.status.as_str());
    let kv = |k: &str, v: Option<String>| {
        if let Some(v) = v {
            println!("{k}: {v}");
        }
    };
    kv("category", t.category.clone());
    kv("labels", join_nonempty(&t.labels));
    kv("needs", join_nonempty(&t.needs));
    kv("parent", t.parent.clone());
    kv("discovered-from", t.discovered_from.clone());
    kv("relates", join_nonempty(&t.relates));
    kv("verify", t.verify.clone());
    kv("seq", t.seq.map(|s| s.to_string()));
    kv("created", t.created.clone());
    kv("github", t.github.map(|n| format!("#{n}")));
    kv("blocked-reason", t.blocked_reason.clone());
    kv("waived", t.waived.clone());
    for d in &t.docs {
        println!("doc: {d}");
    }
    for a in &t.attachments {
        println!("attachment: {a}");
    }
    println!("file: {rel}");
    if !t.description.is_empty() {
        println!("\n{}", t.description);
    }
    if !t.log.is_empty() {
        println!("\nlog:");
        for entry in &t.log {
            println!("- {}", entry.replace('\n', "\n  "));
        }
    }
    if !t.comments.is_empty() {
        let shown = &t.comments[shown_from..];
        println!(
            "\ncomments ({} total, showing last {}):",
            t.comments.len(),
            shown.len()
        );
        if shown_from > 0 {
            println!("… and {shown_from} more (use --comments)");
        }
        for c in shown {
            println!(
                "- {} [{}] {}",
                c.date,
                c.author,
                c.text.replace('\n', "\n  ")
            );
        }
    }
    for w in &t.warnings {
        eprintln!("warning: {w}");
    }
}

fn emit_json(t: &Task, rel: &str, shown_from: usize) {
    let shown: Vec<_> = t.comments[shown_from..]
        .iter()
        .map(|c| serde_json::json!({ "date": c.date, "author": c.author, "text": c.text }))
        .collect();
    crate::cli::emit_json(
        "show",
        &serde_json::json!({
            "id": t.id, "title": t.title, "status": t.status.as_str(),
            "category": t.category, "labels": t.labels, "needs": t.needs,
            "parent": t.parent, "discovered_from": t.discovered_from,
            "relates": t.relates, "verify": t.verify, "docs": t.docs,
            "attachments": t.attachments, "seq": t.seq, "github": t.github,
            "created": t.created, "blocked_reason": t.blocked_reason,
            "waived": t.waived, "description": t.description, "log": t.log,
            "comments": { "total": t.comments.len(), "shown": shown },
            "path": rel, "warnings": t.warnings,
        }),
    );
}

fn join_nonempty(items: &[String]) -> Option<String> {
    (!items.is_empty()).then(|| items.join(", "))
}
