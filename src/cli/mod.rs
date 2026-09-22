//! CLI shell (DESIGN §6 — the surface is frozen; anything not there is a
//! non-goal, enforced by `e2e::cli_surface_frozen` at PLAN 1.6). Verbs land
//! milestone by milestone; this module holds clap types + dispatch only.

mod add;
mod add_batch;
mod asks;
mod close;
mod dep;
mod graph;
mod import;
mod init;
mod lint;
mod notes;
mod portfolio;
mod prime;
mod prime_render;
mod pulse;
mod query;
mod search;
mod set;
mod show;
mod stats;
mod stubs;
mod transition;
mod verify;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Task graph as markdown-in-git, queried with SQL, no database.
#[derive(Parser)]
#[command(name = "meshwork", version, about, disable_help_subcommand = true)]
struct Cli {
    /// Emit stable, versioned JSON instead of text.
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
    /// Field edits without opening the file: --seq, --docs, --handoff.
    Set(set::SetArgs),
    /// Full single-task view; last-3 comments by default.
    Show(show::ShowArgs),
    /// Append a comment (self-professed identity, recorded as claimed).
    Comment(notes::CommentArgs),
    /// Copy a file into attachments/<id>/ and record it.
    Attach(notes::AttachArgs),
    /// open → doing; records an advisory claimed-by: when an identity
    /// resolves (--as, `$MESHWORK_AUTHOR`, or config `default_author`).
    Start(transition::StartArgs),
    /// open|doing → blocked; demands --reason.
    Block(transition::BlockArgs),
    /// open|doing|blocked → dropped (recorded, never deleted); an
    /// optional --reason lands on the log entry.
    Drop(transition::DropArgs),
    /// blocked|doing|done → open.
    Reopen(transition::IdArg),
    /// Run verify:, close on exit 0 only; --waive records a loud skip.
    Close(close::CloseArgs),
    /// Run a task's verify and report — close nothing, write nothing.
    Verify(verify::VerifyArgs),
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
    /// Raw SQL over tasks/edges/labels/comments/log/repos.
    Q(query::QArgs),
    /// Case-insensitive text search over titles, bodies, handoffs, comments, and logs.
    Search(search::SearchArgs),
    /// The derived projection as tables: pulse, weekly flow, close hazard,
    /// spans, lanes, top-10s, mention health, placement.
    Stats(stats::StatsArgs),
    /// Every inbound and outbound ask, uncapped, with its answer's state and age.
    Asks,
    /// The ≤6KB session-start digest.
    Prime,
    /// Structural checks; --fix repairs merge damage.
    Lint(lint::LintArgs),
    /// Append-only GitHub view (not built yet).
    Mirror(stubs::MirrorArgs),
    /// Union of every registered repo.
    Portfolio(portfolio::PortfolioArgs),
    /// Migrate a TODO.md into the store.
    Import(stubs::ImportArgs),
}

/// mw-5hrb22q: unknown verbs a session plausibly reaches for fail with a
/// did-you-mean that carries the reason and the working invocation — the
/// pilot lost a whole session's progress note to `log` answering with bare
/// usage. Suggestions only: the verb still fails and the surface stays §6
/// (MW-D4). The message is two lines TOTAL, because agents habitually pipe
/// through `tail -3`/`head -3` and whatever the error teaches must survive
/// truncation from either end. Typos of real verbs aren't listed here —
/// clap's own similarity tip already covers them.
fn forgiveness(e: &clap::Error) -> Option<String> {
    use clap::error::{ContextKind, ContextValue, ErrorKind};
    match e.kind() {
        ErrorKind::InvalidSubcommand => {
            let ContextValue::String(verb) = e.get(ContextKind::InvalidSubcommand)? else {
                return None;
            };
            verb_forgiveness(verb)
        }
        ErrorKind::UnknownArgument => {
            let ContextValue::String(arg) = e.get(ContextKind::InvalidArg)? else {
                return None;
            };
            flag_forgiveness(arg)
        }
        _ => None,
    }
}

