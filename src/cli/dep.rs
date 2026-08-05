//! `meshwork dep add / dep rm <a> --needs <b>` (PLAN 1.1; MW-B1): edge
//! edits without opening the file. One frontmatter line changes; guardrails
//! refuse self-deps, duplicates, and dangling same-repo targets (cycles
//! stay lint's job, MW-B2).

use crate::edit::set_scalar;
use crate::parse::{parse_task_file, ParsedTask};
use crate::store::find_task_file;
use crate::write::yaml_scalar;

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
    needs: String,
}

pub(crate) fn run(args: &DepArgs, json: bool) -> Result<(), String> {
    let (edge, adding) = match &args.action {
        DepAction::Add(e) => (e, true),
        DepAction::Rm(e) => (e, false),
    };
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("meshwork").join("tasks");
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

    let target = &edge.needs;
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
    let text = if needs.is_empty() {
        crate::edit::remove_scalar(&text, "needs")?
    } else {
        let list = needs
            .iter()
            .map(|n| yaml_scalar(n))
            .collect::<Vec<_>>()
            .join(", ");
        set_scalar(&text, "needs", Some(&format!("[{list}]")))?
    };
    std::fs::write(&path, text).map_err(|e| e.to_string())?;

    let verb_word = if adding { "add" } else { "rm" };
    if json {
        crate::cli::emit_json(
            "dep",
            &serde_json::json!({
                "action": verb_word, "a": edge.a, "needs": target,
                "now": needs,
            }),
        );
    } else {
        println!(
            "{} {verb_word} needs {target} (now: [{}])",
            edge.a,
            needs.join(", ")
        );
    }
    Ok(())
}
