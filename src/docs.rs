//! Anchor-scoped doc excerpts (MW-F1/F2): a `docs:` link is
//! `path[#§-anchor]`, repo-relative, or `repo#path[#§-anchor]` for a doc
//! in a registered sibling repo (mw-8q0srvb — the `needs:` spelling,
//! resolved through the registry and confined to that repo; a bare
//! `../` path stays refused). Its excerpt is the anchored section —
//! heading through the next same-or-shallower heading — byte-capped per
//! link. Drill-through is itself progressive disclosure: the section,
//! never the whole file. Unresolvable links resolve to an error string,
//! not a failure — a task view must never die on a stale doc pointer.

use std::path::{Path, PathBuf};

/// Per-link excerpt budget in bytes (MW-F2; bytes are the budget currency,
/// MW-D5).
pub const EXCERPT_CAP: usize = 4096;

/// One resolved `docs:` link, ready to render.
pub struct Excerpt {
    /// The link as written in frontmatter (`path[#§-anchor]`).
    pub link: String,
    /// The capped excerpt; empty when `error` is set.
    pub text: String,
    /// True when the section outran `EXCERPT_CAP` and was cut.
    pub truncated: bool,
    /// Why the link did not resolve.
    pub error: Option<LinkError>,
}

/// Why a `docs:` link fails to resolve — lint keys warnings off the
/// variant (MW-F3); the view just prints it.
pub enum LinkError {
    /// The path part didn't read.
    Unreadable {
        /// The path as written in the link.
        path: String,
    },
    /// The file read but no heading matched the anchor.
    AnchorMissing {
        /// The fragment as written after `#`.
        anchor: String,
        /// The closest heading's slug, when one shares anything with the
        /// anchor — the disagreement, spelled out for the author.
        nearest: Option<String>,
    },
    /// The path resolves outside the repo — absolute, traversing, or a
    /// symlink escape; never read (mw-2pz0zqc).
    Escapes {
        /// The path as written in the link.
        path: String,
    },
    /// `repo#path` names a repo the registry does not know.
    RepoUnknown {
        /// The repo segment as written.
        repo: String,
    },
    /// `repo#path` cannot be resolved here — no registry, or the repo
    /// has no local checkout; nothing was read, nothing is known.
    RepoUnavailable {
        /// The repo segment as written.
        repo: String,
        /// What is missing.
        why: String,
    },
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::Unreadable { path } => write!(f, "{path} not readable"),
            LinkError::AnchorMissing { anchor, nearest } => {
                write!(f, "anchor not found: #{anchor}")?;
                match nearest {
                    Some(n) => write!(f, " — nearest heading: #{n}"),
                    None => Ok(()),
                }
            }
            LinkError::Escapes { path } => write!(f, "{path} escapes the repo — refusing to read"),
            LinkError::RepoUnknown { repo } => {
                write!(f, "{repo} is not registered — never read")
            }
            LinkError::RepoUnavailable { repo, why } => {
                write!(f, "{repo} unavailable here ({why})")
            }
        }
    }
}

/// Where a link's file lives once the repo segment is settled.
enum Home {
    Local,
    Sibling(PathBuf),
    Failed(LinkError),
}

/// Settle `head` — the text before the first `#` — as a repo name or a
/// local path. A head with no `/` and no `.` may be a registered repo;
/// it is one when the registry says so, a local file when that exists,
/// and otherwise the registry's absence or ignorance is the answer.
fn home_of(root: &Path, head: &str) -> Home {
    let looks_like_repo = !head.contains(['/', '.']) && !head.is_empty();
    if !looks_like_repo {
        return Home::Local;
    }
    let registry = crate::registry::quiet_load();
    let local_exists = root.join(head).exists();
    match registry {
        Ok(Some(reg)) => match reg.resolve(head) {
            Some((entry, _)) => match &entry.path {
                Some(p) if p.exists() => Home::Sibling(p.clone()),
                Some(p) => Home::Failed(LinkError::RepoUnavailable {
                    repo: head.to_string(),
                    why: format!("no checkout at {}", p.display()),
                }),
                None => Home::Failed(LinkError::RepoUnavailable {
                    repo: head.to_string(),
                    why: "no local path in the registry".to_string(),
                }),
            },
            None if local_exists => Home::Local,
            None => Home::Failed(LinkError::RepoUnknown {
                repo: head.to_string(),
            }),
        },
        _ if local_exists => Home::Local,
        Ok(None) => Home::Failed(LinkError::RepoUnavailable {
            repo: head.to_string(),
            why: "no registry (MESHWORK_PORTFOLIO unset)".to_string(),
        }),
        Err(e) => Home::Failed(LinkError::RepoUnavailable {
            repo: head.to_string(),
            why: format!("registry failed to load: {e}"),
        }),
    }
}

