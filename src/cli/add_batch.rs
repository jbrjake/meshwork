//! `add --batch <file|->` (mw-af4kbjy): several tasks in one atomic
//! operation. Input is concatenated §2 task documents — the exact on-disk
//! format — with `id:` omitted and a local-only `handle: <name>` key;
//! `@<name>` is legal anywhere an id is (needs, parent, discovered-from,
//! relates). meshwork mints real ids, rewrites the refs, and writes every
//! file or none: all validation happens before the first write. Handles
//! never persist to disk. `--dry-run` prints the would-be files.

use crate::id::{mint_unique, slugify, IdGen};
use crate::parse::{parse_task_str, ParsedTask};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Read as _;

/// Frontmatter keys whose values are ids — the only places `@handle` refs
/// are resolved (a literal `@` in a title or verify stays untouched).
const EDGE_KEYS: &[&str] = &["needs", "parent", "discovered-from", "relates"];

struct Entry {
    handle: Option<String>,
    /// Frontmatter with the `handle:` line stripped, refs unresolved.
    fm: String,
    body: String,
}

pub(crate) fn run(source: &str, dry_run: bool, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let config = crate::store::load_config(&root).map_err(|e| e.to_string())?;
    let tasks_dir = root.join("docs").join("meshwork");

    let input = if source == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("reading stdin: {e}"))?;
        buf
    } else {
        std::fs::read_to_string(source).map_err(|e| format!("reading {source}: {e}"))?
    };

    let entries = split_documents(&input)?;

    // Mint one id per entry (same generator across the batch, so seeded
    // runs stay deterministic), collision-checked against disk AND the
    // batch itself — these files don't exist yet.
    let seed = std::env::var("MESHWORK_ID_SEED").ok();
    let mut idgen = IdGen::from_seed_str(seed.as_deref());
    let mut ids: Vec<String> = Vec::new();
    for _ in &entries {
        let id = loop {
            let id =
                mint_unique(&config.alias, &tasks_dir, &mut idgen).map_err(|e| e.to_string())?;
            if !ids.contains(&id) {
                break id;
            }
        };
        ids.push(id);
    }
    let mut by_handle: BTreeMap<String, String> = BTreeMap::new();
    for (entry, id) in entries.iter().zip(&ids) {
        if let Some(h) = &entry.handle {
            if by_handle.insert(h.clone(), id.clone()).is_some() {
                return Err(format!(
                    "duplicate handle `{h}` — handles are batch-local names and must be unique"
                ));
            }
        }
    }

    // Resolve refs + inject defaults, then validate EVERY document before
    // writing ANY file (partial failure writes nothing).
    let today = crate::clock::stamp();
    let mut files: Vec<(std::path::PathBuf, String, String)> = Vec::new(); // (path, rel, text)
    for (i, (entry, id)) in entries.iter().zip(&ids).enumerate() {
        let n = i + 1;
        let text = render_task(entry, id, &today, &by_handle)
            .map_err(|e| format!("batch task {n}: {e}"))?;
        let file_name = format!("{id}-{}.md", slug_of(&entry.fm));
        let task = match parse_task_str(&file_name, &text) {
            ParsedTask::Valid(t) => t,
            ParsedTask::Invalid(inv) => {
                return Err(format!("batch task {n}: {} — nothing written", inv.error))
            }
        };
        if task.parent.as_deref().is_some_and(|p| p.contains('#')) {
            return Err(format!(
                "batch task {n}: parent must stay in-repo — hierarchy never crosses repos (MW-B3)"
            ));
        }
        files.push((
            tasks_dir.join(&file_name),
            format!("docs/meshwork/{file_name}"),
            text,
        ));
    }

    if dry_run {
        for (_, rel, text) in &files {
            println!("--- {rel}");
            print!("{text}");
        }
        if json {
            emit(json, &ids, &files, true);
        }
        return Ok(());
    }

    std::fs::create_dir_all(&tasks_dir).map_err(|e| e.to_string())?;
    for (path, _, text) in &files {
        std::fs::write(path, text).map_err(|e| e.to_string())?;
    }
    emit(json, &ids, &files, false);
    Ok(())
}

fn emit(json: bool, ids: &[String], files: &[(std::path::PathBuf, String, String)], dry: bool) {
    if json {
        let tasks: Vec<_> = ids
            .iter()
            .zip(files)
            .map(|(id, (_, rel, _))| serde_json::json!({ "id": id, "path": rel }))
            .collect();
        crate::cli::emit_json(
            "add",
            &serde_json::json!({ "tasks": tasks, "dry_run": dry }),
        );
    } else {
        for (id, (_, rel, _)) in ids.iter().zip(files) {
            println!("{id}");
            println!("  {rel}");
        }
    }
}

