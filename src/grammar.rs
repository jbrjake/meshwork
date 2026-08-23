//! Normative line grammars shared by the parser, renderers, and the
//! mirror (split from parse.rs, mw-0y66mhb): task status spellings
//! (MW-E1), the log-entry grammar (mw-3wnhhvp, DESIGN §2), comment lines
//! with the frozen comment-identity hash (mw-xvtf5jx, FORMAT.md), and
//! fence tracking that keeps heading/bullet scans honest inside quoted
//! code blocks (mw-svbdkvd). `parse` re-exports everything here — the
//! grammar is part of the parsing contract; only the file moved.

use serde::Deserialize;

/// Task lifecycle states (MW-E1). Unparseable files are *not* a status —
/// they surface as [`crate::parse::ParsedTask::Invalid`] rows instead.
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

impl Comment {
    /// The spec-level comment identity (mw-xvtf5jx, FORMAT.md):
    /// `SHA-256(date NUL author NUL text)` as lowercase hex. The mirror's
    /// idempotency markers abbreviate it to the first 8 chars (DESIGN §8);
    /// UI and replication layers dedup on it. Frozen — changing the tuple
    /// encoding desyncs every consumer at once.
    #[must_use]
    pub fn hash(&self) -> String {
        use sha2::{Digest, Sha256};
        use std::fmt::Write as _;
        let mut h = Sha256::new();
        h.update(self.date.as_bytes());
        h.update([0]);
        h.update(self.author.as_bytes());
        h.update([0]);
        h.update(self.text.as_bytes());
        let mut hex = String::with_capacity(64);
        for b in h.finalize() {
            let _ = write!(hex, "{b:02x}");
        }
        hex
    }
}

/// A code-fence line: a run of 3+ backticks or tildes at up to 1 leading
/// space. Returns the fence char, run length, and the remainder (info
/// string on an opener; must be blank on a closer). Tighter than
/// `CommonMark`'s 3-space allowance on purpose: two-space indent is this
/// format's tail-continuation namespace, so fences there belong to an
/// entry, never to the body-level scan.
#[must_use]
pub fn fence_run(line: &str) -> Option<(char, usize, &str)> {
    let indent = line.len() - line.trim_start_matches(' ').len();
    if indent > 1 {
        return None; // entry-continuation or indented code, not a body fence
    }
    let s = &line[indent..];
    let ch = s.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let run = s.chars().take_while(|c| *c == ch).count();
    if run < 3 {
        return None;
    }
    Some((ch, run, &s[run..]))
}

/// Fenced-code state across a line walk. Anything inside a fence —
/// delimiters included — is content: never a heading, a bullet, or a
/// document boundary (mw-svbdkvd; this store quotes its own format).
#[derive(Default)]
pub struct Fence(Option<(char, usize)>);

impl Fence {
    /// Feed the next line; true when it is a fence delimiter or fenced
    /// content. A closer must match the opening char, be at least as
    /// long, and carry no info string.
    pub fn observe(&mut self, line: &str) -> bool {
        match (self.0, fence_run(line)) {
            (Some((ch, len)), Some((c, n, rest)))
                if c == ch && n >= len && rest.trim().is_empty() =>
            {
                self.0 = None;
                true
            }
            (Some(_), _) => true,
            (None, Some((c, n, _info))) => {
                self.0 = Some((c, n));
                true
            }
            (None, None) => false,
        }
    }

    /// Whether the walk currently sits inside an open fence.
    #[must_use]
    pub fn in_fence(&self) -> bool {
        self.0.is_some()
    }
}

/// Per-line fenced flags for a whole task file: frontmatter lines are
/// never fenced (block scalars there are YAML, not markdown); tracking
/// starts after the closing fence.
#[must_use]
pub fn fenced_lines(lines: &[&str]) -> Vec<bool> {
    let body_at = if lines.first().map(|l| l.trim_end()) == Some("---") {
        lines
            .iter()
            .skip(1)
            .position(|l| l.trim_end() == "---")
            .map_or(lines.len(), |i| i + 2)
    } else {
        0
    };
    let mut fence = Fence::default();
    lines
        .iter()
        .enumerate()
        .map(|(i, line)| i >= body_at && fence.observe(line))
        .collect()
}
