//! Archive bundles (mw-bvxpeef; FORMAT.md Store layout, format 2): past a
//! loose-file threshold, `lint --fix` concatenates archived task files
//! into `archive/bundle-NNNN.md`, each a plain sequence of task documents
//! capped by bytes. A bundle is append-friendly under `merge=union`; a
//! live task is never bundled; `reopen` splits a document back out. The
//! [`Located`] seam lets every verb read and write one task whether it is
//! a file of its own or a document inside a bundle.

use crate::grammar::Fence;
use crate::parse::{parse_bundled, parse_task_str, ParsedTask, Status};
use std::path::{Path, PathBuf};

/// A bundle stops taking documents once it would pass this many bytes
/// (DESIGN §1): well under any hosting limit, large enough that a store
/// of thousands of closed tasks is a handful of files.
pub const BUNDLE_BYTES: usize = 512 * 1024;

/// Loose archived files before lint says so and `--fix` bundles them: the
/// step is a format bump the first time, so it is never a side effect of
/// a routine fix run on a small archive.
pub const LOOSE_THRESHOLD: usize = 100;

const BUNDLE_PREFIX: &str = "bundle-";

/// `bundle-NNNN.md` — the one archive file name that is not a task.
#[must_use]
pub fn is_bundle_name(name: &str) -> bool {
    name.strip_prefix(BUNDLE_PREFIX)
        .and_then(|rest| rest.strip_suffix(".md"))
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// A store-relative entry name (`archive/bundle-0001.md`) that is a bundle.
#[must_use]
pub fn is_bundle_path(file_name: &str) -> bool {
    file_name
        .strip_prefix("archive/")
        .is_some_and(is_bundle_name)
}

/// One document of a bundle: its byte span and the id its frontmatter opens with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Doc {
    /// Byte offset of the opening `---` line.
    pub start: usize,
    /// Byte offset where the next document starts (or the text ends).
    pub end: usize,
    /// The `id:` value on the line after the opener.
    pub id: String,
}

/// Split a bundle into documents. A document opens at a top-level `---`
/// line directly followed by an `id:` line; its frontmatter runs to the
/// next `---`; its body runs to the next opener. A `---` inside a fenced
/// block, or one not followed by `id:` (a horizontal rule), is body.
#[must_use]
pub fn split_bundle(text: &str) -> Vec<Doc> {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut docs: Vec<Doc> = Vec::new();
    let mut in_frontmatter = false;
    let mut fence = Fence::default();
    let mut offset = 0usize;
    for (i, raw) in lines.iter().enumerate() {
        let line = raw.trim_end_matches(['\n', '\r']);
        let start = offset;
        offset += raw.len();
        if in_frontmatter {
            if line == "---" {
                in_frontmatter = false;
            }
            continue;
        }
        if line == "---" && !fence.in_fence() {
            let next = lines
                .get(i + 1)
                .map(|n| n.trim_end_matches(['\n', '\r']))
                .and_then(|n| n.strip_prefix("id:"));
            if let Some(id) = next {
                if let Some(last) = docs.last_mut() {
                    last.end = start;
                }
                docs.push(Doc {
                    start,
                    end: text.len(),
                    id: id.trim().to_string(),
                });
                in_frontmatter = true;
                fence = Fence::default();
                continue;
            }
        }
        if !docs.is_empty() {
            fence.observe(line);
        }
    }
    docs
}

/// A document's text, trailing blank lines trimmed to one newline.
fn doc_text(bundle: &str, doc: &Doc) -> String {
    let mut s = bundle[doc.start..doc.end]
        .trim_end_matches('\n')
        .to_string();
    s.push('\n');
    s
}

