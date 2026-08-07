//! `meshwork init` (PLAN 0.4; MW-A3/I1): write the store layout at the git
//! toplevel. Never installs hooks, never writes outside the repo; the
//! committed `merge=union` attribute is the whole concurrency mechanism.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// Files `init` writes, relative to the repo root. The store is flat —
/// task files live in `docs/meshwork/` itself (mw-acgp) with terminal
/// tasks under `archive/` (mw-45e2qf4); the union attribute anchors to
/// both (`/*.md`, gitignore-style).
const GITATTRIBUTES: &str = "/*.md merge=union\n/archive/*.md merge=union\n";
const CACHE_GITIGNORE: &str = "*\n!.gitignore\n";

pub(crate) fn run(json: bool) -> Result<(), String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let Some(root) = find_git_root(&cwd) else {
        return Err(
            "not inside a git repo — meshwork stores travel with a repo (MW-A3); \
             run `git init` first"
                .to_string(),
        );
    };
    let mw = root.join("docs").join("meshwork");
    if mw.join("config.toml").exists() {
        return Err(format!(
            "already initialized: {} exists",
            mw.join("config.toml").display()
        ));
    }

    let alias = default_alias(&root);
    let author = git_user_name(&root);
    let mut config = String::new();
    config.push_str("# meshwork store config (hand-editable; DESIGN §1).\n");
    config.push_str("# alias prefixes every task ID — pick it before the first `add`;\n");
    config.push_str("# IDs embed it forever.\n");
    let _ = writeln!(config, "alias = \"{alias}\"");
    // Format marker (mw-n6nvzpa): absent = 1; newer than the binary knows
    // is refused loudly. Never minted unmarked.
    let _ = writeln!(config, "format = {}", crate::store::STORE_FORMAT);
    if let Some(author) = &author {
        let _ = writeln!(config, "default_author = \"{author}\"");
    }
    config.push_str("\n# Cosmetic names for category depths (MW-B8) — uncomment to taste:\n");
    config.push_str("# [hierarchy]\n# levels = [\"saga\", \"epic\", \"sprint\", \"story\"]\n");

    let created = [
        ("docs/meshwork/config.toml", Some(config.as_str())),
        ("docs/meshwork/.gitattributes", Some(GITATTRIBUTES)),
        ("docs/meshwork/.cache/.gitignore", Some(CACHE_GITIGNORE)),
        ("docs/meshwork/attachments", None),
    ];
    for (rel, content) in created {
        let path = root.join(rel);
        match content {
            Some(text) => {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                std::fs::write(&path, text).map_err(|e| e.to_string())?;
            }
            None => std::fs::create_dir_all(&path).map_err(|e| e.to_string())?,
        }
    }

    if json {
        crate::cli::emit_json(
            "init",
            &serde_json::json!({
                "root": root.display().to_string(),
                "alias": alias,
                "created": created.iter().map(|(rel, _)| rel).collect::<Vec<_>>(),
            }),
        );
    } else {
        println!("initialized meshwork store at {}", mw.display());
        for (rel, _) in created {
            println!("  {rel}");
        }
        println!(
            "edit docs/meshwork/config.toml (alias `{alias}`) before the first `add`, then commit."
        );
    }
    Ok(())
}

/// Walk up to the directory containing `.git` (dir or worktree file).
fn find_git_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(".git").exists())
        .map(Path::to_path_buf)
}

/// Default alias: first two alphanumeric chars of the repo directory name,
/// lowercased. A starting point, not a contract — config is hand-editable.
fn default_alias(root: &Path) -> String {
    let name = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let alias: String = name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(2)
        .collect::<String>()
        .to_lowercase();
    if alias.is_empty() {
        "mw".to_string()
    } else {
        alias
    }
}

/// Seed `default_author` from git's user.name when configured (MW-K1 chain).
fn git_user_name(root: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["config", "user.name"])
        .current_dir(root)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!name.is_empty()).then_some(name)
}
