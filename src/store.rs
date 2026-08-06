//! Store discovery and loading (DESIGN §1, §3): a repo's `meshwork/` layout
//! read into memory — config plus every task file parsed (never skipped;
//! failures ride along as [`ParsedTask::Invalid`], MW-I2).

use crate::parse::{parse_task_file, ParsedTask};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Errors loading a store. Parse failures are NOT here — they're data
/// (invalid rows), not errors.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The directory has no `docs/meshwork/config.toml`.
    #[error("not a meshwork store: {0} (missing docs/meshwork/config.toml — run `meshwork init`)")]
    NotAStore(PathBuf),
    /// Filesystem failure walking the store.
    #[error("reading store: {0}")]
    Io(#[from] std::io::Error),
    /// `config.toml` exists but does not parse.
    #[error("bad config {path}: {message}")]
    BadConfig {
        /// Path to the offending config.toml.
        path: PathBuf,
        /// TOML parse error text.
        message: String,
    },
}

/// `docs/meshwork/config.toml` (DESIGN §1). Unknown keys are ignored by serde —
/// config is not the strict surface task files are.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Repo alias used as the ID prefix (`az` → `az-k7f3`).
    pub alias: String,
    /// Fallback comment author (MW-K1 chain: `--as` → env → this → error).
    #[serde(default)]
    pub default_author: Option<String>,
    /// Cosmetic category-depth names (MW-B8); zero semantics.
    #[serde(default)]
    pub hierarchy: Option<Hierarchy>,
    /// Mirror opt-in (MW-H1); absent = off.
    #[serde(default)]
    pub mirror: Option<bool>,
}

/// `[hierarchy]` table — display names only (MW-B8).
#[derive(Debug, Clone, Deserialize)]
pub struct Hierarchy {
    /// Display names for category depths, outermost first.
    #[serde(default)]
    pub levels: Vec<String>,
}

/// One task file: name + parse outcome, in filename order.
#[derive(Debug, Clone)]
pub struct TaskEntry {
    /// File name within `docs/meshwork/`.
    pub file_name: String,
    /// Parse outcome (valid task or loud invalid).
    pub parsed: ParsedTask,
}

/// A loaded repo store.
#[derive(Debug, Clone)]
pub struct RepoStore {
    /// Registry name; defaults to the repo directory's name (DESIGN §4 gid
    /// prefix — the portfolio registry uses the same default, MW-G2).
    pub repo: String,
    /// Repo root (the directory containing `meshwork/`).
    pub root: PathBuf,
    /// Parsed config.
    pub config: Config,
    /// Every `tasks/*.md`, filename-sorted.
    pub entries: Vec<TaskEntry>,
}

impl RepoStore {
    /// Global ID (`repo#id`) for a task ID in this store.
    #[must_use]
    pub fn gid(&self, id: &str) -> String {
        format!("{}#{id}", self.repo)
    }
}

/// Task files live in the store dir itself — `docs/meshwork/`, flat
/// (mw-acgp). Shared so verbs never re-derive the layout.
#[must_use]
pub fn tasks_dir(root: &Path) -> PathBuf {
    root.join("docs").join("meshwork")
}

/// Where terminal (done/dropped) task files live (mw-45e2qf4). Archived
/// tasks stay fully loaded and queryable — only the clutter moves.
pub const ARCHIVE_SUBDIR: &str = "archive";

/// Move a task file into (`terminal`) or out of (live) the archive,
/// returning its new path; a file already in the right place is untouched.
///
/// # Errors
/// I/O failures creating the target dir or renaming.
pub fn relocate_for_status(path: &Path, terminal: bool) -> std::io::Result<PathBuf> {
    let parent = path
        .parent()
        .ok_or_else(|| std::io::Error::other("task file has no parent dir"))?;
    let in_archive = parent.file_name().is_some_and(|n| n == ARCHIVE_SUBDIR);
    if terminal == in_archive {
        return Ok(path.to_path_buf());
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| std::io::Error::other("task file has no file name"))?
        .to_owned();
    let target_dir = if terminal {
        parent.join(ARCHIVE_SUBDIR)
    } else {
        parent
            .parent()
            .ok_or_else(|| std::io::Error::other("archive dir has no parent"))?
            .to_path_buf()
    };
    std::fs::create_dir_all(&target_dir)?;
    let target = target_dir.join(file_name);
    std::fs::rename(path, &target)?;
    Ok(target)
}