/// The bundle text for a sequence of documents: each ending in one
/// newline, a blank line between.
fn join_docs(docs: &[String]) -> String {
    docs.iter()
        .map(|d| format!("{}\n", d.trim_end_matches('\n')))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One task document is clean to bundle: it opens with its own id, splits
/// as exactly one document, and closes every fence — an unclosed fence
/// would swallow whatever is appended after it.
fn clean_doc(text: &str, id: &str) -> bool {
    let docs = split_bundle(text);
    if docs.len() != 1 || docs[0].id != id || docs[0].start != 0 {
        return false;
    }
    let mut fence = Fence::default();
    let mut in_frontmatter = true;
    for line in text.lines().skip(1) {
        if in_frontmatter {
            if line == "---" {
                in_frontmatter = false;
            }
            continue;
        }
        fence.observe(line);
    }
    !fence.in_fence()
}

/// The bundle files of an archive dir, name-sorted; none when absent.
fn bundles_in(archive_dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(archive_dir)
        .into_iter()
        .flatten()
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| is_bundle_name(&n.to_string_lossy()))
        })
        .collect();
    out.sort();
    out
}

/// Archived task files that are not bundles, name-sorted.
#[must_use]
pub fn loose_singles(archive_dir: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = std::fs::read_dir(archive_dir)
        .into_iter()
        .flatten()
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|e| e.eq_ignore_ascii_case("md"))
                && p.file_name()
                    .is_some_and(|n| !is_bundle_name(&n.to_string_lossy()))
        })
        .collect();
    out.sort();
    out
}

/// The bundle holding `id`, if any.
#[must_use]
pub fn bundle_holding(archive_dir: &Path, id: &str) -> Option<PathBuf> {
    bundles_in(archive_dir).into_iter().find(|p| {
        std::fs::read_to_string(p).is_ok_and(|t| split_bundle(&t).iter().any(|d| d.id == id))
    })
}

/// Every id any bundle claims — minting must never reuse one.
#[must_use]
pub fn ids_in_bundles(archive_dir: &Path) -> Vec<String> {
    bundles_in(archive_dir)
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .flat_map(|t| split_bundle(&t).into_iter().map(|d| d.id))
        .collect()
}

/// Every document of a bundle, parsed, with the id each claims — the
/// loader's view. An empty or unsplittable bundle is one invalid row.
#[must_use]
pub fn load_bundle(entry_name: &str, text: &str) -> Vec<ParsedTask> {
    let docs = split_bundle(text);
    if docs.is_empty() {
        return vec![ParsedTask::Invalid(crate::parse::Invalid {
            id: entry_name
                .rsplit('/')
                .next()
                .unwrap_or(entry_name)
                .trim_end_matches(".md")
                .to_string(),
            file_name: entry_name.to_string(),
            error: "bundle holds no task documents".to_string(),
        })];
    }
    docs.iter()
        .map(|d| parse_bundled(entry_name, &d.id, &doc_text(text, d)))
        .collect()
}

/// Where a task's text lives: a file of its own, or a document inside a
/// bundle. Reads and writes go through here so no verb needs to know.
#[derive(Debug, Clone)]
pub struct Located {
    path: PathBuf,
    id: String,
    bundled: bool,
}

impl Located {
    /// The file holding the task — the bundle, when bundled.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// True when the task is a document inside a bundle.
    #[must_use]
    pub fn bundled(&self) -> bool {
        self.bundled
    }

    /// The task's own document text.
    ///
    /// # Errors
    /// The file cannot be read, or the bundle no longer holds the id.
    pub fn read(&self) -> Result<String, String> {
        let text = std::fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        if !self.bundled {
            return Ok(text);
        }
        split_bundle(&text)
            .iter()
            .find(|d| d.id == self.id)
            .map(|d| doc_text(&text, d))
            .ok_or_else(|| format!("{} no longer holds {}", self.path.display(), self.id))
    }

    /// Replace the task's document — the whole file, or its span in the
    /// bundle, re-found by id so earlier edits to the bundle never shift it.
    ///
    /// # Errors
    /// I/O, or the bundle no longer holds the id.
    pub fn write(&self, text: &str) -> Result<(), String> {
        if !self.bundled {
            return std::fs::write(&self.path, text).map_err(|e| e.to_string());
        }
        let bundle = std::fs::read_to_string(&self.path).map_err(|e| e.to_string())?;
        let docs = split_bundle(&bundle);
        if !docs.iter().any(|d| d.id == self.id) {
            return Err(format!(
                "{} no longer holds {}",
                self.path.display(),
                self.id
            ));
        }
        let parts: Vec<String> = docs
            .iter()
            .map(|d| {
                if d.id == self.id {
                    text.to_string()
                } else {
                    doc_text(&bundle, d)
                }
            })
            .collect();
        std::fs::write(&self.path, join_docs(&parts)).map_err(|e| e.to_string())
    }

