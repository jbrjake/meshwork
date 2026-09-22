//! Clause anchors and pins (MW-T1–T3, MW-T6; DESIGN §15.18). A spec
//! document opts in by ending a heading with `{#sp-<slug>}`; the clause is
//! the section under that heading — its body, not the heading line, so a
//! retitled heading neither breaks the ref nor moves the hash (MW-T2). A
//! clause ref is `path#sp-<slug>` or `repo#path#sp-<slug>`, the `docs:`
//! spelling with a fragment that names an anchor id instead of a heading
//! slug; a pin is that ref plus the SHA-256 of the clause text as read
//! from disk (MW-T6 — no version control needed to detect drift).

use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::path::Path;

/// The fragment prefix that marks a clause id (never a heading slug).
pub const ANCHOR_PREFIX: &str = "sp-";

/// One anchored clause, hashed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    /// The anchor id, `sp-<slug>`.
    pub id: String,
    /// The heading text as written, with the anchor stripped.
    pub heading: String,
    /// The normalized body: lines right-trimmed, outer blank lines gone.
    pub text: String,
    /// SHA-256 of `text`, lowercase hex.
    pub sha: String,
}

/// A clause ref split into its document link and its anchor id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClauseRef {
    /// The document part — `path` or `repo#path`, a `docs:` link shape.
    pub doc: String,
    /// The anchor id, `sp-<slug>`.
    pub id: String,
}

impl ClauseRef {
    /// The ref as written: `<doc>#<id>`.
    #[must_use]
    pub fn render(&self) -> String {
        format!("{}#{}", self.doc, self.id)
    }
}

/// Split a clause ref. The last `#` fragment must be an anchor id;
/// anything else is not a clause ref.
///
/// # Errors
/// The text has no fragment, or its fragment is not `sp-<slug>` with a
/// slug of lowercase letters, digits and hyphens.
pub fn parse_ref(text: &str) -> Result<ClauseRef, String> {
    let Some((doc, id)) = text.rsplit_once('#') else {
        return Err(format!(
            "`{text}` has no clause id — a clause ref ends in `#sp-<slug>`"
        ));
    };
    if !is_anchor_id(id) {
        return Err(format!(
            "`{text}` does not end in a clause id — the fragment must be `sp-<slug>`, \
             lowercase letters, digits and hyphens"
        ));
    }
    if doc.is_empty() {
        return Err(format!("`{text}` names no document before its clause id"));
    }
    Ok(ClauseRef {
        doc: doc.to_string(),
        id: id.to_string(),
    })
}

/// `sp-<slug>`: the prefix then at least one of `[a-z0-9-]`.
#[must_use]
pub fn is_anchor_id(id: &str) -> bool {
    id.strip_prefix(ANCHOR_PREFIX).is_some_and(|slug| {
        !slug.is_empty()
            && slug
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    })
}

