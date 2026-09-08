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
            LinkError::AnchorMissing { anchor } => write!(f, "anchor not found: #{anchor}"),
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
