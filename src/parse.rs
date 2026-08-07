//! Task-file parser (PLAN 0.1): strict serde model over YAML frontmatter plus
//! `## log` / `## comments` tail sections (DESIGN §2–3).
//!
//! Contract: unknown keys warn, never fail (MW-A6); hard failures — missing
//! fences, duplicate top-level keys (union-merge damage), YAML errors, schema
//! violations — become [`ParsedTask::Invalid`] carrying the filename-recovered
//! ID and the error text, so the row stays visible everywhere (MW-I2).

use serde::Deserialize;
use std::path::Path;

/// Task lifecycle states (MW-E1). Unparseable files are *not* a status —
/// they surface as [`ParsedTask::Invalid`] rows instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Not started; the `ready` queue draws from these.
    Open,
    /// In progress.
    Doing,
    /// Stuck; `blocked-reason` must name blocker + unblock condition.
    Blocked,
    /// Closed with `verify:` exit 0 (or an explicit, recorded waive).
    Done,
    /// Deliberately abandoned; never deleted.
    Dropped,
}

impl Status {
    /// The frontmatter spelling of this status.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Status::Open => "open",
            Status::Doing => "doing",
            Status::Blocked => "blocked",
            Status::Done => "done",
            Status::Dropped => "dropped",
        }
    }

    /// Inverse of [`Status::as_str`]; `None` for any other spelling.
    #[must_use]
    pub fn parse_str(s: &str) -> Option<Status> {
        match s {
            "open" => Some(Status::Open),
            "doing" => Some(Status::Doing),
            "blocked" => Some(Status::Blocked),
            "done" => Some(Status::Done),
            "dropped" => Some(Status::Dropped),
            _ => None,
        }
    }
}

/// One `## log` entry per the normative grammar (mw-3wnhhvp, DESIGN §2):
/// `- <date> <from>→<to>[ — <note>]` is a transition; anything else is free
/// text. Parsing is positional and NEVER validates history — the date is
/// the first token as written (minute stamp, date-only, or whatever an old
/// store holds), from/to fill only when the second token reads
/// `<status>→<status>`, and free-text entries keep the whole rest as note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEntry {
    /// First whitespace token as written; `None` only for an empty entry.
    pub date: Option<String>,
    /// Transition source, when the entry is a transition line.
    pub from: Option<Status>,
    /// Transition target, when the entry is a transition line.
    pub to: Option<Status>,
    /// Note after the ` — ` separator (transition) or the whole free text.
    pub note: Option<String>,
}

/// Parse one log entry (the text after `- `, continuations already joined).
#[must_use]
pub fn parse_log_line(entry: &str) -> LogEntry {
    let entry = entry.trim();
    let (date, rest) = match entry.split_once(char::is_whitespace) {
        Some((d, r)) => (d, r.trim_start()),
        None => (entry, ""),
    };
    let date = (!date.is_empty()).then(|| date.to_string());
    let (token, tail) = match rest.split_once(char::is_whitespace) {
        Some((t, r)) => (t, r.trim_start()),
        None => (rest, ""),
    };
    if let Some((f, t)) = token.split_once('\u{2192}') {
        if let (Some(from), Some(to)) = (Status::parse_str(f), Status::parse_str(t)) {
            // The `— ` separator is minted; hand-written notes without it
            // still count — lenient by rule, the grammar binds minting only.
            let note = tail.strip_prefix('\u{2014}').map_or(tail, str::trim_start);
            return LogEntry {
                date,
                from: Some(from),
                to: Some(to),
                note: (!note.is_empty()).then(|| note.to_string()),
            };
        }
    }
    LogEntry {
        date,
        from: None,
        to: None,
        note: (!rest.is_empty()).then(|| rest.to_string()),
    }
}

/// One comment: `- <date> [<author>] text` with two-space continuations
/// joined by newlines. Identity is self-professed, recorded as claimed (MW-K1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// Date token as written (not calendar-validated here; lint's job).
    pub date: String,
    /// Free author string — `jon`, `claude/f10a7561`, … (MW-K1).
    pub author: String,
    /// Comment text, continuation lines joined with `\n`.
    pub text: String,
}

