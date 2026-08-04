//! `fixtures::corpus_covers_features` — keeps the committed corpus honest
//! forever (PLAN B3, MW-J4): every feature and failure mode DESIGN §13 names
//! must appear ≥1× in `fixtures/`. Checks are deliberately text-level: the
//! corpus must not depend on the parser it exists to test.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

struct TaskFile {
    name: String,
    fm: String,
    body: String,
}

fn read_tasks(repo: &str) -> Vec<TaskFile> {
    let dir = fixtures_root().join(repo).join("meshwork").join("tasks");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("missing corpus dir {}: {e}", dir.display()));
    let mut out = Vec::new();
    for entry in entries {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "md") {
            let text = fs::read_to_string(&path).unwrap();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            match split_frontmatter(&text) {
                Some((fm, body)) => out.push(TaskFile {
                    name,
                    fm: fm.to_string(),
                    body: body.to_string(),
                }),
                // The unparseable-YAML fixture still has fences; a file with
                // no fences at all would be a corpus bug.
                None => panic!("{name}: no frontmatter fences"),
            }
        }
    }
    assert!(!out.is_empty(), "no task files under {}", dir.display());
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    Some((&rest[..end], &rest[end + 5..]))
}

/// Value of a top-level `key:` line, trimmed, inline ` #` comments stripped.
fn scalar<'a>(fm: &'a str, key: &str) -> Option<&'a str> {
    fm.lines().find_map(|l| {
        let v = l.strip_prefix(key)?.strip_prefix(':')?;
        let v = v.split(" #").next().unwrap_or(v).trim();
        (!v.is_empty()).then_some(v)
    })
}

