//! `meshwork show <id>` (PLAN 0.5): the full single-task view — level two
//! of the flat two-level disclosure (MW-D1). Comments cap at last-3 with an
//! explicit `… and N more` marker (MW-K4/D2); `--comments` opts out.

use crate::parse::{parse_task_file, ParsedTask, Task};
use crate::store::find_task_file;

#[derive(clap::Args)]
pub(crate) struct ShowArgs {
    /// Task id (e.g. az-k7f3).
    id: String,
    /// Anchor-scoped excerpts of linked docs, ~4KB per link.
    #[arg(long)]
    docs: bool,
    /// Render all comments instead of the last 3.
    #[arg(long)]
    comments: bool,
}

const COMMENT_CAP: usize = 3;
/// Commits listed in the `commits:` tail before `… and N more` (MW-D2).
const COMMIT_CAP: usize = 10;

/// mw-ntn0t32: the closing-work commit set, derived read-side from the
/// id-in-subject convention — `git log --grep=<id>`, fixed-string, local
/// refs only (zero network, MW-J6). Works retroactively for every task
/// ever closed with the id in a commit message; no repo or no matches
/// degrade to empty, never an error.
fn commits_for(root: &std::path::Path, id: &str) -> Vec<(String, String)> {
    let Ok(out) = std::process::Command::new("git")
        .args(["log", "-F", &format!("--grep={id}"), "--format=%h %s"])
        .current_dir(root)
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            l.split_once(' ')
                .map(|(sha, subject)| (sha.to_string(), subject.to_string()))
        })
        .collect()
}