/// The verbs sessions reach for and the reading verbs they meant. The
/// inbox guesses (mw-48mzck9: `inbox`/`addressed`/`next`/`list` typed 22
/// times) point at prime, ready and --help — never at a writing verb.
fn verb_forgiveness(verb: &str) -> Option<String> {
    let (near, hint) = match verb {
        "log" | "note" | "notes" => (
            "comment",
            "notes append via `meshwork comment <id> --as <author> \"text\"`",
        ),
        "done" | "finish" => (
            "close",
            "`meshwork close <id>` runs the task's verify: and closes on exit 0",
        ),
        "rm" | "delete" | "remove" => (
            "drop",
            "tasks are dropped (recorded forever), never deleted: `meshwork drop <id>`",
        ),
        "next" => (
            "prime",
            "`meshwork prime` names the next task; `meshwork ready` lists what is actionable",
        ),
        "inbox" | "addressed" => (
            "asks",
            "`meshwork asks` lists every inbound and outbound ask with its answer's state; \
             `meshwork prime` and `meshwork ready` show the first few",
        ),
        "help" => (
            "--help",
            "`meshwork --help` lists every verb; `meshwork <verb> --help` its flags",
        ),
        "list" | "ls" | "tasks" => (
            "ready",
            "`meshwork ready --all` lists actionable tasks; anything else is `meshwork q \"SELECT …\"`",
        ),
        // Only reachable under `portfolio`: show is single-repo.
        "show" => {
            return Some(
                "error: `portfolio show` does not exist — show is single-repo: cd into the \
                 task's repo (its id prefix names it) and `meshwork show <id>` there; the union \
                 is `meshwork portfolio q \"SELECT …\"`\n\
                 (the verb set is fixed; `meshwork portfolio --help` lists its verbs)"
                    .to_string(),
            )
        }
        _ => return None,
    };
    Some(format!(
        "error: no verb `{verb}` — did you mean `{near}`? {hint}\n\
         (the verb set is fixed; `meshwork --help` lists all of it)"
    ))
}

/// Flags that name real frontmatter keys with no flag on this verb: say
/// which door is open instead of clap's `-- --to` tip (mw-48mzck9).
fn flag_forgiveness(arg: &str) -> Option<String> {
    let field = arg.trim_start_matches('-');
    match field {
        "to" | "answers" | "relates" => Some(format!(
            "error: `--{field}` is not a flag on this verb — set it at creation \
             (`meshwork add … --{field}`) or later (`meshwork set <id> --{field}`)\n\
             (nothing is sent: the key is data in this store, read by the other side)"
        )),
        "needs" => Some(
            "error: `--needs` is not a flag on this verb — edges are edited with \
             `meshwork dep add <id> --needs <id>`, or set at creation with `meshwork add … --needs`\n\
             (`meshwork <verb> --help` lists a verb's flags)"
                .to_string(),
        ),
        "body" | "from" | "parent" | "label" | "labels" => Some(format!(
            "error: `--{field}` is not a flag on this verb — it is set at creation \
             (`meshwork add … --{field}`) or by hand-edit in the task file\n\
             (`meshwork <verb> --help` lists a verb's flags)"
        )),
        _ => None,
    }
}

/// mw-rz4ey2h (§6 ruling 2026-08-10): prose fields are what agents write
/// longest and most carefully — the pilot had a backticked handoff chunk
/// EXECUTED by the shell. `@<path>` reads the file, `-` reads stdin,
/// anything else is the literal text; the payload never transits shell
/// quoting. Trailing newline trimmed (files end with one; the stored
/// field shouldn't).
pub(crate) fn prose_payload(value: &str) -> Result<String, String> {
    use std::io::Read as _;
    let text = if value == "-" {
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("reading stdin: {e}"))?;
        buf
    } else if let Some(path) = value.strip_prefix('@') {
        std::fs::read_to_string(path).map_err(|e| {
            format!("reading @{path}: {e} — for literal text starting with '@', pipe it via `-`")
        })?
    } else {
        value.to_string()
    };
    Ok(text.trim_end().to_string())
}

