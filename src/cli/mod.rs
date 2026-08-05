//! CLI shell (DESIGN §6 — the surface is frozen; anything not there is a
//! non-goal, enforced by `e2e::cli_surface_frozen` at PLAN 1.6). Verbs land
//! milestone by milestone; this module holds clap types + dispatch only.

mod add;
mod close;
mod dep;
mod graph;
mod import;
mod init;
mod lint;
mod notes;
mod prime;
mod query;
mod show;
mod stubs;
mod transition;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Task graph as markdown-in-git, queried with SQL, no database.
#[derive(Parser)]
#[command(name = "meshwork", version, about, disable_help_subcommand = true)]
struct Cli {
    /// Emit the stable, versioned JSON shape instead of text (MW-C3).
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    cmd: Cmd,
}

/// DESIGN §6, in order, complete — anything not here is a non-goal
/// (`e2e::cli_surface_frozen` enforces it).
#[derive(Subcommand)]
enum Cmd {
    /// Create the meshwork/ store in the current git repo.
    Init,
    /// Create a task file and print its id.
    Add(add::AddArgs),
    /// Full single-task view; last-3 comments by default.
    Show(show::ShowArgs),
    /// Append a comment (self-professed identity, recorded as claimed).
    Comment(notes::CommentArgs),
    /// Copy a file into attachments/<id>/ and record it.
    Attach(notes::AttachArgs),
    /// open → doing.
    Start(transition::IdArg),
    /// open|doing → blocked; demands --reason.
    Block(transition::BlockArgs),
    /// open|doing|blocked → dropped (recorded, never deleted).
    Drop(transition::IdArg),
    /// blocked|doing|done → open.
    Reopen(transition::IdArg),
    /// Run verify:, close on exit 0 only; --waive records a loud skip.
    Close(close::CloseArgs),
    /// Edge edits without opening the file.
    Dep(dep::DepArgs),
    /// Open tasks with met deps and no live children (the queue).
    Ready(query::ReadyArgs),
    /// Blocked tasks with their reasons.
    Blocked(graph::BlockedArgs),
    /// Parent hierarchy below a task, any depth, cosmetic level names.
    Tree(transition::IdArg),
    /// The frontier of actually-open blockers for a task.
    Why(transition::IdArg),
    /// Raw SQL over tasks/edges/labels/comments/repos.
    Q(query::QArgs),
    /// The ≤6KB session-start digest.
    Prime,
    /// Structural checks; --fix repairs merge damage.
    Lint(lint::LintArgs),
    /// Append-only GitHub view (M3).
    Mirror(stubs::MirrorArgs),
    /// Union of every registered repo (M2).
    Portfolio(stubs::PortfolioArgs),
    /// Migrate a TODO.md into the store.
    Import(stubs::ImportArgs),
}

/// Repo root of an initialized store, or a user-facing error.
pub(crate) fn require_store_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let Some(root) = crate::store::find_git_root(&cwd) else {
        return Err("not inside a git repo (MW-A3)".to_string());
    };
    if !root.join("meshwork").join("config.toml").exists() {
        return Err(format!(
            "no meshwork store at {} — run `meshwork init`",
            root.display()
        ));
    }
    Ok(root)
}

/// Every verb's JSON output: `{"v":1,"verb":…,"data":…}` — stable and
/// versioned with the binary (MW-C3).
pub(crate) fn emit_json(verb: &str, data: &serde_json::Value) {
    let envelope = serde_json::json!({ "v": 1, "verb": verb, "data": data });
    println!("{envelope}");
}

/// Parse argv and run; returns the process exit code.
#[must_use]
pub fn run() -> i32 {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            // clap prints help/usage itself; keep its exit semantics
            // (0 for --help/--version, 2 for usage errors).
            let code = if e.use_stderr() { 2 } else { 0 };
            e.print().ok();
            return code;
        }
    };
    let result = match &cli.cmd {
        Cmd::Init => init::run(cli.json),
        Cmd::Add(args) => add::run(args, cli.json),
        Cmd::Show(args) => show::run(args, cli.json),
        Cmd::Start(args) => transition::start(args, cli.json),
        Cmd::Block(args) => transition::block(args, cli.json),
        Cmd::Drop(args) => transition::drop(args, cli.json),
        Cmd::Reopen(args) => transition::reopen(args, cli.json),
        Cmd::Close(args) => close::run(args, cli.json),
        Cmd::Ready(args) => query::ready(args, cli.json),
        Cmd::Q(args) => query::q(args, cli.json),
        Cmd::Lint(args) => lint::run(args, cli.json),
        Cmd::Dep(args) => dep::run(args, cli.json),
        Cmd::Blocked(args) => graph::blocked(args, cli.json),
        Cmd::Tree(args) => graph::tree(args, cli.json),
        Cmd::Why(args) => graph::why(args, cli.json),
        Cmd::Comment(args) => notes::comment(args, cli.json),
        Cmd::Attach(args) => notes::attach(args, cli.json),
        Cmd::Prime => prime::run(cli.json),
        Cmd::Mirror(args) => stubs::mirror(args, cli.json),
        Cmd::Portfolio(args) => stubs::portfolio(args, cli.json),
        Cmd::Import(args) => stubs::import(args, cli.json),
    };
    match result {
        Ok(()) => 0,
        Err(msg) => {
            eprintln!("meshwork: {msg}");
            1
        }
    }
}
