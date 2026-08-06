//! `meshwork close` (PLAN 0.7; MW-E2): run `verify:` via `sh -c` from the
//! repo root, record exit + date in the log, close on exit 0 only.
//! `--waive` skips the check but is recorded — loud and queryable.

use crate::edit::{append_section_entry, set_scalar};
use crate::parse::{parse_task_file, ParsedTask, Status};
use crate::store::find_task_file;
use crate::write::yaml_scalar;

#[derive(clap::Args)]
pub(crate) struct CloseArgs {
    /// Task id (e.g. az-k7f3).
    id: String,
    /// Close without running verify; the reason lands in the `waived`
    /// column (`WHERE waived IS NOT NULL`, §15.3).
    #[arg(long, value_name = "REASON")]
    waive: Option<String>,
}

pub(crate) fn run(args: &CloseArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("docs").join("meshwork");
    let Some(path) = find_task_file(&tasks_dir, &args.id) else {
        return Err(format!("{} not found in {}", args.id, tasks_dir.display()));
    };
    let task = match parse_task_file(&path) {
        ParsedTask::Valid(t) => t,
        ParsedTask::Invalid(inv) => {
            return Err(format!(
                "{} is invalid ({}) — repair before closing",
                args.id, inv.error
            ))
        }
    };
    if !matches!(task.status, Status::Open | Status::Doing | Status::Blocked) {
        return Err(format!(
            "cannot close {}: status is {}",
            args.id,
            task.status.as_str()
        ));
    }

    let today = crate::clock::today();
    let from = task.status.as_str();
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    if let Some(reason) = &args.waive {
        let out = set_scalar(&text, "status", Some("done"))?;
        let out = set_scalar(&out, "waived", Some(&yaml_scalar(reason)))?;
        let out = append_section_entry(
            &out,
            "log",
            &format!("{today} {from}→done — waived: {reason}"),
        );
        std::fs::write(&path, out).map_err(|e| e.to_string())?;
        crate::store::relocate_for_status(&path, true).map_err(|e| e.to_string())?;
        if json {
            crate::cli::emit_json(
                "close",
                &serde_json::json!({ "id": args.id, "closed": true, "waived": reason }),
            );
        } else {
            println!("{} {from}→done (waived: {reason})", args.id);
        }
        return Ok(());
    }

    let Some(verify) = &task.verify else {
        return Err(format!(
            "{} has no verify: — a task without a machine check closes only \
             with --waive \"<reason>\" (MW-E2)",
            args.id
        ));
    };

    // From the repo root, always — verify commands are written repo-relative.
    let output = std::process::Command::new("sh")
        .args(["-c", verify])
        .current_dir(&root)
        .output()
        .map_err(|e| format!("running verify: {e}"))?;
    let exit = output.status.code().unwrap_or(-1);

    if !json {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }

    if exit == 0 {
        let out = set_scalar(&text, "status", Some("done"))?;
        let out =
            append_section_entry(&out, "log", &format!("{today} {from}→done — verify exit 0"));
        std::fs::write(&path, out).map_err(|e| e.to_string())?;
        crate::store::relocate_for_status(&path, true).map_err(|e| e.to_string())?;
        if json {
            crate::cli::emit_json(
                "close",
                &serde_json::json!({ "id": args.id, "closed": true, "verify_exit": 0 }),
            );
        } else {
            println!("{} {from}→done (verify exit 0)", args.id);
        }
        Ok(())
    } else {
        // Record the attempt (exit + date, MW-E2), close nothing.
        let out = append_section_entry(
            &text,
            "log",
            &format!("{today} close attempt — verify exit {exit}"),
        );
        std::fs::write(&path, out).map_err(|e| e.to_string())?;
        Err(format!(
            "{} stays {from}: verify exit {exit} (`{verify}`)",
            args.id
        ))
    }
}