/// Repo root of an initialized store, or a user-facing error.
pub(crate) fn require_store_root() -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let Some(root) = crate::store::find_git_root(&cwd) else {
        return Err("not inside a git repo — meshwork stores live in one".to_string());
    };
    if !root
        .join("docs")
        .join("meshwork")
        .join("config.toml")
        .exists()
    {
        return Err(format!(
            "no meshwork store at {} — run `meshwork init`",
            root.display()
        ));
    }
    Ok(root)
}

/// Every verb's JSON output:
/// `{"meshwork":{"version":…,"schema":…},"verb":…,"data":…}` — identity
/// travels in-band (mw-5kp033j, amending MW-C3): per-repo version pinning
/// makes cross-repo aggregation of mixed binaries the NORMAL case, so the
/// stream itself must say who produced it. `schema` is the old `v`, and
/// it IS the store format version (mw-5rgq9ka): one contract, one number,
/// stated in FORMAT.md Versioning — never a second literal to drift.
pub(crate) fn emit_json(verb: &str, data: &serde_json::Value) {
    let envelope = serde_json::json!({
        "meshwork": {
            "version": env!("CARGO_PKG_VERSION"),
            "schema": crate::store::STORE_FORMAT,
        },
        "verb": verb,
        "data": data,
    });
    println!("{envelope}");
}

/// Terminal-safe task content (mw-8fmsws3, DESIGN §12b adjacent): strip
/// C0 and C1 controls — keeping `\n` and `\t` — at render time only;
/// files keep their bytes as written. Task text renders to the
/// operator's terminal and into hook-injected agent context, where raw
/// ESC/CSI/OSC is spoofing (or prompt) surface. JSON mode needs none of
/// this: serde escapes controls by construction.
pub(crate) fn sanitize(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect()
}

/// Mint-side twin of `sanitize` (mw-3tzfqmq): YAML forbids raw control
/// characters, so text the CLI is about to bind into frontmatter refuses
/// them at the verb — writing them would mint a file the strict parser
/// rejects, a success report over an invalid row. `multiline` keeps
/// `\n`/`\t` legal for prose blocks (handoff, body); single-line fields
/// normalize newlines before calling.
pub(crate) fn reject_controls(field: &str, text: &str, multiline: bool) -> Result<(), String> {
    match text
        .chars()
        .find(|c| c.is_control() && !(multiline && (*c == '\n' || *c == '\t')))
    {
        Some(c) => Err(format!(
            "{field} contains a control character (U+{:04X}) — a raw \
             escape would make the task file unparseable; remove it and retry",
            c as u32
        )),
        None => Ok(()),
    }
}

/// Parse argv and run; returns the process exit code.
#[must_use]
pub fn run() -> i32 {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            if let Some(short) = forgiveness(&e) {
                eprintln!("{short}");
                return 2;
            }
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
        Cmd::Set(args) => set::run(args, cli.json),
        Cmd::Show(args) => show::run(args, cli.json),
        Cmd::Start(args) => transition::start(args, cli.json),
        Cmd::Block(args) => transition::block(args, cli.json),
        Cmd::Drop(args) => transition::drop(args, cli.json),
        Cmd::Reopen(args) => transition::reopen(args, cli.json),
        Cmd::Close(args) => close::run(args, cli.json),
        Cmd::Verify(args) => verify::run(args, cli.json),
        Cmd::Ready(args) => query::ready(args, cli.json),
        Cmd::Q(args) => query::q(args, cli.json),
        Cmd::Search(args) => search::run(args, cli.json),
        Cmd::Stats(args) => stats::run(args, cli.json),
        Cmd::Asks => asks::run(cli.json),
        Cmd::Lint(args) => lint::run(args, cli.json),
        Cmd::Dep(args) => dep::run(args, cli.json),
        Cmd::Blocked(args) => graph::blocked(args, cli.json),
        Cmd::Tree(args) => graph::tree(args, cli.json),
        Cmd::Why(args) => graph::why(args, cli.json),
        Cmd::Comment(args) => notes::comment(args, cli.json),
        Cmd::Attach(args) => notes::attach(args, cli.json),
        Cmd::Prime => prime::run(cli.json),
        Cmd::Mirror(args) => stubs::mirror(args, cli.json),
        Cmd::Portfolio(args) => portfolio::run(args, cli.json),
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
