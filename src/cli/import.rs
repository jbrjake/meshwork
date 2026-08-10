//! `meshwork import todo <path>` (PLAN 1.7; MW-J3): the baseline checkbox
//! format — `[ ]`/`[~]`/`[x]`/`[!]`, bold titles, indented `verify:` lines,
//! `## Now` ordering → seq — becomes task files. Nested checkboxes are
//! REAL tasks with `parent:` edges at any depth, status from their own
//! marker (mw-17hnhzk — the sazed pilot lost 15 of 124 items folded into
//! parent prose with exit 0; silent loss is the one forbidden outcome).
//! The source is never touched; archiving it is the migration session's
//! explicit step.

use crate::id::{mint_unique, slugify, IdGen};
use crate::parse::Status;
use crate::write::yaml_scalar;
use std::fmt::Write as _;
use std::path::Path;

struct TodoItem {
    status: Status,
    title: String,
    context: Vec<String>,
    verify: Option<String>,
    seq: Option<i64>,
    /// Index of the enclosing checkbox (mw-17hnhzk) — always earlier in
    /// the document, so its id is minted before this one renders.
    parent: Option<usize>,
}

pub(crate) fn todo(path: &Path, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let config = crate::store::load_config(&root).map_err(|e| e.to_string())?;
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let items = parse_todo(&text);
    if items.is_empty() {
        return Err(format!(
            "{}: no checkbox items found (want the baseline `- [ ] **Title** — …` format)",
            path.display()
        ));
    }

    let tasks_dir = root.join("docs").join("meshwork");
    std::fs::create_dir_all(&tasks_dir).map_err(|e| e.to_string())?;
    let seed = std::env::var("MESHWORK_ID_SEED").ok();
    let mut gen = IdGen::from_seed_str(seed.as_deref());
    let today = crate::clock::stamp();

    let mut created = Vec::new();
    let mut ids: Vec<String> = Vec::new();
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for item in &items {
        let id = mint_unique(&config.alias, &tasks_dir, &mut gen).map_err(|e| e.to_string())?;
        let parent_id = item.parent.map(|i| ids[i].as_str());
        let file = render(item, &id, parent_id, &today);
        let name = format!("{id}-{}.md", slugify(&item.title));
        // Already-terminal imports go straight to archive/ (mw-45e2qf4).
        let terminal = matches!(
            item.status,
            crate::parse::Status::Done | crate::parse::Status::Dropped
        );
        let (dir, rel) = if terminal {
            (
                tasks_dir.join(crate::store::ARCHIVE_SUBDIR),
                format!("docs/meshwork/archive/{name}"),
            )
        } else {
            (tasks_dir.clone(), format!("docs/meshwork/{name}"))
        };
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(&name), file).map_err(|e| e.to_string())?;
        *counts.entry(item.status.as_str()).or_default() += 1;
        created.push(serde_json::json!({
            "id": id, "path": rel,
            "parent": item.parent.map(|i| ids[i].clone()),
        }));
        ids.push(id);
    }
    let nested = items.iter().filter(|i| i.parent.is_some()).count();

    if json {
        crate::cli::emit_json(
            "import",
            &serde_json::json!({
                "source": path.display().to_string(),
                "imported": created.len(), "nested": nested,
                "counts": counts, "tasks": created,
            }),
        );
    } else {
        let mut summary = counts
            .iter()
            .map(|(k, v)| format!("{v} {k}"))
            .collect::<Vec<_>>()
            .join(", ");
        if nested > 0 {
            // Loud by design: nesting used to vanish here (mw-17hnhzk).
            let _ = write!(summary, "; {nested} nested as children");
        }
        println!("{} imported ({summary})", created.len());
        println!(
            "next: review with `meshwork ready` + `lint`, then archive the source \
             (git mv {} docs/archive/) — history never deletes",
            path.display()
        );
    }
    Ok(())
}

