//! Post-merge duplicate ids (the `duplicate-id` finding) and `lint --fix`'s
//! repair of them: which side keeps the id, and the rewrite of the
//! references the other side's clone authored (MW-A4).
//!
//! Two clones can mint one id. After the merge both files carry it, so an
//! inbound edge names the id and never a side. Git history is what defines
//! a side: the commit that introduced a reference held exactly one of the
//! duplicate files — the one its author could see — or both (the reference
//! postdates the merge) or neither (it was never committed). The side with
//! the most references so attributed keeps the id; ties go to the side that
//! entered history first, then the earlier `created:`, then the file name,
//! so an uncommitted copy never takes the id from a committed original.
//! Every other side is re-slugged and the references attributed to it are
//! rewritten to the new id, each rewrite logged in the file it touches.
//! References attributed to neither side are reported, never rewritten. A
//! store git has never seen attributes nothing, and the created/file-name
//! rule decides alone (owner ruling 2026-10-04, mw-6k73vyj).

use crate::archive::Located;
use crate::edit::{append_section_entry, replace_block_item, set_scalar};
use crate::id::{mint_unique, IdGen};
use crate::parse::ParsedTask;
use crate::store::RepoStore;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The store directory as git names it, relative to the repo root.
const STORE_PREFIX: &str = "docs/meshwork";

/// What the repair did: files rewritten, and the notes a person should read.
pub struct Repaired {
    /// Files written: each re-slugged side and each referencer rewritten.
    pub files: usize,
    /// What `--fix` could not settle — a bundled duplicate it left alone, a
    /// reference history could not place.
    pub notes: Vec<String>,
}

/// One file carrying the duplicated id.
struct Side {
    file_name: String,
    created: Option<String>,
    /// Committer time of the commit that added the file; `None` when git
    /// has never seen it.
    first_commit: Option<u64>,
    /// Indexes into the group's inbound list: the references history
    /// places on this side.
    refs: Vec<usize>,
}

/// One same-repo task referencing the duplicated id, and through which
/// frontmatter keys.
struct Inbound {
    file_name: String,
    id: String,
    keys: Vec<&'static str>,
}

/// Re-slug every duplicate side but the keeper and rewrite the references
/// history attributes to it.
///
/// # Errors
/// Minting, reading or writing a task file fails.
pub fn fix_duplicate_ids(store: &RepoStore) -> Result<Repaired, String> {
    let tasks_dir = crate::store::tasks_dir(&store.root);
    let today = crate::clock::stamp();
    let seed = std::env::var("MESHWORK_ID_SEED").ok();
    let mut gen = IdGen::from_seed_str(seed.as_deref());
    let mut repaired = Repaired {
        files: 0,
        notes: Vec::new(),
    };

    for (old_id, mut sides) in groups(store) {
        if sides.len() < 2 {
            continue;
        }
        let inbound = inbound_refs(store, &old_id);
        attribute(&store.root, &old_id, &mut sides, &inbound);
        sides.sort_by_key(|s| {
            (
                std::cmp::Reverse(s.refs.len()),
                s.first_commit.unwrap_or(u64::MAX),
                s.created.clone().unwrap_or_else(|| "9999".into()),
                s.file_name.clone(),
            )
        });
        let keeper = &sides[0];
        let keeper_name = keeper.file_name.clone();
        let keeper_refs = keeper.refs.len();
        for side in &sides[1..] {
            if crate::archive::is_bundle_path(&side.file_name) {
                repaired.notes.push(format!(
                    "`{old_id}` is also inside {} — a bundled duplicate is not re-slugged; \
                     reopen and drop the copy you do not want",
                    side.file_name
                ));
                continue;
            }
            let new_id = mint_unique(&store.config.alias, &tasks_dir, &mut gen)
                .map_err(|e| e.to_string())?;
            reslug(&tasks_dir, side, &old_id, &new_id, keeper_refs, &today)?;
            repaired.files += 1;
            for &i in &side.refs {
                rewrite_reference(&tasks_dir, &inbound[i], &old_id, &new_id, &today)?;
                repaired.files += 1;
            }
        }
        let placed: usize = sides.iter().map(|s| s.refs.len()).sum();
        let unplaced = inbound.len() - placed;
        if unplaced > 0 {
            repaired.notes.push(format!(
                "{unplaced} same-repo reference(s) to `{old_id}` could not be placed by \
                 history and now resolve to {keeper_name} — review that they meant it"
            ));
        }
    }
    Ok(repaired)
}

/// Valid tasks grouped by id, each as a side with no attribution yet.
fn groups(store: &RepoStore) -> BTreeMap<String, Vec<Side>> {
    let mut groups: BTreeMap<String, Vec<Side>> = BTreeMap::new();
    for entry in &store.entries {
        if let ParsedTask::Valid(t) = &entry.parsed {
            groups.entry(t.id.clone()).or_default().push(Side {
                file_name: entry.file_name.clone(),
                created: t.created.clone(),
                first_commit: None,
                refs: Vec::new(),
            });
        }
    }
    groups
}

