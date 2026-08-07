//! `meshwork lint [--fix]` (PLAN 0.9): report every structural finding;
//! `--fix` repairs exactly the mechanical damage — union-merge duplicate
//! keys and post-merge duplicate IDs (MW-A4/I2). Modeling errors (cycles,
//! missing reasons, dangling refs) stay human problems.

use crate::edit::{append_section_entry, set_scalar};
use crate::id::{mint_unique, IdGen};
use crate::lint::{lint_store, Severity};
use crate::parse::{ParsedTask, Status};
use crate::store::{load_repo, RepoStore};
use std::path::Path;

#[derive(clap::Args)]
pub(crate) struct LintArgs {
    /// Repair union-poisoned duplicate keys and re-slug duplicate IDs.
    #[arg(long)]
    fix: bool,
}

pub(crate) fn run(args: &LintArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let mut store = load_repo(&root).map_err(|e| e.to_string())?;

    if args.fix {
        let repairs =
            fix_duplicate_keys(&store)? + fix_duplicate_ids(&store)? + fix_misplaced(&store)?;
        if repairs > 0 && !json {
            println!("fixed {repairs} file(s)");
        }
        store = load_repo(&root).map_err(|e| e.to_string())?;
    }

    let findings = lint_store(&store);
    let errors = findings
        .iter()
        .filter(|f| f.severity == Severity::Error)
        .count();
    let warnings = findings.len() - errors;

    if json {
        let list: Vec<_> = findings
            .iter()
            .map(|f| {
                serde_json::json!({
                    "severity": f.severity.as_str(),
                    "code": f.code,
                    "subject": f.subject,
                    "message": f.message,
                })
            })
            .collect();
        crate::cli::emit_json(
            "lint",
            &serde_json::json!({ "errors": errors, "warnings": warnings, "findings": list }),
        );
    } else {
        for f in &findings {
            println!(
                "{}[{}] {}: {}",
                f.severity.as_str(),
                f.code,
                f.subject,
                f.message
            );
        }
        println!("{errors} error(s), {warnings} warning(s)");
    }
    if errors > 0 {
        Err(format!("{errors} lint error(s)"))
    } else {
        Ok(())
    }
}

/// Union merge's signature damage: the same top-level key twice. Keep the
/// first value, drop the rest, log the repair in the task (MW-I1/I2).
/// Move terminal tasks into `archive/` and live ones back out
/// (mw-45e2qf4) — the `misplaced` warning's mechanical repair.
fn fix_misplaced(store: &RepoStore) -> Result<usize, String> {
    let tasks_dir = crate::store::tasks_dir(&store.root);
    let mut moved = 0;
    for entry in &store.entries {
        let ParsedTask::Valid(t) = &entry.parsed else {
            continue;
        };
        let terminal = matches!(t.status, Status::Done | Status::Dropped);
        let in_archive = entry.file_name.starts_with("archive/");
        if terminal != in_archive {
            let path = tasks_dir.join(&entry.file_name);
            crate::store::relocate_for_status(&path, terminal).map_err(|e| e.to_string())?;
            moved += 1;
        }
    }
    Ok(moved)
}

fn fix_duplicate_keys(store: &RepoStore) -> Result<usize, String> {
    let today = crate::clock::stamp();
    let mut fixed = 0;
    for entry in &store.entries {
        let ParsedTask::Invalid(inv) = &entry.parsed else {
            continue;
        };
        if !inv.error.contains("duplicate frontmatter key") {
            continue;
        }
        let path = store
            .root
            .join("docs")
            .join("meshwork")
            .join(&entry.file_name);
        let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let Some((repaired, dropped)) = drop_duplicate_keys(&text) else {
            continue;
        };
        let mut repaired = repaired;
        for line in &dropped {
            repaired = append_section_entry(
                &repaired,
                "log",
                &format!("{today} lint --fix: dropped duplicate `{line}` (union merge; kept the first value)"),
            );
        }
        std::fs::write(&path, repaired).map_err(|e| e.to_string())?;
        fixed += 1;
    }
    Ok(fixed)
}