fn parse_todo(text: &str) -> Vec<TodoItem> {
    let mut items: Vec<TodoItem> = Vec::new();
    // (indent, item index) — enclosing checkboxes; nesting never spans a
    // heading (mw-17hnhzk).
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut in_now = false;
    let mut now_seq = 0i64;
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            in_now = heading.trim().eq_ignore_ascii_case("now");
            stack.clear();
            continue;
        }
        let trimmed_start = line.trim_start();
        let indent = line.len() - trimmed_start.len();
        if let Some(item) = parse_bullet(trimmed_start) {
            // An indented checkbox is a CHILD of the nearest shallower one
            // — a real task, never parent-body prose (mw-17hnhzk).
            while stack.last().is_some_and(|(i, _)| *i >= indent) {
                stack.pop();
            }
            let parent = stack.last().map(|(_, idx)| *idx);
            let seq = in_now.then(|| {
                now_seq += 10;
                now_seq
            });
            items.push(TodoItem {
                seq,
                parent,
                ..item
            });
            stack.push((indent, items.len() - 1));
        } else if indent > 0 && !trimmed_start.is_empty() {
            // Continuation of the last item (which may itself be nested):
            // a verify line or more context.
            let Some(last) = items.last_mut() else {
                continue;
            };
            if let Some(v) = trimmed_start.strip_prefix("verify:") {
                last.verify = Some(extract_command(v));
            } else {
                last.context.push(trimmed_start.trim_end().to_string());
            }
        }
    }
    items
}

fn parse_bullet(line: &str) -> Option<TodoItem> {
    let rest = line.strip_prefix("- [")?;
    let (marker, rest) = rest.split_at(1);
    let rest = rest.strip_prefix("] ")?;
    let status = match marker {
        " " => Status::Open,
        "~" => Status::Doing,
        "x" => Status::Done,
        "!" => Status::Blocked,
        _ => return None,
    };
    let (title, context) = match rest.split_once("**") {
        Some(("", tail)) => match tail.split_once("**") {
            Some((title, ctx)) => (
                title.trim().to_string(),
                ctx.trim_start_matches([' ', '—', '-']).trim().to_string(),
            ),
            None => (tail.trim().to_string(), String::new()),
        },
        _ => match rest.split_once(" — ") {
            Some((title, ctx)) => (title.trim().to_string(), ctx.trim().to_string()),
            None => (rest.trim().to_string(), String::new()),
        },
    };
    let context = if context.is_empty() {
        vec![]
    } else {
        vec![context]
    };
    Some(TodoItem {
        status,
        title,
        context,
        verify: None,
        seq: None,
        parent: None,
    })
}

/// `verify:` convention: `` `command` exits 0`` — take the backticked
/// command; fall back to the raw text minus the "exits 0" tail.
fn extract_command(v: &str) -> String {
    let v = v.trim();
    if let Some(start) = v.find('`') {
        if let Some(len) = v[start + 1..].find('`') {
            return v[start + 1..start + 1 + len].to_string();
        }
    }
    v.trim_end_matches("exits 0").trim().to_string()
}

fn render(item: &TodoItem, id: &str, parent_id: Option<&str>, today: &str) -> String {
    let mut fm = String::new();
    let _ = writeln!(fm, "id: {id}");
    let _ = writeln!(fm, "title: {}", yaml_scalar(&item.title));
    let _ = writeln!(fm, "status: {}", item.status.as_str());
    if let Some(pid) = parent_id {
        let _ = writeln!(fm, "parent: {pid}");
    }
    if let Some(verify) = &item.verify {
        let _ = writeln!(fm, "verify: {}", yaml_scalar(verify));
    }
    if let Some(seq) = item.seq {
        let _ = writeln!(fm, "seq: {seq}");
    }
    let _ = writeln!(fm, "created: {today}");
    if item.status == Status::Blocked {
        let reason = item
            .context
            .first()
            .cloned()
            .unwrap_or_else(|| "unstated at import — fill in (MW-E1)".to_string());
        let _ = writeln!(fm, "blocked-reason: {}", yaml_scalar(&reason));
    }
    let description = item.context.join("\n");
    let mut body = String::new();
    if !description.is_empty() {
        body.push_str(&description);
        body.push_str("\n\n");
    }
    let _ = write!(body, "## log\n- {today} imported from TODO.md\n");
    format!("---\n{fm}---\n{body}")
}
