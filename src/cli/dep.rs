//! `meshwork dep add / dep rm <a> --needs <b>` (PLAN 1.1; MW-B1): edge
//! edits without opening the file. One frontmatter line changes; guardrails
//! refuse self-deps, duplicates, and dangling same-repo targets (cycles
//! stay lint's job, MW-B2).

use crate::edit::set_list;
use crate::parse::{parse_task_file, ParsedTask};
use crate::store::find_task_file;

#[derive(clap::Args)]
pub(crate) struct DepArgs {
    #[command(subcommand)]
    action: DepAction,
}

#[derive(clap::Subcommand)]
enum DepAction {
    /// Add a hard dependency: <a> needs <b>.
    Add(EdgeArgs),
    /// Remove a hard dependency.
    Rm(EdgeArgs),
}

#[derive(clap::Args)]
struct EdgeArgs {
    /// The depending task.
    a: String,
    /// The dependency target (`repo#id` crosses repos).
    #[arg(long, value_name = "ID")]
    needs: Option<String>,
    /// A target typed positionally (`dep add A B`) — caught so the
    /// refusal can model the flag (mw-48mzck9).
    #[arg(hide = true)]
    positional: Vec<String>,
}

pub(crate) fn run(args: &DepArgs, json: bool) -> Result<(), String> {
    let (edge, adding) = match &args.action {
        DepAction::Add(e) => (e, true),
        DepAction::Rm(e) => (e, false),
    };
    let verb_word = if adding { "add" } else { "rm" };
    let target = match (&edge.needs, edge.positional.first()) {
        (Some(t), None) => t,
        (_, Some(p)) => {
            return Err(format!(
                "dep {verb_word} takes the target as a flag — did you mean: \
                 meshwork dep {verb_word} {} --needs {p}",
                edge.a
            ))
        }
        (None, None) => {
            return Err(format!(
                "dep {verb_word} needs a target: meshwork dep {verb_word} {} --needs <id>",
                edge.a
            ))
        }
    };
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("docs").join("meshwork");
    let Some(path) = find_task_file(&tasks_dir, &edge.a) else {
        return Err(format!("{} not found", edge.a));
    };
    let task = match parse_task_file(&path) {
        ParsedTask::Valid(t) => t,
        ParsedTask::Invalid(inv) => {
            return Err(format!(
                "{} is invalid ({}) — repair first",
                edge.a, inv.error
            ))
        }
    };

    let mut needs = task.needs.clone();
    if adding {
        if target == &edge.a {
            return Err(format!("{} cannot need itself", edge.a));
        }
        if needs.contains(target) {
            return Err(format!("{} already needs {target}", edge.a));
        }
        // Same-repo targets must exist; repo#id targets are the registry's
        // business (MW-B3) and stay unchecked here.
        if !target.contains('#') && find_task_file(&tasks_dir, target).is_none() {
            return Err(format!("needs target `{target}` not found in this repo"));
        }
        needs.push(target.clone());
    } else {
        let before = needs.len();
        needs.retain(|n| n != target);
        if needs.len() == before {
            return Err(format!("{} does not need {target}", edge.a));
        }
    }

    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    // set_list/remove_scalar are block-aware: a hand-written or batch-
    // imported block-style `needs:` collapses cleanly instead of leaving
    // its old `  - item` lines stranded under the new flow line.
    let text = if needs.is_empty() {
        crate::edit::remove_scalar(&text, "needs")?
    } else {
        set_list(&text, "needs", &needs)?
    };
    std::fs::write(&path, text).map_err(|e| e.to_string())?;

    if json {
        crate::cli::emit_json(
            "dep",
            &serde_json::json!({
                "action": verb_word, "a": edge.a, "needs": target,
                "now": needs,
            }),
        );
    } else {
        // The success line models the flag, so the next call is a copy.
        let did = if adding { "added" } else { "removed" };
        println!(
            "{} --needs {target} {did} (now: [{}])",
            edge.a,
            needs.join(", ")
        );
    }
    Ok(())
}