/// A fully parsed, schema-valid task file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Task {
    /// `<alias>-<base32 suffix>` (MW-A4; minted at 7 chars since mw-1b09,
    /// length never validated — pre-existing 4-char IDs stay legal).
    pub id: String,
    /// One-line title.
    pub title: String,
    /// Lifecycle state (MW-E1).
    pub status: Status,
    /// One slash-path, arbitrary depth (MW-B4).
    pub category: Option<String>,
    /// Flat cross-cutting labels (MW-B5).
    pub labels: Vec<String>,
    /// Hard deps; `repo#id` crosses repos (MW-B1/B3).
    pub needs: Vec<String>,
    /// Same-repo nesting edge, child→parent (MW-B1).
    pub parent: Option<String>,
    /// Provenance edge (MW-E4).
    pub discovered_from: Option<String>,
    /// Soft links (MW-B1).
    pub relates: Vec<String>,
    /// Close gate command, run via `sh -c` (MW-E2).
    pub verify: Option<String>,
    /// Repo-relative doc links with optional `#§-anchors` (MW-F1).
    pub docs: Vec<String>,
    /// Repo-relative attachment paths under `docs/meshwork/attachments/` (MW-K2).
    pub attachments: Vec<String>,
    /// Per-repo order weight; portfolio overlay supersedes (MW-G4).
    pub seq: Option<i64>,
    /// Mirror issue number; set once, absent until mirrored (MW-H3).
    pub github: Option<u64>,
    /// Creation date as written.
    pub created: Option<String>,
    /// Required non-empty iff status is `blocked` (MW-E1; lint enforces).
    pub blocked_reason: Option<String>,
    /// Advisory claimant while doing/blocked — self-professed via the MW-K1
    /// chain, set by `start`, released by close/drop/reopen (mw-tb6gdr9).
    /// Never a lock; post-merge double-claims are lint's business.
    pub claimed_by: Option<String>,
    /// Waive reason recorded by `close --waive` — loud and queryable (MW-E2).
    pub waived: Option<String>,
    /// Outgoing session's color commentary to the incoming one — the only
    /// authored piece of the handoff view, meaningful only while up next
    /// (DESIGN §7b; stale on done tasks, lint `handoff-stale`).
    pub handoff: Option<String>,
    /// Body text before the first tail section.
    pub description: String,
    /// `## log` entries in file order, continuations joined (MW-E3).
    pub log: Vec<String>,
    /// `## comments` entries in file order (MW-K1).
    pub comments: Vec<Comment>,
    /// Non-fatal findings: unknown keys, filename/id mismatch, stray lines.
    pub warnings: Vec<String>,
}

/// A file that failed to parse — kept visible, never dropped (MW-I2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invalid {
    /// ID recovered from the `<id>-<slug>.md` filename.
    pub id: String,
    /// File name the error belongs to.
    pub file_name: String,
    /// Human-readable parse error.
    pub error: String,
}

/// Outcome of parsing one task file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedTask {
    /// Schema-valid task (possibly with warnings).
    Valid(Box<Task>),
    /// Parse failure carried as a loud row (MW-I2).
    Invalid(Invalid),
}

/// Strict frontmatter model (DESIGN §2). `Option<Vec<_>>` tolerates
/// hand-edited empty keys (`labels:` with no value) as None.
#[derive(Debug, Deserialize)]
struct Frontmatter {
    id: String,
    title: String,
    status: Status,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    labels: Option<Vec<String>>,
    #[serde(default)]
    needs: Option<Vec<String>>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default, rename = "discovered-from")]
    discovered_from: Option<String>,
    #[serde(default)]
    relates: Option<Vec<String>>,
    #[serde(default)]
    verify: Option<String>,
    #[serde(default)]
    docs: Option<Vec<String>>,
    #[serde(default)]
    attachments: Option<Vec<String>>,
    #[serde(default)]
    seq: Option<i64>,
    #[serde(default)]
    github: Option<u64>,
    #[serde(default)]
    created: Option<String>,
    #[serde(default, rename = "blocked-reason")]
    blocked_reason: Option<String>,
    #[serde(default, rename = "claimed-by")]
    claimed_by: Option<String>,
    #[serde(default)]
    waived: Option<String>,
    #[serde(default)]
    handoff: Option<String>,
}

