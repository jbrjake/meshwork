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

pub(crate) fn mirror(_args: &MirrorArgs, _json: bool) -> Result<(), String> {
    Err("mirror lands at M3 (PLAN 3.1–3.4); everything else works offline (MW-H5)".into())
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
        /// SQL over the unioned five tables.
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

pub(crate) fn import(args: &ImportArgs, _json: bool) -> Result<(), String> {
    let ImportAction::Todo { path } = &args.action;
    Err(format!(
        "import todo lands at PLAN 1.7 (next item); {} left untouched",
        path.display()
    ))
}