/// Resolve one `docs:` link against the repo root.
#[must_use]
pub fn resolve(root: &Path, link: &str) -> Excerpt {
    let make = |text, truncated, error| Excerpt {
        link: link.to_string(),
        text,
        truncated,
        error,
    };
    let (head, rest) = match link.split_once('#') {
        Some((h, r)) => (h, Some(r)),
        None => (link, None),
    };
    let (base, path, anchor) = match home_of(root, head) {
        Home::Local => {
            let (p, a) = (head, rest);
            (root.to_path_buf(), p, a)
        }
        Home::Sibling(sibling) => {
            // `repo#path[#anchor]`: the rest splits once more.
            let (p, a) = match rest.unwrap_or_default().split_once('#') {
                Some((p, a)) => (p, Some(a)),
                None => (rest.unwrap_or_default(), None),
            };
            (sibling, p, a)
        }
        Home::Failed(err) => return make(String::new(), false, Some(err)),
    };
    // Confinement before any read: the link is a string from a merged
    // task file (mw-2pz0zqc) — confined to whichever repo it names.
    let Ok(on_disk) = crate::paths::confine(&base, path) else {
        let err = LinkError::Escapes {
            path: path.to_string(),
        };
        return make(String::new(), false, Some(err));
    };
    let Ok(content) = std::fs::read_to_string(on_disk) else {
        let err = LinkError::Unreadable {
            path: path.to_string(),
        };
        return make(String::new(), false, Some(err));
    };
    let section = match anchor {
        Some(a) => {
            let Some(s) = anchored_section(&content, a) else {
                let err = LinkError::AnchorMissing {
                    anchor: a.to_string(),
                    nearest: nearest_heading(&content, a),
                };
                return make(String::new(), false, Some(err));
            };
            s
        }
        None => content.as_str(),
    };
    let (text, truncated) = cap(section);
    make(text, truncated, None)
}

/// The registered spelling of a `../<dir>/rest` link, when the registry
/// resolves `<dir>` as a repo name or alias: `<name>#rest`. None for any
/// other shape, or when no registry loads — the `../` form then stays a
/// `path-escape` finding for a human.
#[must_use]
pub fn crossrepo_rewrite(link: &str) -> Option<String> {
    let after = link.strip_prefix("../")?;
    let (dir, rest) = after.split_once('/')?;
    if dir.is_empty() || rest.is_empty() || dir.contains("..") {
        return None;
    }
    let registry = crate::registry::quiet_load().ok()??;
    let (entry, _) = registry.resolve(dir)?;
    Some(format!("{}#{rest}", entry.name))
}

/// The section owned by `anchor`: from its heading line through the line
/// before the next heading of the same or shallower level. Headings only
/// count outside fenced code blocks. Anchor matching is slug-prefix at a
/// `-` boundary under either slug rule, so the stable short form
/// (`§-10-migration`) keeps matching a heading whose tail wording drifts,
/// and an anchor copied from a rendered GitHub link matches as written.
fn anchored_section<'a>(content: &'a str, anchor: &str) -> Option<&'a str> {
    let heads = headings(content);
    let (i, &(start, level, _)) = heads
        .iter()
        .enumerate()
        .find(|(_, (_, _, text))| anchor_hits(anchor, text))?;
    let end = heads[i + 1..]
        .iter()
        .find(|(_, l, _)| *l <= level)
        .map_or(content.len(), |(offset, _, _)| *offset);
    Some(content[start..end].trim_end())
}

/// Every heading outside fenced code: (byte offset, level, text).
fn headings(content: &str) -> Vec<(usize, usize, &str)> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for (offset, line) in line_offsets(content) {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some((level, text)) = heading(line) {
            out.push((offset, level, text));
        }
    }
    out
}

/// `anchor` names `heading` when either slug rule prefix-matches at a
/// `-` boundary. Two rules because two authors: meshwork's hyphenates
/// every punctuation run, GitHub's drops punctuation outright — and an
/// anchor copied from a rendered heading link is the second kind
/// (sazed#sa-rj7vxkt).
fn anchor_hits(anchor: &str, heading: &str) -> bool {
    let hit = |target: String, s: String| {
        !target.is_empty() && (s == target || s.starts_with(&format!("{target}-")))
    };
    hit(slug(anchor), slug(heading)) || hit(github_slug(anchor), github_slug(heading))
}

/// GitHub's heading slug: lowercased; every character that is not
/// alphanumeric, `-`, `_` or a space is dropped; spaces become `-`. So
/// `3.2` → `32`, `engine's` → `engines`, and a symbol between two spaces
/// leaves `--`. Leading hyphens trim so the `§-` convention costs nothing.
fn github_slug(text: &str) -> String {
    let mut out = String::new();
    for c in text.trim().chars() {
        if c == ' ' {
            out.push('-');
        } else if c.is_alphanumeric() || c == '-' || c == '_' {
            out.extend(c.to_lowercase());
        }
    }
    out.trim_start_matches('-').to_string()
}