/// Frontmatter keys the schema knows; anything else warns (MW-A6).
const KNOWN_KEYS: &[&str] = &[
    "id",
    "title",
    "status",
    "category",
    "labels",
    "needs",
    "parent",
    "discovered-from",
    "relates",
    "verify",
    "docs",
    "attachments",
    "seq",
    "github",
    "created",
    "blocked-reason",
    "claimed-by",
    "waived",
    "handoff",
];

/// Recover the task ID from a `<alias>-<rand>-<slug>.md` filename: the first
/// two dash segments. The slug is cosmetic and never load-bearing (DESIGN §2).
#[must_use]
pub fn id_from_filename(file_name: &str) -> String {
    let stem = file_name.strip_suffix(".md").unwrap_or(file_name);
    let mut dashes = stem.match_indices('-').map(|(i, _)| i);
    let (Some(_), Some(second)) = (dashes.next(), dashes.next()) else {
        return stem.to_string();
    };
    stem[..second].to_string()
}

/// Parse a task file already read into memory. `file_name` is used for
/// ID recovery on failure and the filename/id mismatch warning.
#[must_use]
pub fn parse_task_str(file_name: &str, text: &str) -> ParsedTask {
    let invalid = |error: String| {
        ParsedTask::Invalid(Invalid {
            id: id_from_filename(file_name),
            file_name: file_name.to_string(),
            error,
        })
    };

    let Some((fm_text, body)) = split_frontmatter(text) else {
        return invalid("missing frontmatter fences (--- … ---)".to_string());
    };
    if let Some(key) = duplicate_top_level_key(fm_text) {
        return invalid(format!(
            "duplicate frontmatter key `{key}` — union-merge damage; run lint --fix (MW-I2)"
        ));
    }
    let fm: Frontmatter = match serde_yaml_ng::from_str(fm_text) {
        Ok(fm) => fm,
        Err(e) => return invalid(e.to_string()),
    };

    let mut warnings = Vec::new();
    warn_unknown_keys(fm_text, &mut warnings);
    if !fm.id.is_empty() && !file_name.starts_with(&format!("{}-", fm.id)) {
        warnings.push(format!(
            "filename `{file_name}` does not start with id `{}` — by-ID lookup globs on the filename prefix",
            fm.id
        ));
    }

    let (description, log, comments) = parse_body(body, &mut warnings);
    ParsedTask::Valid(Box::new(Task {
        id: fm.id,
        title: fm.title,
        status: fm.status,
        category: fm.category,
        labels: fm.labels.unwrap_or_default(),
        needs: fm.needs.unwrap_or_default(),
        parent: fm.parent,
        discovered_from: fm.discovered_from,
        relates: fm.relates.unwrap_or_default(),
        verify: fm.verify,
        docs: fm.docs.unwrap_or_default(),
        attachments: fm.attachments.unwrap_or_default(),
        seq: fm.seq,
        github: fm.github,
        created: fm.created,
        blocked_reason: fm.blocked_reason,
        claimed_by: fm.claimed_by,
        waived: fm.waived,
        handoff: fm.handoff,
        description,
        log,
        comments,
        warnings,
    }))
}

/// Parse a task file from disk; read failures are carried as invalid rows,
/// not errors — a broken file must stay visible (MW-I2).
#[must_use]
pub fn parse_task_file(path: &Path) -> ParsedTask {
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    match std::fs::read_to_string(path) {
        Ok(text) => parse_task_str(&file_name, &text),
        Err(e) => ParsedTask::Invalid(Invalid {
            id: id_from_filename(&file_name),
            file_name,
            error: format!("unreadable: {e}"),
        }),
    }
}

fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    match rest.find("\n---\n") {
        Some(end) => Some((&rest[..end], &rest[end + 5..])),
        // Tolerate a file that ends right at the closing fence.
        None => rest.strip_suffix("\n---").map(|fm| (fm, "")),
    }
}

/// Textual scan for duplicate top-level keys — union merge's signature
/// damage (MW-I1). Runs before YAML so the diagnosis is precise.
fn duplicate_top_level_key(fm_text: &str) -> Option<String> {
    let mut seen = std::collections::BTreeSet::new();
    top_level_keys(fm_text).find(|key| !seen.insert(key.clone()))
}

fn top_level_keys(fm_text: &str) -> impl Iterator<Item = String> + '_ {
    fm_text.lines().filter_map(|line| {
        let first = line.chars().next()?;
        if !first.is_ascii_alphabetic() {
            return None;
        }
        let key = line.split(':').next()?;
        key.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
            .then(|| key.to_string())
    })
}

fn warn_unknown_keys(fm_text: &str, warnings: &mut Vec<String>) {
    for key in top_level_keys(fm_text) {
        if !KNOWN_KEYS.contains(&key.as_str()) {
            warnings.push(format!("unknown frontmatter key `{key}` (MW-A6)"));
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Section {
    Description,
    Log,
    Comments,
    UnknownHeading,
}

/// Split the body into description / log / comments. The description may
/// contain arbitrary markdown; the first `## log` or `## comments` line
/// switches to tail mode, where entries are `- ` bullets with two-space
/// continuations (DESIGN §2).
fn parse_body(body: &str, warnings: &mut Vec<String>) -> (String, Vec<String>, Vec<Comment>) {
    let mut description = String::new();
    let mut entries: Vec<(Section, String)> = Vec::new();
    let mut section = Section::Description;

    for line in body.lines() {
        match line.trim_end() {
            "## log" => {
                section = Section::Log;
                continue;
            }
            "## comments" => {
                section = Section::Comments;
                continue;
            }
            trimmed if trimmed.starts_with("## ") && section != Section::Description => {
                warnings.push(format!(
                    "unexpected heading `{trimmed}` after tail sections; content ignored"
                ));
                section = Section::UnknownHeading;
                continue;
            }
            _ => {}
        }
        match section {
            Section::Description => {
                description.push_str(line);
                description.push('\n');
            }
            Section::UnknownHeading => {}
            sec @ (Section::Log | Section::Comments) => {
                if let Some(rest) = line.strip_prefix("- ") {
                    entries.push((sec, rest.to_string()));
                } else if let Some(cont) = line.strip_prefix("  ") {
                    match entries.last_mut() {
                        Some((last_sec, text)) if *last_sec == sec => {
                            text.push('\n');
                            text.push_str(cont);
                        }
                        _ => warnings.push(format!("continuation line without an entry: `{line}`")),
                    }
                } else if !line.trim().is_empty() {
                    warnings.push(format!("stray line in tail section: `{line}`"));
                }
            }
        }
    }

    let mut log = Vec::new();
    let mut comments = Vec::new();
    for (sec, entry) in entries {
        match sec {
            Section::Log => log.push(entry),
            Section::Comments => match parse_comment(&entry) {
                Some(c) => comments.push(c),
                None => warnings.push(format!(
                    "malformed comment (want `- <date> [<author>] text`): `- {entry}`"
                )),
            },
            _ => unreachable!("entries only collected for log/comments"),
        }
    }
    (description.trim().to_string(), log, comments)
}

fn parse_comment(entry: &str) -> Option<Comment> {
    let (date, rest) = entry.split_once(' ')?;
    let rest = rest.strip_prefix('[')?;
    let (author, text) = rest.split_once(']')?;
    if date.is_empty() || author.is_empty() {
        return None;
    }
    Some(Comment {
        date: date.to_string(),
        author: author.to_string(),
        text: text.trim_start().to_string(),
    })
}