    /// The task, parsed — from its own file or its bundle document.
    #[must_use]
    pub fn parse(&self) -> ParsedTask {
        let name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match self.read() {
            Ok(text) if self.bundled => parse_bundled(&name, &self.id, &text),
            Ok(text) => parse_task_str(&name, &text),
            Err(e) => ParsedTask::Invalid(crate::parse::Invalid {
                id: self.id.clone(),
                file_name: name,
                error: format!("unreadable: {e}"),
            }),
        }
    }

    /// The path, for a verb that must move or replace the file whole.
    ///
    /// # Errors
    /// The task is bundled — the verb names the way out.
    pub fn require_single(&self, action: &str) -> Result<&Path, String> {
        if self.bundled {
            return Err(format!(
                "cannot {action} {}: it sits inside an archive bundle ({}) — `reopen` splits \
                 it back out into a file of its own",
                self.id,
                self.path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            ));
        }
        Ok(&self.path)
    }
}

impl Located {
    /// The location a loaded entry came from — no directory scan; the
    /// loader already knows which file, and whether it is a bundle.
    #[must_use]
    pub fn for_entry(tasks_dir: &Path, file_name: &str, id: &str) -> Self {
        Self {
            path: tasks_dir.join(file_name),
            id: id.to_string(),
            bundled: is_bundle_path(file_name),
        }
    }
}

/// Find a task by id: its own file first, then the bundles.
#[must_use]
pub fn locate(tasks_dir: &Path, id: &str) -> Option<Located> {
    if let Some(path) = crate::store::find_single(tasks_dir, id) {
        return Some(Located {
            path,
            id: id.to_string(),
            bundled: false,
        });
    }
    bundle_holding(&tasks_dir.join(crate::store::ARCHIVE_SUBDIR), id).map(|path| Located {
        path,
        id: id.to_string(),
        bundled: true,
    })
}

/// What one compaction did.
#[derive(Debug, Default)]
pub struct Compacted {
    /// Loose files folded into bundles.
    pub bundled: usize,
    /// Bundle file names written to.
    pub bundles: Vec<String>,
    /// Files left loose, with why.
    pub skipped: Vec<(String, String)>,
}

/// Fold every loose archived file into bundles, name order, filling the
/// last bundle before opening the next. A file that is not a clean,
/// terminal task document stays loose and is named. The store's format
/// becomes 2 the first time a bundle exists.
///
/// # Errors
/// I/O writing a bundle, removing a folded file, or editing the config.
pub fn compact(tasks_dir: &Path) -> Result<Compacted, String> {
    let archive_dir = tasks_dir.join(crate::store::ARCHIVE_SUBDIR);
    let mut report = Compacted::default();
    let existing = bundles_in(&archive_dir);
    let mut number = existing
        .last()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
        .and_then(|stem| stem.strip_prefix(BUNDLE_PREFIX)?.parse::<u32>().ok())
        .unwrap_or(0);
    let mut current: Option<(PathBuf, String)> = existing
        .last()
        .and_then(|p| std::fs::read_to_string(p).ok().map(|t| (p.clone(), t)));
    for single in loose_singles(&archive_dir) {
        let name = single
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let text = std::fs::read_to_string(&single).map_err(|e| e.to_string())?;
        let id = match parse_task_str(&name, &text) {
            ParsedTask::Valid(t) if matches!(t.status, Status::Done | Status::Dropped) => t.id,
            ParsedTask::Valid(t) => {
                report
                    .skipped
                    .push((name, format!("{} — not terminal", t.status.as_str())));
                continue;
            }
            ParsedTask::Invalid(inv) => {
                report.skipped.push((name, inv.error));
                continue;
            }
        };
        let doc = format!("{}\n", text.trim_end_matches('\n'));
        if !clean_doc(&doc, &id) {
            report.skipped.push((
                name,
                "does not split back out whole (an unclosed fence, or the id line is not first)"
                    .to_string(),
            ));
            continue;
        }
        let fits = current
            .as_ref()
            .is_some_and(|(_, t)| t.len() + 1 + doc.len() <= BUNDLE_BYTES);
        if !fits {
            number += 1;
            let path = archive_dir.join(format!("{BUNDLE_PREFIX}{number:04}.md"));
            current = Some((path, String::new()));
        }
        let Some((path, bundle)) = current.as_mut() else {
            unreachable!("a bundle was just opened")
        };
        let mut candidate = bundle.trim_end_matches('\n').to_string();
        if !candidate.is_empty() {
            candidate.push_str("\n\n");
        }
        candidate.push_str(&doc);
        let before = split_bundle(bundle).len();
        if split_bundle(&candidate).len() != before + 1 {
            report.skipped.push((
                name,
                "appending it would not split back out whole".to_string(),
            ));
            continue;
        }
        std::fs::write(&*path, &candidate).map_err(|e| e.to_string())?;
        std::fs::remove_file(&single).map_err(|e| e.to_string())?;
        *bundle = candidate;
        let bundle_name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !report.bundles.contains(&bundle_name) {
            report.bundles.push(bundle_name);
        }
        report.bundled += 1;
    }
    if report.bundled > 0 {
        bump_format(tasks_dir)?;
    }
    Ok(report)
}

