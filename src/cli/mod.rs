//! CLI shell (DESIGN §6 — the surface is frozen; anything not there is a
//! non-goal, enforced by `e2e::cli_surface_frozen` at PLAN 1.6). Verbs land
//! milestone by milestone; this module holds clap types + dispatch only.

mod add;
mod init;
mod show;

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

#[derive(Subcommand)]
enum Cmd {
    /// Create the meshwork/ store in the current git repo.
    Init,
    /// Create a task file and print its id.
    Add(add::AddArgs),
    /// Full single-task view; last-3 comments by default.
    Show(show::ShowArgs),
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
    };
    match result {
        Ok(()) => 0,
        Err(msg) => {
            eprintln!("meshwork: {msg}");
            1
        }
    }
}
