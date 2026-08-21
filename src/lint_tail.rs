//! Stray-body detection and relocation for the tail sections
//! (mw-t01ek6s, mw-n3xgfs0; FORMAT.md Tail-section grammars). The damage
//! class: `cat >>` and hand-written `##` bodies land BELOW `## log`,
//! where the parser legally ignores everything that isn't a `- ` bullet,
//! a two-space continuation, or a blank — content the author meant as
//! body silently vanishes from every projection. Detection walks the
//! same state machine as the parser; the repair moves exactly the
//! ignored lines above the first tail heading, order preserved, and
//! refuses itself if a single non-blank line would be lost or invented.

use crate::lint::{finding, Finding, Severity};
use crate::parse::{ParsedTask, Status};
use crate::store::RepoStore;

/// Flag live tasks whose files strand content in the tail sections, and
/// any task whose body fence never closes. Stray content skips terminal
/// tasks (history rot is not noise — the docs-link precedent); the fence
/// check does not, because an open fence swallows the tail sections and
/// corrupts projected history itself. Warnings: `--fix` owns the stray
/// repair; only a human knows where a missing fence close belongs.
pub(crate) fn check(store: &RepoStore, out: &mut Vec<Finding>) {
    for entry in &store.entries {
        let ParsedTask::Valid(t) = &entry.parsed else {
            continue;
        };
        let path = crate::store::tasks_dir(&store.root).join(&entry.file_name);
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        check_unclosed_fence(&text, &t.id, out);
        if matches!(t.status, Status::Done | Status::Dropped) {
            continue;
        }
        if let Some((_, moved)) = relocate_stray(&text) {
            out.push(finding(
                Severity::Warning,
                "stray-tail-content",
                &t.id,
                format!(
                    "{moved} line(s) stranded in the tail sections — the parser \
                     ignores them, so they are invisible to every projection \
                     (lint --fix relocates them above the tail)"
                ),
            ));
        }
    }
}

/// mw-gw569q7: a fence that never closes swallows every later line —
/// including the real `## log` / `## comments` — into fenced body content,
/// so the task's history projects as empty. Found in the wild: a repro
/// truncated at filing left an open fence that hid an archived task's log.
/// Frontmatter never counts (a handoff block scalar may quote fences);
/// only the body is scanned.
fn check_unclosed_fence(text: &str, id: &str, out: &mut Vec<Finding>) {
    let Some(body) = body_of(text) else {
        return;
    };
    let mut fence = crate::parse::Fence::default();
    for line in body.lines() {
        fence.observe(line);
    }
    if fence.in_fence() {
        out.push(finding(
            Severity::Warning,
            "fence-unclosed",
            id,
            "a fenced code block never closes — everything after its opener, \
             including the log and comments sections, reads as fenced body \
             content and vanishes from history (close the fence where the \
             quoted content ends)"
                .to_string(),
        ));
    }
}

/// The body span: everything after the frontmatter's closing fence.
fn body_of(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    rest[end..].strip_prefix("\n---\n")
}

/// The parser's line classes inside `## log` / `## comments`.
fn legal_tail_line(line: &str) -> bool {
    line.starts_with("- ") || line.starts_with("  ") || line.trim().is_empty()
}

fn is_tail_heading(line: &str) -> bool {
    matches!(line.trim_end(), "## log" | "## comments")
}

/// Relocate every parser-ignored line above the first tail heading,
/// order preserved: `(repaired text, moved line count)`, or `None` when
/// the file has no frontmatter or nothing is stray. Pure reorder — the
/// multiset of lines is unchanged by construction (stray heading blocks
/// move whole: the `## X` line and everything under it until the next
/// tail heading, exactly what the parser discards).
#[must_use]
pub fn relocate_stray(text: &str) -> Option<(String, usize)> {
    // A file ending at the close fence has no body, so no stray.
    let body = body_of(text)?;
    let head_len = text.len() - body.len();

    let mut description: Vec<&str> = Vec::new();
    let mut tail: Vec<&str> = Vec::new();
    let mut stray: Vec<&str> = Vec::new();
    let mut in_tail = false;
    let mut in_ignored_block = false;
    let mut fence = crate::parse::Fence::default();
    for line in body.lines() {
        // A quoted heading inside a fence is content, not a boundary;
        // fenced content stranded in the tail moves whole, like an
        // ignored heading block.
        let fenced = fence.observe(line);
        if !fenced && is_tail_heading(line) {
            (in_tail, in_ignored_block) = (true, false);
            tail.push(line);
        } else if !in_tail {
            description.push(line);
        } else if in_ignored_block || fenced {
            stray.push(line);
        } else if line.trim_end().starts_with("## ") {
            in_ignored_block = true;
            stray.push(line);
        } else if legal_tail_line(line) {
            tail.push(line);
        } else {
            stray.push(line);
        }
    }
    if stray.is_empty() {
        return None;
    }

    let moved = stray.len();
    let mut out = String::with_capacity(text.len() + 1);
    out.push_str(&text[..head_len]);
    for line in description.iter().chain(&stray).chain(&tail) {
        out.push_str(line);
        out.push('\n');
    }
    Some((out, moved))
}
