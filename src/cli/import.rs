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
    /// `[~]` in the source (mw-x5a8g9w): doing without a claimant is a
    /// lie, so the item lands open and its log remembers the marker.
    from_doing: bool,
}

pub(crate) fn todo(path: &Path, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let config = crate::store::load_config(&root).map_err(|e| e.to_string())?;
    let text =
        std::fs::read_to_string(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    let (mut items, carried) = parse_todo(&text);
    if items.is_empty() {
        return Err(format!(
            "{}: no checkbox items found (want the baseline `- [ ] **Title** — …` format)",
            path.display()
        ));
    }
    // Non-checkbox prose carries whole into one triage task (mw-gsgh8s7)
    // — a dropped asks-section is invisible at review; a triage task is not.
    let carried_n = carried.iter().filter(|l| !l.starts_with("## ")).count();
    let triage_idx = (!carried.is_empty()).then(|| {
        items.push(TodoItem {
            status: Status::Open,
            title: format!(
                "Imported prose needing triage ({})",
                path.file_name()
                    .map(|n| n.to_string_lossy())
                    .unwrap_or_default()
            ),
            context: carried,
            verify: None,
            seq: None,
            parent: None,
            from_doing: false,
        });
        items.len() - 1
    });

    let tasks_dir = root.join("docs").join("meshwork");
    std::fs::create_dir_all(&tasks_dir).map_err(|e| e.to_string())?;
    let seed = std::env::var("MESHWORK_ID_SEED").ok();
    let mut gen = IdGen::from_seed_str(seed.as_deref());
    let today = crate::clock::stamp();

    let mut created = Vec::new();
    let mut ids: Vec<String> = Vec::new();
    let mut short_titles = 0usize;
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for item in &items {
        let id = mint_unique(&config.alias, &tasks_dir, &mut gen).map_err(|e| e.to_string())?;
        short_titles += usize::from(warn_short_title(&id, &item.title));
        let parent_id = item.parent.map(|i| ids[i].as_str());
        let file = render(item, &id, parent_id, &today);
        let name = format!("{id}-{}.md", slugify(&item.title));
        // Already-terminal imports go straight to archive/ (mw-45e2qf4).
        let terminal = matches!(item.status, Status::Done | Status::Dropped);
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

    let carried_into = triage_idx.map(|i| ids[i].clone());
    if json {
        crate::cli::emit_json(
            "import",
            &serde_json::json!({
                "source": path.display().to_string(),
                "imported": created.len(), "nested": nested,
                "counts": counts, "tasks": created,
                "carried": carried_into.as_ref().map(|id| serde_json::json!({
                    "task": id, "lines": carried_n,
                })),
            }),
        );
    } else {
        println!(
            "{} imported ({})",
            created.len(),
            summary_line(&counts, nested)
        );
        let downgraded = items.iter().filter(|i| i.from_doing).count();
        print_notes(carried_into.as_deref(), carried_n, short_titles, downgraded);
        println!(
            "next: review with `meshwork ready` + `lint`, then archive the source \
             (git mv {} docs/archive/) — history never deletes",
            path.display()
        );
    }
    Ok(())
}

/// What the next continuation line would join (mw-mrjhwws): wrapped
/// checkbox lines join their headline, wrapped `verify:` lines join the
/// command — a blank line, heading, or bullet ends the join.
enum Cont {
    Headline,
    Verify,
    Body,
}

/// Carry a line no item can own (mw-gsgh8s7), prefixing its `## <heading>`
/// marker the first time a section contributes.
fn carry(carried: &mut Vec<String>, heading_emitted: &mut bool, heading: Option<&str>, line: &str) {
    if let Some(h) = heading {
        if !*heading_emitted {
            carried.push(format!("## {h}"));
            *heading_emitted = true;
        }
    }
    carried.push(line.trim_end().to_string());
}

fn parse_todo(text: &str) -> (Vec<TodoItem>, Vec<String>) {
    let mut items: Vec<TodoItem> = Vec::new();
    // (indent, item index) — enclosing checkboxes; nesting never spans a
    // heading (mw-17hnhzk).
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut in_now = false;
    let mut now_seq = 0i64;
    let mut cont = Cont::Body;
    // Prose with no item to belong to (mw-gsgh8s7): carried lines, with a
    // `## <heading>` marker emitted once per section that contributes.
    let mut carried: Vec<String> = Vec::new();
    let mut heading: Option<String> = None;
    let mut heading_emitted = false;
    for line in text.lines() {
        if let Some(h) = line.strip_prefix("## ") {
            in_now = h.trim().eq_ignore_ascii_case("now");
            stack.clear();
            cont = Cont::Body;
            heading = Some(h.trim().to_string());
            heading_emitted = false;
            continue;
        }
        let trimmed_start = line.trim_start();
        if trimmed_start.is_empty() {
            cont = Cont::Body;
            continue;
        }
        let indent = line.len() - trimmed_start.len();
        if let Some((status, rest)) = parse_marker(trimmed_start) {
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
                status,
                title: rest.to_string(), // raw headline; split after joining
                context: Vec::new(),
                verify: None,
                seq,
                parent,
                from_doing: false,
            });
            stack.push((indent, items.len() - 1));
            cont = Cont::Headline;
            continue;
        }
        if indent == 0 && trimmed_start.starts_with('#') {
            cont = Cont::Body;
            continue;
        }
        let Some(last) = items.last_mut() else {
            // Before the first checkbox there is no item to join — carry.
            carry(
                &mut carried,
                &mut heading_emitted,
                heading.as_deref(),
                trimmed_start,
            );
            continue;
        };
        continue_line(
            last,
            &mut cont,
            &mut carried,
            &mut heading_emitted,
            heading.as_deref(),
            indent,
            trimmed_start,
        );
    }
    for item in &mut items {
        let (title, headline_ctx) = split_headline(&item.title);
        item.title = title;
        if let Some(ctx) = headline_ctx {
            item.context.insert(0, ctx);
        }
        if let Some(v) = item.verify.take() {
            item.verify = Some(extract_command(&v));
        }
        // [~] lands open (mw-x5a8g9w): an import can't claim work, and
        // doing without a claimant seeds instant rot (mw-06j1wqe).
        if item.status == Status::Doing {
            item.status = Status::Open;
            item.from_doing = true;
        }
    }
    (items, carried)
}