/// Every same-repo task naming `id` through an edge key, one entry per
/// referencing file. Cross-repo references live in other stores and are
/// never seen here, let alone rewritten.
fn inbound_refs(store: &RepoStore, id: &str) -> Vec<Inbound> {
    store
        .entries
        .iter()
        .filter_map(|e| match &e.parsed {
            ParsedTask::Valid(t) if t.id != id => Some((e, t)),
            _ => None,
        })
        .filter_map(|(e, t)| {
            let mut keys = Vec::new();
            if t.needs.iter().any(|n| n == id) {
                keys.push("needs");
            }
            if t.relates.iter().any(|n| n == id) {
                keys.push("relates");
            }
            if t.parent.as_deref() == Some(id) {
                keys.push("parent");
            }
            if t.discovered_from.as_deref() == Some(id) {
                keys.push("discovered-from");
            }
            (!keys.is_empty()).then(|| Inbound {
                file_name: e.file_name.clone(),
                id: t.id.clone(),
                keys,
            })
        })
        .collect()
}

/// Place each inbound reference on the side its introducing commit could
/// see, and stamp each side with when it entered history. Any git failure
/// — no git, no commits, a file git never saw — places nothing.
fn attribute(root: &Path, id: &str, sides: &mut [Side], inbound: &[Inbound]) {
    for side in sides.iter_mut() {
        side.first_commit = first_commit_time(root, &side.file_name);
    }
    let mut trees: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (i, r) in inbound.iter().enumerate() {
        let Some(sha) = introducing_commit(root, &r.file_name, id) else {
            continue;
        };
        let present = trees
            .entry(sha.clone())
            .or_insert_with(|| basenames_in_tree(root, &sha).unwrap_or_default());
        let mut seen: Vec<usize> = sides
            .iter()
            .enumerate()
            .filter(|(_, s)| present.contains(basename(&s.file_name)))
            .map(|(j, _)| j)
            .collect();
        if let (Some(j), None) = (seen.pop(), seen.pop()) {
            sides[j].refs.push(i);
        }
    }
}

fn basename(file_name: &str) -> &str {
    file_name.rsplit('/').next().unwrap_or(file_name)
}

fn store_path(file_name: &str) -> String {
    format!("{STORE_PREFIX}/{file_name}")
}

/// The oldest commit in which the file's count of `id` changed — where the
/// reference was written.
fn introducing_commit(root: &Path, file_name: &str, id: &str) -> Option<String> {
    let pickaxe = format!("-S{id}");
    let rel = store_path(file_name);
    git_lines(
        root,
        &["log", "--follow", "--format=%H", &pickaxe, "--", &rel],
    )?
    .pop()
}

/// Committer time of the commit that added the file, following renames.
fn first_commit_time(root: &Path, file_name: &str) -> Option<u64> {
    let rel = store_path(file_name);
    git_lines(
        root,
        &[
            "log",
            "--follow",
            "--diff-filter=A",
            "--format=%ct",
            "--",
            &rel,
        ],
    )?
    .pop()?
    .trim()
    .parse()
    .ok()
}

/// Base names of every store file in a commit's tree.
fn basenames_in_tree(root: &Path, sha: &str) -> Option<BTreeSet<String>> {
    Some(
        git_lines(
            root,
            &["ls-tree", "-r", "--name-only", sha, "--", STORE_PREFIX],
        )?
        .iter()
        .map(|p| basename(p).to_string())
        .collect(),
    )
}

/// Stdout lines of a git command, or `None` on any failure.
fn git_lines(root: &Path, args: &[&str]) -> Option<Vec<String>> {
    let out = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect(),
    )
}

/// Give one side a fresh id — frontmatter, file name, and a log line
/// saying why this side lost.
fn reslug(
    tasks_dir: &Path,
    side: &Side,
    old_id: &str,
    new_id: &str,
    keeper_refs: usize,
    today: &str,
) -> Result<(), String> {
    let old_path = tasks_dir.join(&side.file_name);
    let text = std::fs::read_to_string(&old_path).map_err(|e| e.to_string())?;
    let text = set_scalar(&text, "id", Some(new_id))?;
    let text = append_section_entry(
        &text,
        "log",
        &format!(
            "{today} lint --fix: re-slugged from {old_id} (post-merge duplicate; history \
             placed {} reference(s) on this side, {keeper_refs} on the keeper)",
            side.refs.len()
        ),
    );
    let new_name = side.file_name.replacen(old_id, new_id, 1);
    std::fs::write(tasks_dir.join(&new_name), text).map_err(|e| e.to_string())?;
    std::fs::remove_file(&old_path).map_err(|e| e.to_string())?;
    Ok(())
}

/// Point one referencing task at the re-slugged side's new id, through
/// every key that named the old one, and log the rewrite there.
fn rewrite_reference(
    tasks_dir: &Path,
    r: &Inbound,
    old_id: &str,
    new_id: &str,
    today: &str,
) -> Result<(), String> {
    let located = Located::for_entry(tasks_dir, &r.file_name, &r.id);
    let mut text = located.read()?;
    for key in &r.keys {
        text = match *key {
            "needs" | "relates" => replace_block_item(&text, key, old_id, new_id)?,
            _ => set_scalar(&text, key, Some(new_id))?,
        };
    }
    let text = append_section_entry(
        &text,
        "log",
        &format!(
            "{today} lint --fix: {} now {new_id}, was {old_id} (the side it meant was \
             re-slugged after a post-merge duplicate)",
            r.keys.join(", ")
        ),
    );
    located.write(&text)
}