/// Split a bundled task back out into `archive/<id>-<slug>.md` — the file
/// `reopen` then relocates, or `lint --fix` moves — and drop the bundle
/// when that emptied it.
///
/// # Errors
/// No bundle holds the id, the single already exists, or I/O.
pub fn extract(tasks_dir: &Path, id: &str) -> Result<PathBuf, String> {
    let archive_dir = tasks_dir.join(crate::store::ARCHIVE_SUBDIR);
    let Some(bundle_path) = bundle_holding(&archive_dir, id) else {
        return Err(format!("no archive bundle holds {id}"));
    };
    let bundle = std::fs::read_to_string(&bundle_path).map_err(|e| e.to_string())?;
    let docs = split_bundle(&bundle);
    let (mine, rest): (Vec<&Doc>, Vec<&Doc>) = docs.iter().partition(|d| d.id == id);
    let Some(doc) = mine.first() else {
        return Err(format!("no archive bundle holds {id}"));
    };
    let text = doc_text(&bundle, doc);
    let title = text
        .lines()
        .skip(1)
        .take_while(|l| *l != "---")
        .find_map(|l| l.strip_prefix("title:"))
        .map(|t| t.trim().trim_matches('"').trim_matches('\'').to_string())
        .unwrap_or_default();
    let slug = crate::id::slugify(&title);
    let single = archive_dir.join(if slug.is_empty() {
        format!("{id}.md")
    } else {
        format!("{id}-{slug}.md")
    });
    if single.exists() {
        return Err(format!(
            "{} already exists beside the bundle holding {id} — a duplicate; lint --fix",
            single.display()
        ));
    }
    std::fs::write(&single, &text).map_err(|e| e.to_string())?;
    if rest.is_empty() {
        std::fs::remove_file(&bundle_path).map_err(|e| e.to_string())?;
    } else {
        let parts: Vec<String> = rest.iter().map(|d| doc_text(&bundle, d)).collect();
        std::fs::write(&bundle_path, join_docs(&parts)).map_err(|e| e.to_string())?;
    }
    Ok(single)
}

