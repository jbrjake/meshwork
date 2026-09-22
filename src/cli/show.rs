//! `meshwork show <id>` (PLAN 0.5): the full single-task view — level two
//! of the flat two-level disclosure (MW-D1). Comments cap at last-3 with an
//! explicit `… and N more` marker (MW-K4/D2); `--comments` opts out.

use crate::parse::{ParsedTask, Task};

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
    let Some(located) = crate::archive::locate(&tasks_dir, &args.id) else {
        return Err(not_found(&root, &tasks_dir, &args.id));
    };
    let path = located.path().to_path_buf();
    // The path the store actually holds — archive/ included once close
    // has moved the file (mw-7ywrxf1: a root path for an archived task
    // sent an agent to `find`), the bundle once compaction folded it in.
    let rel = path.strip_prefix(&root).map_or_else(
        |_| {
            format!(
                "docs/meshwork/{}",
                path.file_name().unwrap_or_default().to_string_lossy()
            )
        },
        |p| p.to_string_lossy().replace('\\', "/"),
    );

    match located.parse() {
        ParsedTask::Valid(task) => {
            let shown_from = if args.comments {
                0
            } else {
                task.comments.len().saturating_sub(COMMENT_CAP)
            };
            // Stranded tail content surfaces in the body area, where it
            // would have rendered — stderr warnings scroll past unread
            // (mw-7tseswy). Same counter as lint's stray-tail-content.
            let stray = located
                .read()
                .ok()
                .and_then(|text| crate::lint_tail::relocate_stray(&text))
                .map(|(_, moved)| moved);
            let commits = commits_for(&root, &args.id);
            let (repo, lineage, cites) = derived(&task.id)?;
            // An ask carries its answer's state (MW-L5) — the one union
            // read `show` ever makes, and only on a task that has `to:`.
            let answered_by = task
                .to
                .as_ref()
                .and_then(|_| crate::addressed::answer_for(&root, &format!("{repo}#{}", task.id)));
            let excerpts = if args.docs {
                task.docs
                    .iter()
                    .map(|d| crate::docs::resolve(&root, d))
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            let views = Derived {
                repo: &repo,
                lineage: &lineage,
                cites: &cites,
                answered_by: answered_by.as_ref(),
            };
            if json {
                emit_json(
                    &task, &rel, shown_from, &commits, args.docs, &excerpts, stray, &views,
                );
            } else {
                render_text(&task, &rel, shown_from, &commits, stray, &views);
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

/// The lineage row's columns `show` renders (FORMAT.md §Views).
#[derive(Debug, Default)]
struct Lineage {
    spawned_total: i64,
    spawned_live: i64,
    spawn_depth: i64,
    children_direct: i64,
    descendants_total: i64,
    mentioned_by: i64,
}

/// What the views say about one task: its lineage row and the closed
/// tasks it still names anywhere (mw-9x1g9r1). `show` has no perf gate,
/// so it reads the views themselves rather than a Rust twin.
struct Derived<'a> {
    repo: &'a str,
    lineage: &'a Lineage,
    cites: &'a [String],
    /// The task answering this ask, when it has `to:` and one exists.
    answered_by: Option<&'a crate::addressed::Answer>,
}

/// The two derived rows for `id`: one query session with `lineage` and
/// `mentions` registered, the id quoted as a SQL literal.
fn derived(id: &str) -> Result<(String, Lineage, Vec<String>), String> {
    use super::query::{query_session, run_query, string_rows};
    let (ctx, repo) = query_session("lineage mentions")?;
    let gid = format!("{repo}#{}", id.replace('\'', "''"));
    let (_, batches) = run_query(
        &ctx,
        &format!(
            "SELECT spawned_total, spawned_live, spawn_depth, children_direct, \
             descendants_total, mentioned_by FROM lineage WHERE gid = '{gid}'"
        ),
    )?;
    let cell = |row: &[String], i: usize| row.get(i).and_then(|c| c.parse().ok()).unwrap_or(0);
    let lineage = string_rows(&batches)
        .first()
        .map(|r| Lineage {
            spawned_total: cell(r, 0),
            spawned_live: cell(r, 1),
            spawn_depth: cell(r, 2),
            children_direct: cell(r, 3),
            descendants_total: cell(r, 4),
            mentioned_by: cell(r, 5),
        })
        .unwrap_or_default();
    let (_, batches) = run_query(
        &ctx,
        &format!(
            "SELECT DISTINCT ref_gid FROM mentions WHERE src_gid = '{gid}' \
             AND ref_status IN ('done','dropped') ORDER BY ref_gid"
        ),
    )?;
    let cites = string_rows(&batches)
        .into_iter()
        .filter_map(|r| r.into_iter().next())
        .collect();
    Ok((repo, lineage, cites))
}

/// `lineage: spawned N (L live, depth D) · children C (T descendants) ·
/// mentioned by M` — each part only when its count is non-zero, the line
/// only when any is.
fn lineage_line(l: &Lineage) -> Option<String> {
    let mut parts = Vec::new();
    if l.spawned_total > 0 {
        parts.push(format!(
            "spawned {} ({} live, depth {})",
            l.spawned_total, l.spawned_live, l.spawn_depth
        ));
    }
    if l.children_direct > 0 {
        parts.push(format!(
            "children {} ({} descendants)",
            l.children_direct, l.descendants_total
        ));
    }
    if l.mentioned_by > 0 {
        parts.push(format!("mentioned by {}", l.mentioned_by));
    }
    (!parts.is_empty()).then(|| format!("lineage: {}", parts.join(" \u{b7} ")))
}

/// `cites: N closed (ids…)` — local ids bare, foreign as written, three
/// named then `+N`; nothing when nothing closed is named.
fn cites_line(cites: &[String], repo: &str) -> Option<String> {
    if cites.is_empty() {
        return None;
    }
    let prefix = format!("{repo}#");
    let mut named: Vec<String> = cites
        .iter()
        .take(3)
        .map(|g| g.strip_prefix(&prefix).unwrap_or(g).to_string())
        .collect();
    if cites.len() > 3 {
        named.push(format!("+{}", cites.len() - 3));
    }
    Some(format!(
        "cites: {} closed ({})",
        cites.len(),
        named.join(", ")
    ))
}

fn render_text(
    t: &Task,
    rel: &str,
    shown_from: usize,
    commits: &[(String, String)],
    stray: Option<usize>,
    views: &Derived,
) {
    // Every task-derived string passes through sanitize (mw-8fmsws3) —
    // render-time only, the file keeps its bytes.
    let clean = crate::cli::sanitize;
    println!("{} — {} [{}]", t.id, clean(&t.title), t.status.as_str());
    render_frontmatter(t, views);
    println!("file: {rel}");
    if let Some(voice) = t.handoff.as_deref().filter(|h| !h.trim().is_empty()) {
        println!();
        for line in voice.lines() {
            println!("\u{bb} {}", clean(line));
        }
    }
    render_tail(t, shown_from, commits, stray, views);
}

/// The frontmatter keys, one `k: v` line each, in projection order; an
/// ask additionally reads as its addressee, age and answer state.
fn render_frontmatter(t: &Task, views: &Derived) {
    let clean = crate::cli::sanitize;
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
    if let Some(to) = t.to.as_deref() {
        // The sender's view of its own ask: addressee, age, answer state
        // — the same words as the `asks out` line (MW-L5).
        let today = crate::clock::today();
        let age = t
            .created
            .as_deref()
            .and_then(|c| crate::clock::days_between(c, &today))
            .map_or(String::new(), |d| format!(" ({}d)", d.max(0)));
        println!(
            "ask: \u{2192} {}{age}{}",
            clean(to),
            crate::addressed::answered_suffix(views.answered_by, " \u{b7} ")
        );
    }
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
    for c in &t.covers {
        let pin = c.sha.as_deref().map_or("unpinned".to_string(), |s| {
            format!("@{}", &s[..s.len().min(12)])
        });
        println!("covers: {} {pin}", clean(&c.reference));
    }
    for a in &t.attachments {
        println!("attachment: {}", clean(a));
    }
}

/// Everything below the file line: body, the stray-tail marker, log,
/// comments, commits, and the derived lines.
fn render_tail(
    t: &Task,
    shown_from: usize,
    commits: &[(String, String)],
    stray: Option<usize>,
    views: &Derived,
) {
    let clean = crate::cli::sanitize;
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
    let derived: Vec<String> = [
        lineage_line(views.lineage),
        cites_line(views.cites, views.repo),
    ]
    .into_iter()
    .flatten()
    .collect();
    if !derived.is_empty() {
        println!();
        for line in derived {
            println!("{}", clean(&line));
        }
    }
    for w in &t.warnings {
        eprintln!("warning: {}", clean(w));
    }
}

/// The drill-through tail (MW-F2): one header line per link, then the
/// mw-48mzck9: an id with a sibling's prefix (or a `repo#id`) is not
/// "not found" — it is elsewhere. When the registry resolves the
/// prefix, the refusal says where and gives the one-liner; without a
/// registry, nothing is invented.
fn not_found(root: &std::path::Path, tasks_dir: &std::path::Path, id: &str) -> String {
    let plain = format!("{id} not found in {}", tasks_dir.display());
    let (repo_part, bare) = match id.split_once('#') {
        Some((r, i)) => (Some(r), i),
        None => (None, id),
    };
    let prefix = bare.split('-').next().unwrap_or(bare);
    let local_alias = crate::store::load_config(root).ok().map(|c| c.alias);
    if repo_part.is_none() && local_alias.as_deref() == Some(prefix) {
        return plain;
    }
    let Ok(Some(registry)) = crate::registry::quiet_load() else {
        return plain;
    };
    let Ok((stores, _)) = crate::registry::load_stores(&registry) else {
        return plain;
    };
    let home = stores
        .iter()
        .find(|s| repo_part.map_or(s.config.alias == prefix, |r| s.repo == r));
    match home {
        Some(s) => format!(
            "{plain}\n  `{bare}` belongs to {} (prefix `{prefix}`) — show it there:\n  \
             (cd {} && ./docs/meshwork/meshwork show {bare})",
            s.repo,
            s.root.display()
        ),
        None => plain,
    }
}

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
    views: &Derived,
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
            "covers": t.covers.iter()
                .map(|c| serde_json::json!({ "ref": c.reference, "sha": c.sha }))
                .collect::<Vec<_>>(),
            "attachments": t.attachments, "seq": t.seq, "github": t.github,
            "created": t.created, "blocked_reason": t.blocked_reason,
            "claimed_by": t.claimed_by, "waived": t.waived, "handoff": t.handoff,
            "to": t.to, "answers": t.answers,
            "answered_by": super::query::answer_json(views.answered_by),
            "description": t.description, "log": t.log,
            "comments": { "total": t.comments.len(), "shown": shown },
            "commits": commits.iter().take(COMMIT_CAP)
                .map(|(sha, subject)| serde_json::json!({ "sha": sha, "subject": subject }))
                .collect::<Vec<_>>(),
            "commits_total": commits.len(),
            "ignored_tail_lines": stray,
            "lineage": {
                "spawned_total": views.lineage.spawned_total,
                "spawned_live": views.lineage.spawned_live,
                "spawn_depth": views.lineage.spawn_depth,
                "children_direct": views.lineage.children_direct,
                "descendants_total": views.lineage.descendants_total,
                "mentioned_by": views.lineage.mentioned_by,
            },
            "cites": views.cites,
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
