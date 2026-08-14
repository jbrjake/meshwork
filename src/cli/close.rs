//! `meshwork close` (PLAN 0.7; MW-E2): run `verify:` from the repo root,
//! record the outcome + date in the log, close on a pass only. `--waive`
//! skips the check but is recorded — loud and queryable. Routing is
//! DESIGN §12b (mw-4aqmf0t): DSL native predicates execute ungated, DSL
//! `run` executes free only on store-only provenance (mw-egksvhm),
//! malformed DSL refuses before anything runs, and legacy shell — still
//! `sh -c` — sits behind the MW-E5 trust gate (mw-9rc4vs6).

use crate::edit::{append_section_entry, remove_scalar, set_scalar};
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
    /// Show the verify text and record this clone's approval of it before
    /// running (MW-E5 trust gate; approval is per-clone, DESIGN §12b).
    #[arg(long)]
    approve: bool,
}

/// MW-E5 (DESIGN §12b): never hand untrusted task content to a shell.
/// Trusted = the `MESHWORK_TRUST=1` reviewed-checkout grant or a recorded
/// per-clone approval of exactly this (id, text); `--approve` records one
/// with the text on screen. Refusal is loud and names the approval step.
fn require_trusted(
    root: &std::path::Path,
    id: &str,
    verify: &str,
    approve: bool,
) -> Result<(), String> {
    if crate::trust::env_trusted() || crate::trust::is_approved(root, id, verify) {
        return Ok(());
    }
    if approve {
        println!("approving verify for {id} (this clone only, MW-E5):\n  verify: {verify}");
        crate::trust::record_approval(root, id, verify)
            .map_err(|e| format!("recording approval: {e}"))
    } else {
        Err(format!(
            "refusing unapproved verify for {id} (MW-E5, DESIGN §12b)\n  \
             verify: {verify}\n  \
             task files arrive via merge and are untrusted; review the \
             command, then:\n  \
             meshwork close {id} --approve   (records approval for this clone)\n  \
             reviewed checkouts (CI, gates) may grant MESHWORK_TRUST=1 instead"
        ))
    }
}

/// mw-ntn0t32: anchor the close to the repo moment — ` @ <short-sha>[+N]`
/// appended to the →done note (N = uncommitted paths). The closing commit
/// lands after close runs, so the sha names its parent; `show` recovers
/// the closing-commit set read-side via the id-in-subject convention.
/// Unborn HEAD degrades to omission — the anchor is a nicety (mw-3jwwh5d
/// precedent), never a failure.
fn head_anchor(root: &std::path::Path) -> String {
    let git = |args: &[&str]| -> Option<String> {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    };
    let Some(sha) = git(&["rev-parse", "--short", "HEAD"]) else {
        return String::new();
    };
    let dirty = git(&["status", "--porcelain"]).map_or(0, |s| s.lines().count());
    if dirty > 0 {
        format!(" @ {sha}+{dirty}")
    } else {
        format!(" @ {sha}")
    }
}