/// One non-checkbox, non-heading line under an existing item: joins the
/// item's headline or verify per `cont` (mw-mrjhwws), lands in context, or
/// — column-0 body prose, the old silent drop point — carries (mw-gsgh8s7).
fn continue_line(
    last: &mut TodoItem,
    cont: &mut Cont,
    carried: &mut Vec<String>,
    heading_emitted: &mut bool,
    heading: Option<&str>,
    indent: usize,
    line: &str,
) {
    if line.starts_with("- ") {
        // A bullet is a new block, never a continuation.
        *cont = Cont::Body;
        if indent > 0 {
            last.context.push(line.trim_end().to_string());
        }
        return;
    }
    if indent > 0 {
        if let Some(v) = line.strip_prefix("verify:") {
            last.verify = Some(v.trim().to_string());
            *cont = Cont::Verify;
            return;
        }
    }
    match cont {
        Cont::Headline => {
            last.title.push(' ');
            last.title.push_str(line.trim_end());
        }
        Cont::Verify => {
            if let Some(v) = last.verify.as_mut() {
                v.push(' ');
                v.push_str(line.trim_end());
            }
        }
        Cont::Body => {
            if indent > 0 {
                last.context.push(line.trim_end().to_string());
            } else {
                carry(carried, heading_emitted, heading, line);
            }
        }
    }
}

/// The loud lines under the tally — every class of import surprise gets a
/// count; silence is the forbidden outcome.
fn print_notes(carried_into: Option<&str>, carried_n: usize, short_titles: usize, doing_n: usize) {
    if let Some(id) = carried_into {
        // Loud by design: prose used to vanish here (mw-gsgh8s7).
        println!("{carried_n} prose line(s) carried into {id} — triage into real tasks");
    }
    if short_titles > 0 {
        println!("{short_titles} single-token title(s) — retitle as work orders at review");
    }
    if doing_n > 0 {
        println!("{doing_n} [~] item(s) imported as open — claim in-flight work with `start`");
    }
}

/// The parenthesized import tally: per-status counts, plus the nested
/// count when present.
fn summary_line(counts: &std::collections::BTreeMap<&str, usize>, nested: usize) -> String {
    let mut summary = counts
        .iter()
        .map(|(k, v)| format!("{v} {k}"))
        .collect::<Vec<_>>()
        .join(", ");
    if nested > 0 {
        // Loud by design: nesting used to vanish here (mw-17hnhzk).
        let _ = write!(summary, "; {nested} nested as children");
    }
    summary
}

/// A whitespace-free title is a code, not a work order (mw-6mqm4em) —
/// sazed's R11/R8/R7 were unintelligible in every listing three days
/// later. Warn per title, on stderr, naming the minted id so the review
/// pass has its retitle handle; the import itself never blocks on this.
fn warn_short_title(id: &str, title: &str) -> bool {
    let single = title.split_whitespace().count() == 1;
    if single {
        eprintln!("warn: {id}: single-token title {title:?} — retitle as a work order");
    }
    single
}

fn parse_marker(line: &str) -> Option<(Status, &str)> {
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
    Some((status, rest))
}

/// The joined headline splits into title + its own context: bold form
/// (`**Title** — ctx`) first, em-dash form second.
fn split_headline(headline: &str) -> (String, Option<String>) {
    let (title, ctx) = match headline.split_once("**") {
        Some(("", tail)) => match tail.split_once("**") {
            Some((title, ctx)) => (
                title.trim().to_string(),
                ctx.trim_start_matches([' ', '—', '-']).trim().to_string(),
            ),
            None => (tail.trim().to_string(), String::new()),
        },
        _ => match headline.split_once(" — ") {
            Some((title, ctx)) => (title.trim().to_string(), ctx.trim().to_string()),
            None => (headline.trim().to_string(), String::new()),
        },
    };
    let ctx = (!ctx.is_empty()).then_some(ctx);
    (title, ctx)
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
    let note = if item.from_doing {
        " ([~] in source imported as open — doing needs a claimant)"
    } else {
        ""
    };
    let _ = write!(body, "## log\n- {today} imported from TODO.md{note}\n");
    format!("---\n{fm}---\n{body}")
}
