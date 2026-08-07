//! Frozen-surface verbs whose behavior lands in later milestones (DESIGN
//! §6 is complete from day one; unbuilt verbs error honestly instead of
//! pretending). `mirror` arrives at M3, `portfolio` at M2.

use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct MirrorArgs {
    #[command(subcommand)]
    action: MirrorAction,
}

#[derive(clap::Subcommand)]
enum MirrorAction {
    /// Create/append the GitHub view; dry-run unless --yes (MW-H1).
    Push {
        /// Actually push (default is a dry run).
        #[arg(long)]
        yes: bool,
    },
    /// Report local↔remote drift; never writes (MW-H4).
    Status,
}

pub(crate) fn mirror(args: &MirrorArgs, _json: bool) -> Result<(), String> {
    // The branch guard rules before M3 builds the push path (mw-pvfrpd4)
    // — the guard's contract must not be shaped by what's convenient for
    // the implementation that comes later.
    if let MirrorAction::Push { .. } = &args.action {
        branch_guard()?;
    }
    Err("mirror lands at M3 (PLAN 3.1–3.4); everything else works offline (MW-H5)".into())
}

/// mw-pvfrpd4: mirror is append-only and unretractable, so `push` refuses
/// off the repo's default branch — feature-branch state may rebase away
/// or never merge. Default = the local `origin/HEAD` ref (zero network,
/// MW-J6); indeterminate refuses too. `[mirror] allow_non_default = true`
/// skips the guard loudly.
fn branch_guard() -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let store = crate::store::load_repo(&root).map_err(|e| e.to_string())?;
    let git = |args: &[&str]| -> Option<String> {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(&root)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    let current = git(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "HEAD".into());

    if store.config.mirror_allow_non_default() {
        println!(
            "mirror: allow_non_default set \u{2014} pushing from `{current}` \
             (append-only is unretractable, mw-pvfrpd4)"
        );
        return Ok(());
    }
    match git(&["symbolic-ref", "--short", "refs/remotes/origin/HEAD"]) {
        Some(head) => {
            let default = head.strip_prefix("origin/").unwrap_or(&head);
            if default == current {
                Ok(())
            } else {
                Err(format!(
                    "mirror push refused: on `{current}`, default branch is `{default}` \
                     \u{2014} append-only publishes are unretractable (mw-pvfrpd4); merge \
                     first, or set [mirror] allow_non_default = true (loud) in \
                     docs/meshwork/config.toml"
                ))
            }
        }
        None => Err(format!(
            "mirror push refused: cannot determine the default branch (no origin/HEAD \
             ref; on `{current}`) \u{2014} fix: git remote set-head origin <branch>, or \
             set [mirror] allow_non_default = true (loud) in docs/meshwork/config.toml"
        )),
    }
}

#[derive(clap::Args)]
pub(crate) struct PortfolioArgs {
    #[command(subcommand)]
    action: PortfolioAction,
}

#[derive(clap::Subcommand)]
enum PortfolioAction {
    /// Ready across every registered repo.
    Ready,
    /// First task by the total ordering (sequence.md → registry → per-repo).
    Next,
    /// Raw SQL over the unioned portfolio tables.
    Q {
        /// SQL over the unioned six tables.
        sql: String,
    },
    /// Renumber seq weights when gaps exhaust (§15.2).
    Seq,
}

pub(crate) fn portfolio(_args: &PortfolioArgs, _json: bool) -> Result<(), String> {
    Err("portfolio lands at M2 (PLAN 2.1–2.4); single-repo verbs are complete".into())
}

#[derive(clap::Args)]
pub(crate) struct ImportArgs {
    #[command(subcommand)]
    action: ImportAction,
}

#[derive(clap::Subcommand)]
enum ImportAction {
    /// Convert a baseline-checkbox TODO.md into task files (MW-J3).
    Todo {
        /// Path to the TODO.md to import.
        path: PathBuf,
    },
}

pub(crate) fn import(args: &ImportArgs, json: bool) -> Result<(), String> {
    let ImportAction::Todo { path } = &args.action;
    super::import::todo(path, json)
}
