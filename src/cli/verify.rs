//! `meshwork verify <id>` (mw-dx4pndb, §6 ruling 2026-08-22): run the
//! task's `verify:` under close's exact §12b gate routing and report the
//! verdict — close nothing, release nothing, write nothing, not even the
//! attempt line close records. Red-first authoring and rot-sweeps get one
//! verb instead of a hand-rolled `sh -c` loop in the interactive shell —
//! the authoring-shell mismatch that ships fail-closed verifies. No
//! `--approve` here: approval stays a close-side act, and the gate's
//! refusal already names it. Single-task only, per the same ruling.

use crate::parse::{parse_task_file, ParsedTask};
use crate::store::find_task_file;

#[derive(clap::Args)]
pub(crate) struct VerifyArgs {
    /// Task id (e.g. az-k7f3).
    id: String,
}

pub(crate) fn run(args: &VerifyArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("docs").join("meshwork");
    let Some(path) = find_task_file(&tasks_dir, &args.id) else {
        return Err(format!("{} not found in {}", args.id, tasks_dir.display()));
    };
    // Any status runs: a rotted verify on a done task is the sweep's quarry.
    let task = match parse_task_file(&path) {
        ParsedTask::Valid(t) => t,
        ParsedTask::Invalid(inv) => {
            return Err(format!(
                "{} is invalid ({}) — repair it (lint --fix) first",
                args.id, inv.error
            ))
        }
    };
    let Some(verify) = &task.verify else {
        return Err(format!(
            "{} has no verify: — nothing to run; set one: \
             meshwork set {} --verify '<cmd>'",
            args.id, args.id
        ));
    };

    match super::close::routed_verdict(&root, &path, &args.id, verify, false, json)? {
        Ok(()) => {
            if json {
                crate::cli::emit_json(
                    "verify",
                    &serde_json::json!({ "id": args.id, "verify_exit": 0, "closed": false }),
                );
            } else {
                println!("{} verify exit 0 (dry run — nothing closed)", args.id);
            }
            Ok(())
        }
        Err((_, stays)) => Err(format!("{}: {stays} — nothing recorded", args.id)),
    }
}
