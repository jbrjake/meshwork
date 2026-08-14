//! Anchor-scoped doc excerpts (MW-F1/F2): a `docs:` link is
//! `path[#§-anchor]`, repo-relative; its excerpt is the anchored section —
//! heading through the next same-or-shallower heading — byte-capped per
//! link. Drill-through is itself progressive disclosure: the section,
//! never the whole file. Unresolvable links resolve to an error string,
//! not a failure — a task view must never die on a stale doc pointer.

use std::path::Path;

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
    },
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LinkError::Unreadable { path } => write!(f, "{path} not readable"),
            LinkError::AnchorMissing { anchor } => write!(f, "anchor not found: #{anchor}"),
        }
    }
}

/// Resolve one `docs:` link against the repo root.
#[must_use]
pub fn resolve(root: &Path, link: &str) -> Excerpt {
    let (path, anchor) = match link.split_once('#') {
        Some((p, a)) => (p, Some(a)),
        None => (link, None),
    };
    let make = |text, truncated, error| Excerpt {
        link: link.to_string(),
        text,
        truncated,
        error,
    };
    let Ok(content) = std::fs::read_to_string(root.join(path)) else {
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

/// The section owned by `anchor`: from its heading line through the line
/// before the next heading of the same or shallower level. Headings only
/// count outside fenced code blocks. Anchor matching is slug-prefix at a
/// `-` boundary, so the stable short form (`§-10-migration`) keeps
/// matching a heading whose tail wording drifts.
fn anchored_section<'a>(content: &'a str, anchor: &str) -> Option<&'a str> {
    let target = slug(anchor);
    let mut start = None;
    let mut level = 0;
    let mut in_fence = false;
    let mut end = content.len();
    for (offset, line) in line_offsets(content) {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        let Some((l, text)) = heading(line) else {
            continue;
        };
        if in_fence {
            continue;
        }
        match start {
            None => {
                let s = slug(text);
                if s == target || s.starts_with(&format!("{target}-")) {
                    start = Some(offset);
                    level = l;
                }
            }
            Some(_) if l <= level => {
                end = offset;
                break;
            }
            Some(_) => {}
        }
    }
    start.map(|s| content[s..end].trim_end())
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

    #[test]
    fn cap_is_bytes_on_line_boundary() {
        let big = "0123456789\n".repeat(1000);
        let (text, truncated) = cap(&big);
        assert!(truncated && text.len() <= EXCERPT_CAP);
    }
}
