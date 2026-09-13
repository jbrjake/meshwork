//! `start` / `block --reason` / `drop [--reason]` / `reopen` (PLAN 0.6;
//! MW-E1/E3):
//! one-frontmatter-line status edits plus a dated log append — never a
//! full-file rewrite (MW-I1). `start` also records an advisory `claimed-by:`
//! when the MW-K1 chain yields an identity; close/drop/reopen release it
//! (mw-tb6gdr9 — a claim coordinates, it never locks).

use crate::edit::{append_section_entry, remove_scalar, set_scalar};
use crate::parse::{ParsedTask, Status};
use crate::write::yaml_scalar;

#[derive(clap::Args)]
pub(crate) struct IdArg {
    /// Task id (e.g. az-k7f3).
    pub(crate) id: String,
}

#[derive(clap::Args)]
pub(crate) struct StartArgs {
    /// Task id (e.g. az-k7f3).
    id: String,
    /// Claim identity — self-professed, advisory (falls back to
    /// `$MESHWORK_AUTHOR`, then config `default_author`). No identity
    /// resolving = no claim; the start still happens.
    #[arg(long = "as", value_name = "AUTHOR")]
    author: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct BlockArgs {
    /// Task id (e.g. az-k7f3).
    id: String,
    /// Blocker + unblock condition — required; a bare "blocked" helps no
    /// one at session start.
    #[arg(long, required = true, value_name = "TEXT")]
    reason: String,
}

#[derive(clap::Args)]
pub(crate) struct DropArgs {
    /// Task id (e.g. az-k7f3).
    id: String,
    /// Why the work is not happening — recorded on the log entry.
    /// Optional: some drops are self-evident.
    #[arg(long, value_name = "TEXT")]
    reason: Option<String>,
}

pub(crate) fn start(args: &StartArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    // Capture-before-verifiable gate (mw-6wdpz1b, owner-ruled): filing
    // without a verify: is legal; STARTING is not — writing the done-test
    // is the first unit of the work itself. Waive stays a close-time
    // concept for the genuinely unverifiable; this is the not-yet-specified.
    let tasks_dir = root.join("docs").join("meshwork");
    if let Some(located) = crate::archive::locate(&tasks_dir, &args.id) {
        let path = located.path().to_path_buf();
        if let ParsedTask::Valid(t) = located.parse() {
            match t.verify.as_deref().map(str::trim) {
                None | Some("") => {
                    return Err(format!(
                        "cannot start {id}: needs-verify — write the done-test first: \
                         `meshwork set {id} --verify '<cmd>'`, then start",
                        id = args.id
                    ));
                }
                Some(verify) => red_check(&root, &path, &args.id, verify)?,
            }
        }
    }
    let claimant = super::notes::resolve_author(&root, args.author.as_deref())?;
    // The claimant lands in `claimed-by:` frontmatter — same door
    // (mw-3tzfqmq), whichever link of the identity chain supplied it.
    if let Some(claimant) = claimant.as_deref() {
        crate::cli::reject_controls("author", claimant, false)?;
    }
    transition(
        "start",
        &args.id,
        &[Status::Open],
        Status::Doing,
        None,
        claimant.as_deref(),
        json,
    )
}

/// mw-175bn4c: a verify already green at start cannot detect the work —
/// the sister of the needs-verify gate above (absent vs present-but-
/// vacuous). Advisory by the mw-kkvs8zq precedent (a warning is behavior,
/// no new surface). Routing mirrors close (DESIGN §12b, mw-4aqmf0t):
/// native DSL runs untrusted (pure reads), DSL `run` needs trust or
/// store-only provenance, legacy shell needs the MW-E5 gate — every
/// skip is loud instead of silent.
fn red_check(
    root: &std::path::Path,
    task_path: &std::path::Path,
    id: &str,
    verify: &str,
) -> Result<(), String> {
    use crate::verify_dsl::{classify, Classified, Predicate};
    let trusted = || crate::trust::env_trusted() || crate::trust::is_approved(root, id, verify);
    match classify(verify) {
        // A verify close would refuse is refused here too (mw-8e769q0):
        // starting work behind a gate that can never open is the
        // hand-flip's first step.
        Classified::Malformed(why) => {
            return Err(format!(
                "cannot start {id}: {} — fix it: meshwork set {id} --verify '<predicate>'",
                crate::verify_dsl::malformed_refusal(verify, &why)
            ));
        }
        Classified::Dsl(preds) => {
            let has_run = preds.iter().any(|p| matches!(p, Predicate::Run { .. }));
            if has_run && !trusted() && !store_only(root, task_path) {
                eprintln!(
                    "note: red-check skipped for {id} — run verify gated for \
                     this clone (approve at close, or MESHWORK_TRUST=1)"
                );
                return Ok(());
            }
            if has_run {
                announce(id);
            }
            match crate::verify_exec::execute(root, &preds) {
                Ok(()) => eprintln!(
                    "warning: red-check: {id}'s verify is already green — it \
                     cannot detect the work; tighten it, or close if the work \
                     is done"
                ),
                Err(e) if e.contains("timeout after") => timed_out(id),
                Err(_) => {}
            }
        }
        Classified::LegacyShell => {
            if !trusted() {
                eprintln!(
                    "note: red-check skipped for {id} — verify unapproved for this \
                     clone (approve at close, or MESHWORK_TRUST=1)"
                );
                return Ok(());
            }
            // The same wall clock and output cap as a DSL run
            // (mw-82thxwz): an unscoped `cargo test` compiled for minutes
            // in silence, and the agent hand-flipped the status.
            announce(id);
            let argv = ["sh", "-c", verify].map(String::from);
            let outcome = crate::verify_exec::spawn_capped(
                root,
                &argv,
                crate::verify_exec::run_timeout(),
                crate::verify_exec::OUTPUT_CAP,
            );
            match outcome {
                Ok((status, _)) => match status.code().unwrap_or(-1) {
                    0 => eprintln!(
                        "warning: red-check: {id}'s verify is already green (exit 0) — \
                         it cannot detect the work; tighten it, or close if the work is \
                         done"
                    ),
                    127 => eprintln!(
                        "warning: red-check: {id}'s verify exits 127 under sh -c — \
                         close's shell won't have agent-shell functions; recast in \
                         grep/test/cargo"
                    ),
                    _ => {}
                },
                Err(crate::verify_exec::SpawnError::Timeout(_)) => timed_out(id),
                Err(e) => eprintln!("warning: red-check: could not run {id}'s verify: {e}"),
            }
        }
    }
    Ok(())
}

/// Said before a verify that may build runs, so a slow compile reads as
/// work in progress, never a hang.
fn announce(id: &str) {
    eprintln!(
        "note: red-checking {id}'s verify — may build; up to {}s",
        crate::verify_exec::run_timeout().as_secs()
    );
}

/// A red-check that hit the wall clock: a warning, and the start still
/// transitions — close runs the verify in full.
fn timed_out(id: &str) {
    eprintln!(
        "warning: red-check: {id}'s verify did not finish within {}s — close \
         will run it in full; scope it (a test filter, a package) so it runs \
         fast",
        crate::verify_exec::run_timeout().as_secs()
    );
}

/// Advisory-tier provenance: true iff the task's history is store-only
/// (mw-egksvhm `Trusted`). `RodeAlong` and `Unknown` both read as "not free"
/// here — close's gate owns the full refusal wording.
fn store_only(root: &std::path::Path, task_path: &std::path::Path) -> bool {
    task_path.strip_prefix(root).is_ok_and(|rel| {
        matches!(
            crate::provenance::task_provenance(root, &rel.to_string_lossy()),
            crate::provenance::Provenance::Trusted
        )
    })
}

pub(crate) fn block(args: &BlockArgs, json: bool) -> Result<(), String> {
    crate::cli::reject_controls("reason", &args.reason, false)?;
    transition(
        "block",
        &args.id,
        &[Status::Open, Status::Doing],
        Status::Blocked,
        Some(&args.reason),
        None,
        json,
    )
}

pub(crate) fn drop(args: &DropArgs, json: bool) -> Result<(), String> {
    if let Some(reason) = &args.reason {
        crate::cli::reject_controls("reason", reason, false)?;
    }
    // Scan BEFORE the write (mw-kkvs8zq): a found-but-broken registry is
    // the mw-k7r5 loud error, and it must fire with nothing yet changed.
    // No registry anywhere = no cross-repo namespace = no scan (quiet).
    let scan = match crate::registry::quiet_load()? {
        Some(registry) => {
            let root = crate::cli::require_store_root()?;
            crate::registry_hygiene::inbound_needs(&registry, &root, &args.id)
        }
        None => None,
    };
    transition(
        "drop",
        &args.id,
        &[Status::Open, Status::Doing, Status::Blocked],
        Status::Dropped,
        args.reason.as_deref(),
        None,
        json,
    )?;
    // Advisory, stderr, both modes: only done/dropped satisfies a
    // dependency, and these needs were just cleared by a drop — the
    // needed work never happened. The drop itself always proceeds;
    // refusal is a §6 question this verb does not take.
    if let Some(scan) = scan {
        for h in &scan.hits {
            eprintln!(
                "warning: {} needs {} — cleared by a drop, not a done; \
                 the needed work never happened",
                h.src_gid, h.target
            );
        }
        for (repo, why) in &scan.unscanned {
            eprintln!(
                "warning: {repo} unscanned ({why}) — inbound needs on {}#{} \
                 may hide there",
                scan.self_name, args.id
            );
        }
    }
    Ok(())
}

pub(crate) fn reopen(args: &IdArg, json: bool) -> Result<(), String> {
    // The missing inverse: without it every unblock is a hand-edit (§6).
    transition(
        "reopen",
        &args.id,
        &[Status::Blocked, Status::Doing, Status::Done],
        Status::Open,
        None,
        None,
        json,
    )
}

fn transition(
    verb: &str,
    id: &str,
    allowed_from: &[Status],
    to: Status,
    reason: Option<&str>,
    claim: Option<&str>,
    json: bool,
) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("docs").join("meshwork");
    let Some(located) = crate::archive::locate(&tasks_dir, id) else {
        return Err(format!("{id} not found in {}", tasks_dir.display()));
    };
    let task = match located.parse() {
        ParsedTask::Valid(t) => t,
        ParsedTask::Invalid(inv) => {
            return Err(format!(
                "{id} is invalid ({}) — repair it (lint --fix) before transitioning",
                inv.error
            ))
        }
    };
    if !allowed_from.contains(&task.status) {
        return Err(format!(
            "cannot {verb} {id}: status is {}, needs one of [{}]",
            task.status.as_str(),
            allowed_from
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // A bundled document comes back out as a file of its own first
    // (mw-bvxpeef) — reopen is the one transition a terminal task takes,
    // and the relocation below then carries the file to the root.
    let path = if located.bundled() {
        crate::archive::extract(&tasks_dir, id)?
    } else {
        located.path().to_path_buf()
    };
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut text = set_scalar(&text, "status", Some(to.as_str()))?;
    match (to, reason) {
        (Status::Blocked, Some(reason)) => {
            text = set_scalar(&text, "blocked-reason", Some(&yaml_scalar(reason)))?;
        }
        // Leaving blocked clears the reason but keeps the key — matching
        // the normative example's empty `blocked-reason:` line (DESIGN §2).
        (_, _) if task.status == Status::Blocked && task.blocked_reason.is_some() => {
            text = set_scalar(&text, "blocked-reason", None)?;
        }
        _ => {}
    }
    if let Some(claimant) = claim {
        text = set_scalar(&text, "claimed-by", Some(&yaml_scalar(claimant)))?;
    } else if task.claimed_by.is_some() && !matches!(to, Status::Doing | Status::Blocked) {
        // Leaving the claimed states releases the claim (mw-tb6gdr9).
        text = remove_scalar(&text, "claimed-by")?;
    }
    // Terminal states silence the handoff (mw-e8hg2kt): the voice belongs
    // to whatever is up next, and left behind it becomes a handoff-stale
    // warning on an archived file — unfixable except by hand-edit.
    let terminal = matches!(to, Status::Done | Status::Dropped);
    if terminal && task.handoff.is_some() {
        text = remove_scalar(&text, "handoff")?;
    }

    let today = crate::clock::stamp();
    let mut entry = format!("{today} {}→{}", task.status.as_str(), to.as_str());
    if let Some(reason) = reason {
        use std::fmt::Write as _;
        let _ = write!(entry, " — {reason}");
    }
    if let Some(claimant) = claim {
        use std::fmt::Write as _;
        let _ = write!(entry, " — claimed by {claimant}");
    }
    let text = append_section_entry(&text, "log", &entry);
    std::fs::write(&path, text).map_err(|e| e.to_string())?;
    // Terminal tasks live in archive/; reopen brings the file back
    // (mw-45e2qf4 — the graph never notices, only the directory does).
    crate::store::relocate_for_status(&path, terminal).map_err(|e| e.to_string())?;

    if json {
        crate::cli::emit_json(
            verb,
            &serde_json::json!({
                "id": id, "from": task.status.as_str(), "to": to.as_str(),
                "reason": reason,
            }),
        );
    } else {
        println!("{id} {}→{}", task.status.as_str(), to.as_str());
    }
    Ok(())
}
