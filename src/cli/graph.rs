//! `tree` / `why` / `blocked` (PLAN 1.2; MW-B8/C2, DESIGN §5): tree walks
//! `parent` downward at any depth with cosmetic level names; why walks
//! `needs` transitively and prints the frontier of actually-open blockers;
//! blocked lists reasons. The tool knows nothing about sprints — levels
//! are display strings indexed by depth, nothing more.

use crate::parse::{ParsedTask, Status, Task};
use crate::store::{load_repo, RepoStore};
use std::collections::BTreeMap;

#[derive(clap::Args)]
pub(crate) struct BlockedArgs {
    /// Show every blocked task instead of the first 20 (MW-D2).
    #[arg(long)]
    all: bool,
}

type TaskMap<'a> = BTreeMap<&'a str, &'a Task>;

fn task_map(store: &RepoStore) -> TaskMap<'_> {
    store
        .entries
        .iter()
        .filter_map(|e| match &e.parsed {
            ParsedTask::Valid(t) => Some((t.id.as_str(), t.as_ref())),
            ParsedTask::Invalid(_) => None,
        })
        .collect()
}

/// Children in stable file order (entries are filename-sorted).
fn children_map(store: &RepoStore) -> BTreeMap<&str, Vec<&str>> {
    let mut map: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for entry in &store.entries {
        if let ParsedTask::Valid(t) = &entry.parsed {
            if let Some(parent) = &t.parent {
                if !parent.contains('#') {
                    map.entry(parent.as_str()).or_default().push(t.id.as_str());
                }
            }
        }
    }
    map
}

pub(crate) fn tree(args: &super::transition::IdArg, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let store = load_repo(&root).map_err(|e| e.to_string())?;
    let tasks = task_map(&store);
    let children = children_map(&store);
    if !tasks.contains_key(args.id.as_str()) {
        return Err(format!("{} not found (or not parseable)", args.id));
    }

    // Level names index by ABSOLUTE hierarchy depth, so a subtree renders
    // the same labels as the full tree (MW-B8, cosmetic only).
    let levels = store
        .config
        .hierarchy
        .as_ref()
        .map(|h| h.levels.clone())
        .unwrap_or_default();
    let mut base_depth = 0usize;
    let mut cursor = args.id.as_str();
    while let Some(parent) = tasks.get(cursor).and_then(|t| t.parent.as_deref()) {
        if parent.contains('#') || !tasks.contains_key(parent) || base_depth > 32 {
            break;
        }
        base_depth += 1;
        cursor = parent;
    }

    if json {
        let node = tree_json(&args.id, base_depth, &tasks, &children, &levels);
        crate::cli::emit_json("tree", &node);
    } else {
        render_tree_text(&args.id, base_depth, 0, &tasks, &children, &levels);
    }
    Ok(())
}

fn level_label(levels: &[String], depth: usize) -> Option<&str> {
    levels.get(depth).map(String::as_str)
}

fn tree_json(
    id: &str,
    depth: usize,
    tasks: &TaskMap<'_>,
    children: &BTreeMap<&str, Vec<&str>>,
    levels: &[String],
) -> serde_json::Value {
    let task = tasks.get(id);
    let kids: Vec<serde_json::Value> = children
        .get(id)
        .map(|ids| {
            ids.iter()
                .map(|c| tree_json(c, depth + 1, tasks, children, levels))
                .collect()
        })
        .unwrap_or_default();
    serde_json::json!({
        "id": id,
        "title": task.map(|t| t.title.clone()),
        "status": task.map(|t| t.status.as_str()),
        "level": level_label(levels, depth),
        "children": kids,
    })
}

fn render_tree_text(
    id: &str,
    depth: usize,
    indent: usize,
    tasks: &TaskMap<'_>,
    children: &BTreeMap<&str, Vec<&str>>,
    levels: &[String],
) {
    let pad = "  ".repeat(indent);
    let label = level_label(levels, depth).map_or(String::new(), |l| format!("[{l}] "));
    match tasks.get(id) {
        Some(t) => println!("{pad}{id} {label}{} ({})", t.title, t.status.as_str()),
        None => println!("{pad}{id} {label}(missing)"),
    }
    if let Some(kids) = children.get(id) {
        for kid in kids {
            render_tree_text(kid, depth + 1, indent + 1, tasks, children, levels);
        }
    }
}