/// The heading nearest a missed anchor, as the slug an author would copy
/// from a rendered link: one whose slug contains the anchor's (a dropped
/// section number or `§-`), else the longest shared prefix. None when no
/// heading shares a character — nothing to point at.
fn nearest_heading(content: &str, anchor: &str) -> Option<String> {
    let target = github_slug(anchor);
    if target.is_empty() {
        return None;
    }
    let mut best: Option<(usize, String)> = None;
    for (_, _, text) in headings(content) {
        let s = github_slug(text);
        let score = if s.contains(&target) {
            usize::MAX
        } else {
            s.chars()
                .zip(target.chars())
                .take_while(|(a, b)| a == b)
                .count()
        };
        if score > 0 && best.as_ref().is_none_or(|(b, _)| score > *b) {
            best = Some((score, s));
        }
    }
    best.map(|(_, s)| s)
}

/// (byte offset, line) pairs — offsets let the section borrow from the
/// original string instead of re-joining lines.
fn line_offsets(content: &str) -> impl Iterator<Item = (usize, &str)> {
    content.split_inclusive('\n').scan(0, |offset, line| {
        let at = *offset;
        *offset += line.len();
        Some((at, line.trim_end_matches(['\n', '\r'])))
    })
}

/// `## 10. Migration …` → `(2, "10. Migration …")`; None for non-headings.
fn heading(line: &str) -> Option<(usize, &str)> {
    let hashes = line.len() - line.trim_start_matches('#').len();
    let rest = &line[hashes..];
    (hashes > 0 && rest.starts_with(' ')).then(|| (hashes, rest.trim()))
}

/// Slug shared by anchors and headings: ascii-alphanumeric lowercased,
/// runs of anything else collapse to one `-`. Uncapped — anchors match by
/// prefix, so truncation would manufacture collisions.
fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut pending = false;
    for c in text.chars() {
        if c.is_ascii_alphanumeric() {
            if pending && !out.is_empty() {
                out.push('-');
            }
            pending = false;
            out.push(c.to_ascii_lowercase());
        } else {
            pending = true;
        }
    }
    out
}

/// Cap at `EXCERPT_CAP` bytes on a line boundary.
fn cap(section: &str) -> (String, bool) {
    if section.len() <= EXCERPT_CAP {
        return (section.to_string(), false);
    }
    let mut out = String::new();
    for line in section.lines() {
        if out.len() + line.len() + 1 > EXCERPT_CAP {
            break;
        }
        out.push_str(line);
        out.push('\n');
    }
    (out.trim_end().to_string(), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# T\n\n## 1. One (extra words)\n\nbody one.\n\n### 1a. Sub\n\nsub body.\n\n## 2. Two\n\nbody two.\n";

    #[test]
    fn anchor_scopes_to_section() {
        let s = anchored_section(DOC, "§-1-one").unwrap();
        assert!(s.contains("body one.") && s.contains("sub body."));
        assert!(!s.contains("body two."));
    }

    #[test]
    fn anchor_prefix_stops_at_boundary() {
        // `1-one` must not match a hypothetical `## 1-oneish` heading.
        assert!(anchored_section("## 1. Oneish\n\nx\n", "§-1-one").is_none());
        assert!(anchored_section(DOC, "§-9-none").is_none());
    }

    /// Punctuation vanishes rather than separating; a symbol between two
    /// spaces leaves a double hyphen; underscores and Unicode letters
    /// survive; the `§-` convention costs nothing.
    #[test]
    fn github_slug_matches_the_rendered_link() {
        for (heading, want) in [
            (
                "3.2 🔴 The peer engine's half — and its price",
                "32--the-peer-engines-half--and-its-price",
            ),
            (
                "Expression compiler (`sazed-plan::expr`)",
                "expression-compiler-sazed-planexpr",
            ),
            ("check_perf and Élan", "check_perf-and-élan"),
            ("§-10-migration", "10-migration"),
        ] {
            assert_eq!(github_slug(heading), want, "{heading}");
        }
        let doc = "## 5.4 A checkpoint holds STATE, never the computation\n\nbody.\n";
        assert!(anchored_section(doc, "54-a-checkpoint-holds-state").is_some());
        assert!(anchored_section(doc, "§-5-4-a-checkpoint-holds-state").is_some());
    }

    /// A miss points at the heading whose slug contains the anchor's,
    /// else the longest shared prefix — never at nothing shared.
    #[test]
    fn nearest_heading_names_the_disagreement() {
        let doc = "## 1. What it is\n\n### Crates\n\n## 6. Engine parity — the gap\n";
        assert_eq!(
            nearest_heading(doc, "engine-parity").as_deref(),
            Some("6-engine-parity--the-gap")
        );
        assert_eq!(
            nearest_heading(doc, "1-the-crates").as_deref(),
            Some("1-what-it-is")
        );
        assert_eq!(nearest_heading(doc, "zzz"), None);
    }

    #[test]
    fn cap_is_bytes_on_line_boundary() {
        let big = "0123456789\n".repeat(1000);
        let (text, truncated) = cap(&big);
        assert!(truncated && text.len() <= EXCERPT_CAP);
    }
}