/// Remove second-and-later occurrences of top-level keys in the
/// frontmatter; returns the repaired text and the dropped lines.
fn drop_duplicate_keys(text: &str) -> Option<(String, Vec<String>)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let (fm, tail) = (&rest[..end], &rest[end..]);

    let mut seen = Vec::new();
    let mut kept = Vec::new();
    let mut dropped = Vec::new();
    for line in fm.lines() {
        let key = line
            .chars()
            .next()
            .filter(char::is_ascii_alphabetic)
            .and_then(|_| line.split(':').next())
            .map(ToString::to_string);
        match key {
            Some(k) if seen.contains(&k) => dropped.push(line.to_string()),
            Some(k) => {
                seen.push(k);
                kept.push(line);
            }
            None => kept.push(line),
        }
    }
    if dropped.is_empty() {
        return None;
    }
    Some((format!("---\n{}{tail}", kept.join("\n")), dropped))
}

/// Post-merge duplicate IDs (MW-A4): the earliest side (created date, then
/// file name) keeps the id — references made before the collision resolve
/// to it — later sides get fresh ids. Inbound refs are reported, since no
/// file content can say which side a reference meant.
fn fix_duplicate_ids(store: &RepoStore) -> Result<usize, String> {
    let tasks_dir = store.root.join("docs").join("meshwork");
    let today = crate::clock::stamp();
    let seed = std::env::var("MESHWORK_ID_SEED").ok();
    let mut gen = IdGen::from_seed_str(seed.as_deref());

    let mut groups: std::collections::BTreeMap<String, Vec<(&String, Option<String>)>> =
        std::collections::BTreeMap::new();
    for entry in &store.entries {
        if let ParsedTask::Valid(t) = &entry.parsed {
            groups
                .entry(t.id.clone())
                .or_default()
                .push((&entry.file_name, t.created.clone()));
        }
    }

    let mut fixed = 0;
    for (old_id, mut files) in groups {
        if files.len() < 2 {
            continue;
        }
        files.sort_by(|a, b| {
            let key_a = (a.1.clone().unwrap_or_else(|| "9999".into()), a.0.clone());
            let key_b = (b.1.clone().unwrap_or_else(|| "9999".into()), b.0.clone());
            key_a.cmp(&key_b)
        });
        for (file_name, _) in files.iter().skip(1) {
            let new_id = mint_unique(&store.config.alias, &tasks_dir, &mut gen)
                .map_err(|e| e.to_string())?;
            reslug(&tasks_dir, file_name, &old_id, &new_id, &today)?;
            fixed += 1;
        }
        let inbound = count_inbound(store, &old_id);
        if inbound > 0 {
            eprintln!(
                "note: {inbound} same-repo reference(s) to `{old_id}` now resolve to the \
                 keeper file — review that they meant it"
            );
        }
    }
    Ok(fixed)
}

fn reslug(
    tasks_dir: &Path,
    file_name: &str,
    old_id: &str,
    new_id: &str,
    today: &str,
) -> Result<(), String> {
    let old_path = tasks_dir.join(file_name);
    let text = std::fs::read_to_string(&old_path).map_err(|e| e.to_string())?;
    let text = set_scalar(&text, "id", Some(new_id))?;
    let text = append_section_entry(
        &text,
        "log",
        &format!("{today} lint --fix: re-slugged from {old_id} (post-merge duplicate, MW-A4)"),
    );
    let new_name = file_name.replacen(old_id, new_id, 1);
    std::fs::write(tasks_dir.join(&new_name), text).map_err(|e| e.to_string())?;
    std::fs::remove_file(&old_path).map_err(|e| e.to_string())?;
    Ok(())
}

fn count_inbound(store: &RepoStore, id: &str) -> usize {
    store
        .entries
        .iter()
        .filter_map(|e| match &e.parsed {
            ParsedTask::Valid(t) if t.id != id => Some(t),
            _ => None,
        })
        .map(|t| {
            let one = |v: &Option<String>| usize::from(v.as_deref() == Some(id));
            t.needs.iter().filter(|n| *n == id).count()
                + t.relates.iter().filter(|n| *n == id).count()
                + one(&t.parent)
                + one(&t.discovered_from)
        })
        .sum()
}