pub(crate) fn why(args: &super::transition::IdArg, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let store = load_repo(&root).map_err(|e| e.to_string())?;
    let tasks = task_map(&store);
    if !tasks.contains_key(args.id.as_str()) {
        return Err(format!("{} not found (or not parseable)", args.id));
    }

    let mut frontier: Vec<serde_json::Value> = Vec::new();
    let mut visited = Vec::new();
    collect_frontier(&args.id, &tasks, &mut visited, &mut frontier);
    frontier.sort_by_key(|f| {
        f["id"]
            .as_str()
            .or_else(|| f["ref"].as_str())
            .unwrap_or_default()
            .to_string()
    });
    frontier.dedup();

    if json {
        crate::cli::emit_json(
            "why",
            &serde_json::json!({ "id": args.id, "frontier": frontier }),
        );
    } else if frontier.is_empty() {
        println!(
            "{}: nothing blocking — every hard dep is done/dropped",
            args.id
        );
    } else {
        println!("{} blocked by {}:", args.id, frontier.len());
        for f in &frontier {
            if f["unresolved"] == true {
                println!(
                    "- {} (unresolved — absent or unregistered repo)",
                    f["ref"].as_str().unwrap_or("?")
                );
            } else {
                let reason = f["blocked_reason"]
                    .as_str()
                    .map_or(String::new(), |r| format!(" — {r}"));
                let verify = f["verify"]
                    .as_str()
                    .map_or(String::new(), |v| format!(" — verify: {v}"));
                println!(
                    "- {} ({}){reason}{verify}",
                    f["id"].as_str().unwrap_or("?"),
                    f["status"].as_str().unwrap_or("?"),
                );
            }
        }
    }
    Ok(())
}

/// DFS through unmet needs; a node joins the frontier when it blocks the
/// walk but nothing further blocks it — the actual thing to go do.
fn collect_frontier(
    id: &str,
    tasks: &TaskMap<'_>,
    visited: &mut Vec<String>,
    frontier: &mut Vec<serde_json::Value>,
) {
    if visited.iter().any(|v| v == id) {
        return; // cycle-safe; lint owns reporting cycles (MW-B2)
    }
    visited.push(id.to_string());
    let Some(task) = tasks.get(id) else { return };
    for target in &task.needs {
        match tasks.get(target.as_str()) {
            Some(dep) if matches!(dep.status, Status::Done | Status::Dropped) => {}
            Some(dep) => {
                let before = frontier.len();
                collect_frontier(target, tasks, visited, frontier);
                if frontier.len() == before {
                    frontier.push(serde_json::json!({
                        "id": dep.id, "title": dep.title,
                        "status": dep.status.as_str(),
                        "blocked_reason": dep.blocked_reason,
                        "verify": dep.verify,
                    }));
                }
            }
            // Cross-repo/dangling: conservative blocking (MW-G5); the
            // registry resolves what it can at M2.
            None => frontier.push(serde_json::json!({ "ref": target, "unresolved": true })),
        }
    }
}

/// `blocked` stays SQL like the other canned listings (MW-C2).
const BLOCKED_SQL: &str = "SELECT t.id, t.title, t.blocked_reason FROM tasks t \
     WHERE t.status = 'blocked' ORDER BY coalesce(t.seq, 999999), t.created, t.id";

pub(crate) fn blocked(args: &BlockedArgs, json: bool) -> Result<(), String> {
    let rows = super::query::sql_rows_local(BLOCKED_SQL)?;
    let total = rows.len();
    let cap = if args.all { total } else { total.min(20) };
    if json {
        let shown: Vec<_> = rows[..cap]
            .iter()
            .map(|r| serde_json::json!({ "id": r[0], "title": r[1], "blocked_reason": r[2] }))
            .collect();
        crate::cli::emit_json(
            "blocked",
            &serde_json::json!({ "total": total, "rows": shown }),
        );
    } else {
        for r in &rows[..cap] {
            println!("{}  {} — {}", r[0], r[1], r[2]);
        }
        if total > cap {
            println!("… and {} more (use --all)", total - cap);
        }
        if total == 0 {
            println!("nothing blocked");
        }
    }
    Ok(())
}