/// A pin's hash is 64 lowercase hex characters, exactly what [`sha_hex`]
/// writes; anything else was typed by hand (MW-T3).
#[must_use]
pub fn is_sha(text: &str) -> bool {
    text.len() == 64
        && text
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// SHA-256 of `text`, lowercase hex.
#[must_use]
pub fn sha_hex(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    let mut hex = String::with_capacity(64);
    for b in h.finalize() {
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// Every anchored clause in a document, in file order. A heading counts
/// only outside fenced code; its clause runs to the next heading of the
/// same or a shallower level, the heading line itself excluded.
#[must_use]
pub fn clauses(content: &str) -> Vec<Clause> {
    let heads = headings(content);
    let mut out = Vec::new();
    for (i, (start, level, line)) in heads.iter().enumerate() {
        let Some((heading, id)) = split_anchor(line) else {
            continue;
        };
        let body_start = start + line.len();
        let end = heads[i + 1..]
            .iter()
            .find(|(_, l, _)| *l <= *level)
            .map_or(content.len(), |(offset, _, _)| *offset);
        let text = normalize(&content[body_start.min(end)..end]);
        out.push(Clause {
            id: id.to_string(),
            heading: heading.trim_start_matches('#').trim().to_string(),
            sha: sha_hex(&text),
            text,
        });
    }
    out
}

/// The clause `id` names in `content`, if any.
#[must_use]
pub fn clause(content: &str, id: &str) -> Option<Clause> {
    clauses(content).into_iter().find(|c| c.id == id)
}

/// Read the ref's document — repo-relative, or through the registry for
/// a `repo#path` — and find its clause.
///
/// # Errors
/// The ref does not parse, the document cannot be read (unregistered
/// repo, escaping path, missing file), or no heading carries the id.
pub fn resolve(root: &Path, text: &str) -> Result<Clause, String> {
    let r = parse_ref(text)?;
    let content = crate::docs::read_target(root, &r.doc).map_err(|e| e.to_string())?;
    clause(&content, &r.id).ok_or_else(|| {
        let known: Vec<String> = clauses(&content).into_iter().map(|c| c.id).collect();
        if known.is_empty() {
            format!(
                "{} carries no clause anchors — a heading opts in by ending with `{{#{}<slug>}}`",
                r.doc, ANCHOR_PREFIX
            )
        } else {
            format!(
                "no clause `{}` in {} — anchored: {}",
                r.id,
                r.doc,
                known.join(", ")
            )
        }
    })
}

/// One task's pin on a clause of the audited document (MW-T5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    /// `repo#id` of the pinning task.
    pub gid: String,
    /// Its current status.
    pub status: crate::parse::Status,
    /// The ref as written in its `covers:`.
    pub reference: String,
    /// The clause id the ref names.
    pub id: String,
    /// The pinned hash, when the entry carries a well-formed one.
    pub sha: Option<String>,
}

/// A document's clauses as they read now, and every pin on them across
/// the loaded stores — the five coverage questions are views over it.
#[derive(Debug)]
pub struct Audit {
    /// The document as asked for.
    pub doc: String,
    /// Its anchored clauses, in file order.
    pub clauses: Vec<Clause>,
    /// Every pin naming this document, in store then file order.
    pub pins: Vec<Pin>,
}

impl Audit {
    /// The pins on clause `id`, dropped tasks included.
    #[must_use]
    pub fn covering(&self, id: &str) -> Vec<&Pin> {
        self.pins.iter().filter(|p| p.id == id).collect()
    }

    /// Clauses no task pins at all.
    #[must_use]
    pub fn unclaimed(&self) -> Vec<&Clause> {
        self.clauses
            .iter()
            .filter(|c| self.covering(&c.id).is_empty())
            .collect()
    }

    /// Clauses whose every pin is a dropped task's.
    #[must_use]
    pub fn orphaned(&self) -> Vec<(&Clause, Vec<&Pin>)> {
        self.clauses
            .iter()
            .filter_map(|c| {
                let pins = self.covering(&c.id);
                (!pins.is_empty() && pins.iter().all(|p| p.status.is_dropped()))
                    .then_some((c, pins))
            })
            .collect()
    }

    /// Pins whose hash no longer matches the clause: `live` selects the
    /// live tasks (stale) or the done ones (re-open candidates).
    #[must_use]
    pub fn drifted(&self, live: bool) -> Vec<(&Pin, &Clause)> {
        self.pins
            .iter()
            .filter(|p| p.status.is_live() == live && !p.status.is_dropped())
            .filter_map(|p| {
                let clause = self.clauses.iter().find(|c| c.id == p.id)?;
                (p.sha.as_deref() != Some(clause.sha.as_str())).then_some((p, clause))
            })
            .collect()
    }

    /// Pins of non-dropped tasks naming a clause the document no longer
    /// carries.
    #[must_use]
    pub fn dangling(&self) -> Vec<&Pin> {
        self.pins
            .iter()
            .filter(|p| !p.status.is_dropped() && self.clauses.iter().all(|c| c.id != p.id))
            .collect()
    }
}

/// The document `doc` names — a repo-relative path in `home`, or
/// `repo#path` — split into the `(repo, path)` identity every pin's ref
/// is compared against. A bare path with no home is refused: the union
/// has no repo to relate it to.
fn doc_identity(home: &str, doc: &str) -> Result<(String, String), String> {
    match doc.split_once('#') {
        Some((repo, path)) if !repo.contains('/') && !repo.contains('.') => {
            Ok((repo.to_string(), path.to_string()))
        }
        _ if home.is_empty() => Err(format!(
            "`{doc}` names no repo — over the union a spec document is `<repo>#<path>`"
        )),
        _ => Ok((home.to_string(), doc.to_string())),
    }
}

/// Audit `doc` over `stores`: read it (through the registry for a
/// `repo#path`, confined like a `docs:` link), take its clauses, and
/// collect every pin whose ref names the same document.
///
/// # Errors
/// The document does not read, or names no repo over the union.
pub fn audit(
    root: &Path,
    home: &str,
    stores: &[crate::store::RepoStore],
    doc: &str,
) -> Result<Audit, String> {
    let identity = doc_identity(home, doc)?;
    let content = crate::docs::read_target(root, doc).map_err(|e| e.to_string())?;
    let mut pins = Vec::new();
    for store in stores {
        for entry in &store.entries {
            let crate::parse::ParsedTask::Valid(t) = &entry.parsed else {
                continue;
            };
            for c in &t.covers {
                let Ok(r) = parse_ref(&c.reference) else {
                    continue;
                };
                let Ok(names) = doc_identity(&store.repo, &r.doc) else {
                    continue;
                };
                if names != identity {
                    continue;
                }
                pins.push(Pin {
                    gid: store.gid(&t.id),
                    status: t.status,
                    reference: c.reference.clone(),
                    id: r.id,
                    sha: c.sha.clone().filter(|s| is_sha(s)),
                });
            }
        }
    }
    // Live pins first, then done, then dropped, then by gid — the order a
    // reader wants to see coverage in, and a stable one.
    let rank = |s: crate::parse::Status| match s {
        crate::parse::Status::Open => 0,
        crate::parse::Status::Doing => 1,
        crate::parse::Status::Blocked => 2,
        crate::parse::Status::Done => 3,
        crate::parse::Status::Dropped => 4,
    };
    pins.sort_by(|a, b| (rank(a.status), &a.gid).cmp(&(rank(b.status), &b.gid)));
    Ok(Audit {
        doc: doc.to_string(),
        clauses: clauses(&content),
        pins,
    })
}

/// The live tasks the spec moved under (MW-T10): ids of those carrying
/// at least one real pin whose clause resolves and hashes differently
/// now, in the order given. A pin that does not resolve is not drift.
#[must_use]
pub fn moved_under(root: &Path, tasks: &[&crate::parse::Task]) -> Vec<String> {
    tasks
        .iter()
        .filter(|t| t.status.is_live())
        .filter(|t| {
            t.covers.iter().any(|c| {
                c.sha.as_deref().is_some_and(is_sha)
                    && resolve(root, &c.reference).is_ok_and(|clause| Some(clause.sha) != c.sha)
            })
        })
        .map(|t| t.id.clone())
        .collect()
}

/// `## Heading text {#sp-slug}` → `("Heading text", "sp-slug")`; None
/// when the heading carries no anchor.
fn split_anchor(heading: &str) -> Option<(&str, &str)> {
    let trimmed = heading.trim_end();
    let open = trimmed.rfind("{#")?;
    let id = trimmed[open + 2..].strip_suffix('}')?;
    is_anchor_id(id).then(|| (&trimmed[..open], id))
}

/// Right-trim every line, drop leading and trailing blank lines, join
/// with `\n` — so trailing whitespace and a moved blank line never read
/// as drift while any word change does.
fn normalize(section: &str) -> String {
    let lines: Vec<&str> = section.lines().map(str::trim_end).collect();
    let first = lines.iter().position(|l| !l.is_empty());
    let last = lines.iter().rposition(|l| !l.is_empty());
    match (first, last) {
        (Some(a), Some(b)) => lines[a..=b].join("\n"),
        _ => String::new(),
    }
}

/// Every heading outside fenced code: (byte offset, level, the full line).
fn headings(content: &str) -> Vec<(usize, usize, &str)> {
    let mut out = Vec::new();
    let mut in_fence = false;
    let mut offset = 0;
    for line in content.split_inclusive('\n') {
        let at = offset;
        offset += line.len();
        let text = line.trim_end_matches(['\n', '\r']);
        if text.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let hashes = text.len() - text.trim_start_matches('#').len();
        if hashes > 0 && text[hashes..].starts_with(' ') {
            out.push((at, hashes, line));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = "# Spec\n\n## Close gate {#sp-close-gate}\n\nA task closes on exit 0.  \n\n### Detail\n\nsub text\n\n## Plain heading\n\nnot a clause\n\n```\n## Fenced {#sp-fenced}\n```\n\n## Waive {#sp-waive}\nloud.\n";

    #[test]
    fn anchored_headings_become_clauses() {
        let all = clauses(DOC);
        let ids: Vec<&str> = all.iter().map(|c| c.id.as_str()).collect();
        assert_eq!(
            ids,
            ["sp-close-gate", "sp-waive"],
            "fenced text is not a heading"
        );
        let gate = &all[0];
        assert_eq!(gate.heading, "Close gate");
        assert_eq!(
            gate.text,
            "A task closes on exit 0.\n\n### Detail\n\nsub text"
        );
        assert_eq!(gate.sha, sha_hex(&gate.text));
        assert_eq!(all[1].text, "loud.");
    }

    #[test]
    fn refs_parse_by_their_fragment() {
        let r = parse_ref("docs/SPEC.md#sp-close-gate").unwrap();
        assert_eq!(
            (r.doc.as_str(), r.id.as_str()),
            ("docs/SPEC.md", "sp-close-gate")
        );
        let r = parse_ref("leras#docs/SPEC.md#sp-x").unwrap();
        assert_eq!(r.doc, "leras#docs/SPEC.md");
        assert!(
            parse_ref("docs/SPEC.md#§-close-gate").is_err(),
            "a heading slug is not a clause"
        );
        assert!(parse_ref("docs/SPEC.md").is_err());
        assert!(parse_ref("#sp-x").is_err());
        assert!(parse_ref("docs/SPEC.md#sp-").is_err());
        assert!(parse_ref("docs/SPEC.md#sp-Upper").is_err());
    }

    #[test]
    fn sha_shape_is_the_hex_the_tool_writes() {
        assert!(is_sha(&sha_hex("x")));
        assert!(!is_sha("abc"));
        assert!(!is_sha(&sha_hex("x").to_uppercase()));
    }
}