/// Load just the config of a store.
///
/// # Errors
/// [`StoreError::NotAStore`] when `docs/meshwork/config.toml` is missing,
/// [`StoreError::BadConfig`] when it doesn't parse.
pub fn load_config(root: &Path) -> Result<Config, StoreError> {
    let config_path = root.join("docs").join("meshwork").join("config.toml");
    let config_text = match std::fs::read_to_string(&config_path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(StoreError::NotAStore(root.to_path_buf()))
        }
        Err(e) => return Err(e.into()),
    };
    toml::from_str(&config_text).map_err(|e| StoreError::BadConfig {
        path: config_path,
        message: e.to_string(),
    })
}

/// Load a repo's store from its root directory.
///
/// # Errors
/// [`StoreError::NotAStore`] when `docs/meshwork/config.toml` is missing,
/// [`StoreError::BadConfig`] when it doesn't parse, or I/O failures walking
/// `docs/meshwork/`. Task-file parse failures never error — they load as
/// invalid entries (MW-I2).
pub fn load_repo(root: &Path) -> Result<RepoStore, StoreError> {
    let config = load_config(root)?;
    let repo = repo_name(root);
    let tasks_dir = root.join("docs").join("meshwork");
    let mut entries = Vec::new();
    // Root + archive/ every invocation (mw-45e2qf4): archived tasks stay in
    // every table so needs-resolution and history are location-blind.
    for (dir, prefix) in [
        (tasks_dir.clone(), ""),
        (tasks_dir.join(ARCHIVE_SUBDIR), "archive/"),
    ] {
        match std::fs::read_dir(&dir) {
            Ok(dir) => {
                for entry in dir {
                    let entry = entry?;
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let is_md = entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
                    if is_md {
                        entries.push(TaskEntry {
                            parsed: parse_task_file(&entry.path()),
                            file_name: format!("{prefix}{name}"),
                        });
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {} // fresh store
            Err(e) => return Err(e.into()),
        }
    }
    entries.sort_by(|a, b| a.file_name.cmp(&b.file_name));

    Ok(RepoStore {
        repo,
        root: root.to_path_buf(),
        config,
        entries,
    })
}

/// Walk up from `start` to the directory containing `.git` (dir, or the
/// file a linked worktree uses).
#[must_use]
pub fn find_git_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Locate a task file by ID: `<id>.md` or the `<id>-<slug>.md` glob —
/// the ID prefix exists precisely for this lookup (DESIGN §2/§5).
#[must_use]
pub fn find_task_file(tasks_dir: &Path, id: &str) -> Option<PathBuf> {
    find_in_dir(tasks_dir, id).or_else(|| find_in_dir(&tasks_dir.join(ARCHIVE_SUBDIR), id))
}

fn find_in_dir(dir: &Path, id: &str) -> Option<PathBuf> {
    let exact = format!("{id}.md");
    let prefix = format!("{id}-");
    let mut hits: Vec<PathBuf> = std::fs::read_dir(dir)
        .ok()?
        .filter_map(std::result::Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().is_some_and(|n| {
                let n = n.to_string_lossy();
                n == exact || (n.starts_with(&prefix) && n.ends_with(".md"))
            })
        })
        .collect();
    hits.sort();
    hits.into_iter().next()
}

/// Registry name for a repo root: its directory name (canonicalized so `.`
/// works), falling back to `repo`.
fn repo_name(root: &Path) -> String {
    let canonical = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    canonical
        .file_name()
        .map_or_else(|| "repo".to_string(), |n| n.to_string_lossy().into_owned())
}