/// Declare format 2 in `config.toml` when the store says less.
fn bump_format(tasks_dir: &Path) -> Result<(), String> {
    let path = tasks_dir.join("config.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let current: u64 = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("format"))
        .and_then(|rest| rest.trim().strip_prefix('='))
        .and_then(|v| v.split('#').next()?.trim().parse().ok())
        .unwrap_or(1);
    if current >= 2 {
        return Ok(());
    }
    let mut out = String::new();
    let mut placed = false;
    for line in text.lines() {
        if line.trim_start().starts_with("format") && line.contains('=') {
            out.push_str("format = 2\n");
            placed = true;
        } else {
            out.push_str(line);
            out.push('\n');
            if !placed && line.trim_start().starts_with("alias") {
                out.push_str("format = 2\n");
                placed = true;
            }
        }
    }
    if !placed {
        out.push_str("format = 2\n");
    }
    std::fs::write(&path, out).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(id: &str, body: &str) -> String {
        format!("---\nid: {id}\ntitle: T {id}\nstatus: done\nverify: \"true\"\n---\n{body}\n")
    }

    #[test]
    fn split_ignores_fenced_and_bare_rules() {
        let a = doc(
            "zz-a1",
            "text\n\n```\n---\nid: fenced\n```\n\n---\n\nafter the rule",
        );
        let b = doc("zz-b2", "second");
        let bundle = join_docs(&[a.clone(), b.clone()]);
        let docs = split_bundle(&bundle);
        assert_eq!(docs.len(), 2, "{docs:?}");
        assert_eq!(docs[0].id, "zz-a1");
        assert_eq!(docs[1].id, "zz-b2");
        assert_eq!(doc_text(&bundle, &docs[0]), a);
        assert_eq!(doc_text(&bundle, &docs[1]), b);
        assert!(clean_doc(&a, "zz-a1"));
        assert!(!clean_doc(&doc("zz-c3", "```\nnever closed"), "zz-c3"));
        assert!(!clean_doc(&a, "zz-b2"));
    }

    #[test]
    fn bundle_names() {
        assert!(is_bundle_name("bundle-0001.md"));
        assert!(is_bundle_name("bundle-12.md"));
        assert!(!is_bundle_name("bundle-.md"));
        assert!(!is_bundle_name("bundle-0001.txt"));
        assert!(!is_bundle_name("zz-bundle-0001.md"));
        assert!(is_bundle_path("archive/bundle-0001.md"));
        assert!(!is_bundle_path("bundle-0001.md"));
    }

    #[test]
    fn compact_extract_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let tasks = dir.path().join("docs/meshwork");
        let archive = tasks.join("archive");
        std::fs::create_dir_all(&archive).unwrap();
        std::fs::write(tasks.join("config.toml"), "alias = \"zz\"\nformat = 1\n").unwrap();
        std::fs::write(archive.join("zz-a1-t.md"), doc("zz-a1", "one")).unwrap();
        std::fs::write(archive.join("zz-b2-t.md"), doc("zz-b2", "two")).unwrap();
        std::fs::write(
            archive.join("zz-l3-t.md"),
            doc("zz-l3", "live").replace("status: done", "status: open"),
        )
        .unwrap();
        let report = compact(&tasks).unwrap();
        assert_eq!(report.bundled, 2);
        assert_eq!(report.skipped.len(), 1, "{:?}", report.skipped);
        assert_eq!(report.bundles, vec!["bundle-0001.md".to_string()]);
        assert!(archive.join("zz-l3-t.md").is_file(), "live stays loose");
        let cfg = std::fs::read_to_string(tasks.join("config.toml")).unwrap();
        assert!(cfg.contains("format = 2"), "{cfg}");

        let loc = locate(&tasks, "zz-b2").unwrap();
        assert!(loc.bundled());
        assert_eq!(loc.read().unwrap(), doc("zz-b2", "two"));
        loc.write(&doc("zz-b2", "two, edited")).unwrap();
        assert_eq!(
            locate(&tasks, "zz-a1").unwrap().read().unwrap(),
            doc("zz-a1", "one")
        );
        assert!(ids_in_bundles(&archive).contains(&"zz-b2".to_string()));

        let single = extract(&tasks, "zz-a1").unwrap();
        assert_eq!(single, archive.join("zz-a1-t-zz-a1.md"));
        assert_eq!(
            std::fs::read_to_string(&single).unwrap(),
            doc("zz-a1", "one")
        );
        let bundle = std::fs::read_to_string(archive.join("bundle-0001.md")).unwrap();
        assert_eq!(split_bundle(&bundle).len(), 1);
        assert!(bundle.contains("two, edited"));
        extract(&tasks, "zz-b2").unwrap();
        assert!(
            !archive.join("bundle-0001.md").exists(),
            "an emptied bundle goes"
        );
    }
}