pub(crate) fn run(args: &ShowArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("docs").join("meshwork");
    let Some(path) = find_task_file(&tasks_dir, &args.id) else {
        return Err(format!("{} not found in {}", args.id, tasks_dir.display()));
    };
    // The path the store actually holds — archive/ included once close
    // has moved the file (mw-7ywrxf1: a root path for an archived task
    // sent an agent to `find`).
    let rel = path.strip_prefix(&root).map_or_else(
        |_| {
            format!(
                "docs/meshwork/{}",
                path.file_name().unwrap_or_default().to_string_lossy()
            )
        },
        |p| p.to_string_lossy().replace('\\', "/"),
    );

    match parse_task_file(&path) {
        ParsedTask::Valid(task) => {
            let shown_from = if args.comments {
                0
            } else {
                task.comments.len().saturating_sub(COMMENT_CAP)
            };
            // Stranded tail content surfaces in the body area, where it
            // would have rendered — stderr warnings scroll past unread
            // (mw-7tseswy). Same counter as lint's stray-tail-content.
            let stray = std::fs::read_to_string(&path)
                .ok()
                .and_then(|text| crate::lint_tail::relocate_stray(&text))
                .map(|(_, moved)| moved);
            let commits = commits_for(&root, &args.id);
            let excerpts = if args.docs {
                task.docs
                    .iter()
                    .map(|d| crate::docs::resolve(&root, d))
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            if json {
                emit_json(
                    &task, &rel, shown_from, &commits, args.docs, &excerpts, stray,
                );
            } else {
                render_text(&task, &rel, shown_from, &commits, stray);
                render_excerpts(&task, args.docs, &excerpts);
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

fn render_text(
    t: &Task,
    rel: &str,
    shown_from: usize,
    commits: &[(String, String)],
    stray: Option<usize>,
) {
    // Every task-derived string passes through sanitize (mw-8fmsws3) —
    // render-time only, the file keeps its bytes.
    let clean = crate::cli::sanitize;
    println!("{} — {} [{}]", t.id, clean(&t.title), t.status.as_str());
    let kv = |k: &str, v: Option<String>| {
        if let Some(v) = v {
            println!("{k}: {}", clean(&v));
        }
    };
    kv("category", t.category.clone());
    kv("labels", join_nonempty(&t.labels));
    kv("needs", join_nonempty(&t.needs));
    kv("parent", t.parent.clone());
    kv("discovered-from", t.discovered_from.clone());
    kv("relates", join_nonempty(&t.relates));
    kv("to", t.to.clone());
    kv("answers", t.answers.clone());
    kv("verify", t.verify.clone());
    kv("seq", t.seq.map(|s| s.to_string()));
    kv("created", t.created.clone());
    kv("github", t.github.map(|n| format!("#{n}")));
    kv("blocked-reason", t.blocked_reason.clone());
    kv("claimed-by", t.claimed_by.clone());
    kv("waived", t.waived.clone());
    for d in &t.docs {
        println!("doc: {}", clean(d));
    }
    for a in &t.attachments {
        println!("attachment: {}", clean(a));
    }
    println!("file: {rel}");
    if let Some(voice) = t.handoff.as_deref().filter(|h| !h.trim().is_empty()) {
        println!();
        for line in voice.lines() {
            println!("\u{bb} {}", clean(line));
        }
    }
    if !t.description.is_empty() {
        println!("\n{}", clean(&t.description));
    }
    if let Some(moved) = stray {
        println!(
            "\n\u{26a0} {moved} line(s) ignored in the tail sections — body \
             belongs above ## log / ## comments; lint --fix relocates them"
        );
    }
    if !t.log.is_empty() {
        println!("\nlog:");
        for entry in &t.log {
            println!("- {}", clean(entry).replace('\n', "\n  "));
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
                clean(&c.date),
                clean(&c.author),
                clean(&c.text).replace('\n', "\n  ")
            );
        }
    }
    if !commits.is_empty() {
        println!("\ncommits ({}):", commits.len());
        for (sha, subject) in commits.iter().take(COMMIT_CAP) {
            println!("- {sha} {}", clean(subject));
        }
        if commits.len() > COMMIT_CAP {
            println!(
                "… and {} more (git log --grep={})",
                commits.len() - COMMIT_CAP,
                t.id
            );
        }
    }
    for w in &t.warnings {
        eprintln!("warning: {}", clean(w));
    }
}

/// The drill-through tail (MW-F2): one header line per link, then the
/// anchored excerpt. Dead links render loud, the view never dies on them.
fn render_excerpts(t: &Task, docs: bool, excerpts: &[crate::docs::Excerpt]) {
    if !docs {
        return;
    }
    if t.docs.is_empty() {
        println!("\nno docs: links on {}", t.id);
        return;
    }
    let clean = crate::cli::sanitize;
    for e in excerpts {
        if let Some(err) = &e.error {
            println!("\n── {} — {}", clean(&e.link), clean(&err.to_string()));
            continue;
        }
        println!("\n── {}\n{}", clean(&e.link), clean(&e.text));
        if e.truncated {
            println!(
                "… truncated at {}B — read {} for the rest",
                crate::docs::EXCERPT_CAP,
                e.link.split('#').next().unwrap_or(&e.link)
            );
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_json(
    t: &Task,
    rel: &str,
    shown_from: usize,
    commits: &[(String, String)],
    docs: bool,
    excerpts: &[crate::docs::Excerpt],
    stray: Option<usize>,
) {
    let shown: Vec<_> = t.comments[shown_from..]
        .iter()
        .map(|c| serde_json::json!({ "date": c.date, "author": c.author, "text": c.text }))
        .collect();
    let mut payload = serde_json::json!({
            "id": t.id, "title": t.title, "status": t.status.as_str(),
            "category": t.category, "labels": t.labels, "needs": t.needs,
            "parent": t.parent, "discovered_from": t.discovered_from,
            "relates": t.relates, "verify": t.verify, "docs": t.docs,
            "attachments": t.attachments, "seq": t.seq, "github": t.github,
            "created": t.created, "blocked_reason": t.blocked_reason,
            "claimed_by": t.claimed_by, "waived": t.waived, "handoff": t.handoff,
            "to": t.to, "answers": t.answers,
            "description": t.description, "log": t.log,
            "comments": { "total": t.comments.len(), "shown": shown },
            "commits": commits.iter().take(COMMIT_CAP)
                .map(|(sha, subject)| serde_json::json!({ "sha": sha, "subject": subject }))
                .collect::<Vec<_>>(),
            "commits_total": commits.len(),
            "ignored_tail_lines": stray,
            "path": rel, "warnings": t.warnings,
    });
    if docs {
        payload["docs_excerpts"] = excerpts
            .iter()
            .map(|e| {
                serde_json::json!({
                    "link": e.link, "text": e.text,
                    "truncated": e.truncated,
                    "error": e.error.as_ref().map(ToString::to_string),
                })
            })
            .collect();
    }
    crate::cli::emit_json("show", &payload);
}

fn join_nonempty(items: &[String]) -> Option<String> {
    (!items.is_empty()).then(|| items.join(", "))
}