/// The `--waive` path: no verify runs (so no trust gate), the reason is
/// recorded loud and queryable (MW-E2), the anchor still lands.
#[allow(clippy::too_many_arguments)]
fn close_waived(
    path: &std::path::Path,
    root: &std::path::Path,
    id: &str,
    reason: &str,
    text: &str,
    today: &str,
    from: &str,
    json: bool,
) -> Result<(), String> {
    let out = set_scalar(text, "status", Some("done"))?;
    let out = set_scalar(&out, "waived", Some(&yaml_scalar(reason)))?;
    let out = append_section_entry(
        &out,
        "log",
        &format!(
            "{today} {from}→done — waived: {reason}{}",
            head_anchor(root)
        ),
    );
    std::fs::write(path, out).map_err(|e| e.to_string())?;
    crate::store::relocate_for_status(path, true).map_err(|e| e.to_string())?;
    if json {
        crate::cli::emit_json(
            "close",
            &serde_json::json!({ "id": id, "closed": true, "waived": reason }),
        );
    } else {
        println!("{id} {from}→done (waived: {reason})");
    }
    Ok(())
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

    let today = crate::clock::stamp();
    let from = task.status.as_str();
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;

    // Actually closing releases the claim; a failed verify keeps it —
    // the task is still someone's claimed work (mw-tb6gdr9).
    let release_claim = |t: &str| -> Result<String, String> {
        if task.claimed_by.is_some() {
            remove_scalar(t, "claimed-by")
        } else {
            Ok(t.to_string())
        }
    };

    if let Some(reason) = &args.waive {
        let out = release_claim(&text)?;
        return close_waived(&path, &root, &args.id, reason, &out, &today, from, json);
    }

    let Some(verify) = &task.verify else {
        return Err(format!(
            "{} has no verify: — a task without a machine check closes only \
             with --waive \"<reason>\" (MW-E2)",
            args.id
        ));
    };

    // DESIGN §12b gate routing (mw-4aqmf0t): the shape decides the gate.
    // Native predicates load no code and run free; `run` runs free only
    // on store-only provenance; malformed refuses before any gate; legacy
    // shell keeps the full MW-E5 gate.
    let verdict = match crate::verify_dsl::classify(verify) {
        crate::verify_dsl::Classified::Malformed(why) => {
            // Never runs, never gate-prompts: approving garbage is not a
            // reviewable act, and a downgrade to shell reopens the hole.
            return Err(format!(
                "refusing malformed verify for {id}: {why}\n  verify: {verify}\n  \
                 keyword-led text never runs as shell (DESIGN §12b) — fix it: \
                 meshwork set {id} --verify '<predicate>'",
                id = args.id
            ));
        }
        crate::verify_dsl::Classified::Dsl(preds) => {
            if preds
                .iter()
                .any(|p| matches!(p, crate::verify_dsl::Predicate::Run { .. }))
            {
                gate_run(&root, &path, &args.id, verify, args.approve)?;
            }
            crate::verify_exec::execute(&root, &preds).map_err(|why| {
                (
                    "verify failed (dsl)".to_string(),
                    format!("verify failed — {why}"),
                )
            })
        }
        crate::verify_dsl::Classified::LegacyShell => {
            require_trusted(&root, &args.id, verify, args.approve)?;
            run_shell(&root, verify, json)?
        }
    };

    match verdict {
        Ok(()) => {
            let out = release_claim(&text)?;
            let out = set_scalar(&out, "status", Some("done"))?;
            let out = append_section_entry(
                &out,
                "log",
                &format!("{today} {from}→done — verify exit 0{}", head_anchor(&root)),
            );
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
        }
        Err((note, stays)) => {
            // Record the attempt (MW-E2), close nothing.
            let out =
                append_section_entry(&text, "log", &format!("{today} close attempt — {note}"));
            std::fs::write(&path, out).map_err(|e| e.to_string())?;
            Err(format!("{} stays {from}: {stays}", args.id))
        }
    }
}

/// A verify outcome: `Ok` closes; `Err` is (log note, stays-open reason).
type Verdict = Result<(), (String, String)>;

/// Legacy shell execution, `sh -c` from the repo root — verify commands
/// are written repo-relative. The outer error is "could not run at all";
/// the inner `Verdict` is what the run said.
fn run_shell(root: &std::path::Path, verify: &str, json: bool) -> Result<Verdict, String> {
    let output = std::process::Command::new("sh")
        .args(["-c", verify])
        .current_dir(root)
        .output()
        .map_err(|e| format!("running verify: {e}"))?;
    let exit = output.status.code().unwrap_or(-1);
    if !json {
        print!("{}", String::from_utf8_lossy(&output.stdout));
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if exit == 0 {
        Ok(Ok(()))
    } else {
        Ok(Err((
            format!("verify exit {exit}"),
            format!("verify exit {exit} (`{verify}`)"),
        )))
    }
}

/// Gate one DSL `run` (DESIGN §12b, mw-4aqmf0t): env trust and recorded
/// approvals short-circuit; otherwise store-only provenance runs free —
/// the frictionless test-backed close is the point of the DSL — while a
/// ride-along or unanswerable history gates exactly like legacy shell,
/// the refusal naming the arrival to review.
fn gate_run(
    root: &std::path::Path,
    task_path: &std::path::Path,
    id: &str,
    verify: &str,
    approve: bool,
) -> Result<(), String> {
    if crate::trust::env_trusted() || crate::trust::is_approved(root, id, verify) {
        return Ok(());
    }
    let rel = task_path
        .strip_prefix(root)
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|_| format!("task file {} escapes the repo root", task_path.display()))?;
    let why = match crate::provenance::task_provenance(root, &rel) {
        crate::provenance::Provenance::Trusted => return Ok(()),
        crate::provenance::Provenance::RodeAlong { arrival, path } => format!(
            "the task arrived with non-store content: {arrival} carried `{path}` \
             (ride-along, DESIGN §12b) — a task must never self-verify against \
             code that arrived with it"
        ),
        crate::provenance::Provenance::Unknown { why } => {
            format!("git cannot vouch for the task's history ({why}) — gating, never trusting")
        }
    };
    if approve {
        println!("approving verify for {id} (this clone only, MW-E5):\n  verify: {verify}");
        crate::trust::record_approval(root, id, verify)
            .map_err(|e| format!("recording approval: {e}"))
    } else {
        Err(format!(
            "refusing run verify for {id} (MW-E5, DESIGN §12b)\n  verify: {verify}\n  {why}\n  \
             review the named arrival, then: meshwork close {id} --approve\n  \
             reviewed checkouts (CI, gates) may grant MESHWORK_TRUST=1 instead"
        ))
    }
}