/// Items of an inline list value `key: [a, b]`.
fn inline_list(fm: &str, key: &str) -> Vec<String> {
    scalar(fm, key)
        .and_then(|v| v.strip_prefix('[')?.strip_suffix(']').map(str::to_string))
        .map(|inner| {
            inner
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Top-level frontmatter keys in order (line-based, duplicates kept).
fn top_keys(fm: &str) -> Vec<String> {
    fm.lines()
        .filter_map(|l| {
            let first = l.chars().next()?;
            if !first.is_ascii_alphabetic() {
                return None;
            }
            let key = l.split(':').next()?;
            key.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
                .then(|| key.to_string())
        })
        .collect()
}

/// True if the directed graph (id → targets) has a cycle.
fn has_cycle(edges: &BTreeMap<String, Vec<String>>) -> bool {
    fn visit(
        n: &str,
        edges: &BTreeMap<String, Vec<String>>,
        path: &mut BTreeSet<String>,
        done: &mut BTreeSet<String>,
    ) -> bool {
        if path.contains(n) {
            return true;
        }
        if done.contains(n) {
            return false;
        }
        path.insert(n.to_string());
        let looped = edges
            .get(n)
            .is_some_and(|ts| ts.iter().any(|t| visit(t, edges, path, done)));
        path.remove(n);
        done.insert(n.to_string());
        looped
    }
    let mut done = BTreeSet::new();
    edges
        .keys()
        .any(|n| visit(n, edges, &mut BTreeSet::new(), &mut done))
}

fn heading_slug(line: &str) -> Option<String> {
    let text = line.strip_prefix('#')?.trim_start_matches('#').trim();
    Some(text.to_lowercase().replace(' ', "-"))
}

/// All `path#anchor` doc links found in frontmatter block lists.
fn doc_links(fm: &str) -> Vec<(String, String)> {
    fm.lines()
        .filter_map(|l| {
            let item = l.trim().strip_prefix("- ")?;
            let item = item.split(" #").next().unwrap_or(item).trim();
            let (path, anchor) = item.split_once('#')?;
            Path::new(path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
                .then(|| (path.to_string(), anchor.to_string()))
        })
        .collect()
}

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
    "waived",
];

/// DESIGN §13 alpha: the kitchen-sink repo. ~30 tasks, every feature.
fn check_alpha(missing: &mut Vec<String>) {
    let tasks = read_tasks("alpha");
    let mut need = |ok: bool, what: &str| {
        if !ok {
            missing.push(format!("alpha: {what}"));
        }
    };

    need(tasks.len() >= 25, "~30 tasks (found fewer than 25)");

    let statuses: BTreeSet<_> = tasks
        .iter()
        .filter_map(|t| scalar(&t.fm, "status"))
        .collect();
    for s in ["open", "doing", "blocked", "done", "dropped"] {
        need(statuses.contains(s), &format!("status `{s}`"));
    }

    // Every edge kind (MW-B1).
    need(
        tasks
            .iter()
            .any(|t| !inline_list(&t.fm, "needs").is_empty()),
        "a `needs` edge",
    );
    need(
        tasks.iter().any(|t| scalar(&t.fm, "parent").is_some()),
        "a `parent` edge",
    );
    need(
        tasks
            .iter()
            .any(|t| scalar(&t.fm, "discovered-from").is_some()),
        "a `discovered-from` edge",
    );
    need(
        tasks
            .iter()
            .any(|t| !inline_list(&t.fm, "relates").is_empty()),
        "a `relates` edge",
    );

    // Cross-repo needs → beta (registered) and → gamma (absent) (MW-B3/G5).
    let all_needs: Vec<String> = tasks
        .iter()
        .flat_map(|t| inline_list(&t.fm, "needs"))
        .collect();
    need(
        all_needs.iter().any(|n| n.starts_with("beta#")),
        "cross-repo needs → beta",
    );
    need(
        all_needs.iter().any(|n| n.starts_with("gamma#")),
        "needs → gamma (absent repo)",
    );

    // 5-deep parent chain (MW-B8).
    let parent_of: BTreeMap<String, String> = tasks
        .iter()
        .filter_map(|t| {
            Some((
                scalar(&t.fm, "id")?.to_string(),
                scalar(&t.fm, "parent")?.to_string(),
            ))
        })
        .collect();
    let depth = |mut id: String| {
        let mut d = 1;
        while let Some(p) = parent_of.get(&id) {
            d += 1;
            id.clone_from(p);
            if d > 32 {
                break; // corpus cycle would be its own bug
            }
        }
        d
    };
    need(
        parent_of
            .keys()
            .map(|id| depth(id.clone()))
            .max()
            .unwrap_or(0)
            >= 5,
        "5-deep parent chain (saga→epic→sprint→story→task)",
    );

    check_alpha_docs_comments(&tasks, missing);
    check_alpha_payloads(&tasks, missing);
}

/// alpha, continued: docs drill-through anchors + comment format (MW-F*, K1).
fn check_alpha_docs_comments(tasks: &[TaskFile], missing: &mut Vec<String>) {
    let mut need = |ok: bool, what: &str| {
        if !ok {
            missing.push(format!("alpha: {what}"));
        }
    };

    // docs: links with one good and one bad anchor (MW-F1/F3).
    let links: Vec<_> = tasks.iter().flat_map(|t| doc_links(&t.fm)).collect();
    let resolves = |path: &str, anchor: &str| {
        fs::read_to_string(fixtures_root().join("alpha").join(path))
            .ok()
            .is_some_and(|doc| {
                doc.lines()
                    .filter_map(heading_slug)
                    .any(|slug| slug == anchor)
            })
    };
    need(
        links.iter().any(|(p, a)| resolves(p, a)),
        "docs: link with a good anchor",
    );
    need(
        links
            .iter()
            .any(|(p, a)| fixtures_root().join("alpha").join(p).exists() && !resolves(p, a)),
        "docs: link with a bad anchor",
    );

    // Multi-author comments with continuation lines (MW-K1).
    let mut authors = BTreeSet::new();
    let mut continuation = false;
    for t in tasks {
        let Some(section) = t.body.split("## comments").nth(1) else {
            continue;
        };
        let mut in_bullet = false;
        for line in section.lines() {
            if let Some(rest) = line.strip_prefix("- ") {
                if let Some((_, tail)) = rest.split_once('[') {
                    if let Some((author, _)) = tail.split_once(']') {
                        authors.insert(author.to_string());
                    }
                }
                in_bullet = true;
            } else if in_bullet && line.starts_with("  ") && !line.trim().is_empty() {
                continuation = true;
            } else if !line.trim().is_empty() {
                in_bullet = false;
            }
        }
    }
    need(authors.len() >= 2, "comments from ≥2 distinct authors");
    need(
        continuation,
        "a comment continuation line (two-space indent)",
    );
}

/// alpha, continued: attachments, seq/verify discipline, repo config (K2/K3).
fn check_alpha_payloads(tasks: &[TaskFile], missing: &mut Vec<String>) {
    let mut need = |ok: bool, what: &str| {
        if !ok {
            missing.push(format!("alpha: {what}"));
        }
    };

    // Attachments: referenced, on disk, one >1MB for the K3 warning.
    let attachment_sizes: Vec<u64> = tasks
        .iter()
        .flat_map(|t| {
            t.fm.lines()
                .filter_map(|l| l.trim().strip_prefix("- "))
                .filter(|item| item.starts_with("attachments/"))
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .filter_map(|rel| fs::metadata(fixtures_root().join("alpha/meshwork").join(rel)).ok())
        .map(|m| m.len())
        .collect();
    need(
        !attachment_sizes.is_empty(),
        "a referenced attachment present on disk",
    );
    need(
        attachment_sizes.iter().any(|&s| s > 1_048_576),
        "an attachment >1MB (K3 warning case)",
    );

    // seq with gaps; a task with no verify (lint-warn case, MW-E2).
    let mut seqs: Vec<i64> = tasks
        .iter()
        .filter_map(|t| scalar(&t.fm, "seq")?.parse().ok())
        .collect();
    seqs.sort_unstable();
    need(
        seqs.windows(2).any(|w| w[1] - w[0] > 1),
        "seq values with gaps",
    );
    need(
        tasks
            .iter()
            .any(|t| scalar(&t.fm, "verify").is_none() && scalar(&t.fm, "status") == Some("open")),
        "an open task with no verify:",
    );

    // Cosmetic level names + committed union merge attribute (MW-B8/I1).
    let config =
        fs::read_to_string(fixtures_root().join("alpha/meshwork/config.toml")).unwrap_or_default();
    need(config.contains("levels"), "config.toml [hierarchy] levels");
    let attrs = fs::read_to_string(fixtures_root().join("alpha/meshwork/.gitattributes"))
        .unwrap_or_default();
    need(
        attrs.contains("merge=union"),
        ".gitattributes tasks/*.md merge=union",
    );
}

/// DESIGN §13 alpha-broken: every failure mode, one instance each.
fn check_alpha_broken(missing: &mut Vec<String>) {
    let tasks = read_tasks("alpha-broken");
    let mut need = |ok: bool, what: &str| {
        if !ok {
            missing.push(format!("alpha-broken: {what}"));
        }
    };

    let ids: Vec<String> = tasks
        .iter()
        .filter_map(|t| scalar(&t.fm, "id").map(str::to_string))
        .collect();
    let id_set: BTreeSet<&String> = ids.iter().collect();

    let same_repo_edges = |key: &str| -> BTreeMap<String, Vec<String>> {
        tasks
            .iter()
            .filter_map(|t| {
                let id = scalar(&t.fm, "id")?.to_string();
                let mut targets = inline_list(&t.fm, key);
                if let Some(one) = scalar(&t.fm, key) {
                    if !one.starts_with('[') {
                        targets.push(one.to_string());
                    }
                }
                targets.retain(|x| !x.contains('#'));
                Some((id, targets))
            })
            .collect()
    };

    need(has_cycle(&same_repo_edges("needs")), "needs-cycle");
    need(has_cycle(&same_repo_edges("parent")), "parent-cycle");
    need(
        tasks
            .iter()
            .any(|t| scalar(&t.fm, "parent").is_some_and(|p| p.contains('#'))),
        "cross-repo parent (lint error, MW-B3)",
    );
    need(
        tasks.iter().any(|t| {
            scalar(&t.fm, "status") == Some("blocked") && scalar(&t.fm, "blocked-reason").is_none()
        }),
        "blocked without reason",
    );
    need(ids.len() > id_set.len(), "duplicate-ID pair");
    let dup_key = |t: &TaskFile| {
        let keys = top_keys(&t.fm);
        keys.len() > keys.iter().collect::<BTreeSet<_>>().len()
    };
    need(
        tasks.iter().any(dup_key),
        "duplicate frontmatter key (post-union file)",
    );
    need(
        tasks.iter().any(|t| {
            !dup_key(t) && serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&t.fm).is_err()
        }),
        "unparseable YAML",
    );
    need(
        same_repo_edges("needs")
            .values()
            .flatten()
            .any(|target| !id_set.contains(target)),
        "dangling same-repo edge",
    );
    need(
        tasks.iter().any(|t| {
            top_keys(&t.fm)
                .iter()
                .any(|k| !KNOWN_KEYS.contains(&k.as_str()))
        }),
        "unknown frontmatter field",
    );
}

/// beta: small clean repo, target of cross-repo edges. portfolio: registry +
/// sequence overlay naming alpha, beta, and the absent gamma. golden: home of
/// byte-compared expected outputs (populated per feature via --bless).
fn check_beta_portfolio_golden(missing: &mut Vec<String>) {
    let mut need = |ok: bool, what: &str| {
        if !ok {
            missing.push(what.to_string());
        }
    };

    let beta = read_tasks("beta");
    need(
        beta.iter().any(|t| scalar(&t.fm, "status") == Some("done")),
        "beta: a done task (cross-repo needs target)",
    );
    need(
        beta.iter()
            .all(|t| serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&t.fm).is_ok()),
        "beta: all files clean YAML",
    );

    let root = fixtures_root();
    let repos = fs::read_to_string(root.join("portfolio/repos.toml")).unwrap_or_default();
    for name in ["alpha", "beta", "gamma"] {
        need(
            repos.contains(&format!("\"{name}\"")),
            &format!("portfolio: repos.toml registers `{name}`"),
        );
    }
    need(
        !root.join("gamma").exists(),
        "portfolio: gamma stays absent on disk",
    );

    let seq = fs::read_to_string(root.join("portfolio/sequence.md")).unwrap_or_default();
    need(
        seq.lines().filter(|l| l.starts_with("## ")).count() >= 2,
        "portfolio: sequence.md with ≥2 tranche headings",
    );
    need(
        seq.lines()
            .filter(|l| l.trim().starts_with("- ") && l.contains('#'))
            .count()
            >= 2,
        "portfolio: sequence.md with ≥2 repo#id refs",
    );

    need(root.join("golden").is_dir(), "golden/ directory");
}

#[test]
fn corpus_covers_features() {
    let mut missing = Vec::new();
    check_alpha(&mut missing);
    check_alpha_broken(&mut missing);
    check_beta_portfolio_golden(&mut missing);
    assert!(
        missing.is_empty(),
        "corpus gaps (DESIGN §13):\n  {}",
        missing.join("\n  ")
    );
}