/// Split concatenated §2 documents. A top-level `---` opens a frontmatter
/// block; the matching `---` closes it; body runs until the next opener.
/// (A bare `---` hr inside a batch body therefore starts a new document —
/// batch bodies use `***` for rules.)
fn split_documents(input: &str) -> Result<Vec<Entry>, String> {
    let mut docs = Vec::new();
    let mut lines = input.lines().peekable();
    loop {
        while lines.peek().is_some_and(|l| l.trim().is_empty()) {
            lines.next();
        }
        let Some(first) = lines.next() else { break };
        if first.trim_end() != "---" {
            return Err(format!(
                "expected `---` to open a task document, got `{first}`"
            ));
        }
        let mut fm = String::new();
        loop {
            let Some(line) = lines.next() else {
                return Err("unclosed frontmatter fence in batch input".to_string());
            };
            if line.trim_end() == "---" {
                break;
            }
            fm.push_str(line);
            fm.push('\n');
        }
        let mut body = String::new();
        while let Some(line) = lines.peek() {
            if line.trim_end() == "---" {
                break;
            }
            body.push_str(lines.next().unwrap_or_default());
            body.push('\n');
        }
        docs.push(parse_entry(&fm, body.trim())?);
    }
    if docs.is_empty() {
        return Err("empty batch — no task documents found".to_string());
    }
    Ok(docs)
}

/// Pull `handle:` out (it never persists) and reject a supplied `id:`.
fn parse_entry(fm: &str, body: &str) -> Result<Entry, String> {
    let mut handle = None;
    let mut kept = String::new();
    for line in fm.lines() {
        if let Some(rest) = line.strip_prefix("handle:") {
            let name = rest.trim();
            if name.is_empty()
                || !name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                return Err(format!(
                    "handle `{name}` must be non-empty [A-Za-z0-9_-] — it is referenced as @{name}"
                ));
            }
            handle = Some(name.to_string());
        } else {
            if line.starts_with("id:") {
                return Err(
                    "ids are minted, never supplied — drop `id:` and use `handle:` for local refs"
                        .to_string(),
                );
            }
            kept.push_str(line);
            kept.push('\n');
        }
    }
    Ok(Entry {
        handle,
        fm: kept,
        body: body.to_string(),
    })
}

/// The minted-id document: id first, refs resolved, `add`'s defaults
/// injected when absent (status: open, created, the created log line).
fn render_task(
    entry: &Entry,
    id: &str,
    today: &str,
    by_handle: &BTreeMap<String, String>,
) -> Result<String, String> {
    let mut fm = format!("id: {id}\n");
    let mut in_edge_block = false;
    let mut has_status = false;
    let mut has_created = false;
    for line in entry.fm.lines() {
        let top_level = line.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
        if top_level {
            let key = line.split(':').next().unwrap_or_default();
            in_edge_block = EDGE_KEYS.contains(&key);
            has_status |= key == "status";
            has_created |= key == "created";
        }
        // Edge values (inline or block items under an edge key) get their
        // @handles resolved; everything else passes through byte-for-byte.
        if in_edge_block {
            fm.push_str(&resolve_refs(line, by_handle)?);
        } else {
            fm.push_str(line);
        }
        fm.push('\n');
    }
    if !has_status {
        fm.push_str("status: open\n");
    }
    if !has_created {
        let _ = writeln!(fm, "created: {today}");
    }

    let mut text = format!("---\n{fm}---\n");
    if !entry.body.is_empty() {
        let _ = writeln!(text, "{}", entry.body);
    }
    if !entry.body.contains("## log") {
        let _ = write!(text, "\n## log\n- {today} created\n");
    }
    Ok(text)
}

/// Replace every `@name` in an edge line with its minted id; a ref to a
/// handle nobody declared fails the whole batch.
fn resolve_refs(line: &str, by_handle: &BTreeMap<String, String>) -> Result<String, String> {
    let mut out = String::new();
    let mut rest = line;
    while let Some(pos) = rest.find('@') {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + 1..];
        let end = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
            .unwrap_or(after.len());
        let name = &after[..end];
        let id = by_handle.get(name).ok_or(format!(
            "unknown handle `@{name}` — declare `handle: {name}` on a sibling task in the batch"
        ))?;
        out.push_str(id);
        rest = &after[end..];
    }
    out.push_str(rest);
    Ok(out)
}

/// Slug from the entry's own title line (pre-parse — the real parse runs
/// on the rendered document).
fn slug_of(fm: &str) -> String {
    let title = fm
        .lines()
        .find_map(|l| l.strip_prefix("title:"))
        .unwrap_or_default()
        .trim();
    slugify(title.trim_matches('"'))
}
