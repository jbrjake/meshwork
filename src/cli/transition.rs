//! `start` / `block --reason` / `drop` / `reopen` (PLAN 0.6; MW-E1/E3):
//! one-frontmatter-line status edits plus a dated log append — never a
//! full-file rewrite (MW-I1). `start` also records an advisory `claimed-by:`
//! when the MW-K1 chain yields an identity; close/drop/reopen release it
//! (mw-tb6gdr9 — a claim coordinates, it never locks).

use crate::edit::{append_section_entry, remove_scalar, set_scalar};
use crate::parse::{parse_task_file, ParsedTask, Status};
use crate::store::find_task_file;
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
    /// Claim identity — self-professed, advisory (MW-K1 chain: this flag,
    /// then `$MESHWORK_AUTHOR`, then config `default_author`). No identity
    /// resolving = no claim; the start still happens.
    #[arg(long = "as", value_name = "AUTHOR")]
    author: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct BlockArgs {
    /// Task id (e.g. az-k7f3).
    id: String,
    /// Blocker + unblock condition — required; a bare "blocked" helps no
    /// one at session start (MW-E1).
    #[arg(long, required = true, value_name = "TEXT")]
    reason: String,
}

pub(crate) fn start(args: &StartArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    // Capture-before-verifiable gate (mw-6wdpz1b, owner-ruled): filing
    // without a verify: is legal; STARTING is not — writing the done-test
    // is the first unit of the work itself. Waive stays a close-time
    // concept for the genuinely unverifiable; this is the not-yet-specified.
    let tasks_dir = root.join("docs").join("meshwork");
    if let Some(path) = find_task_file(&tasks_dir, &args.id) {
        if let ParsedTask::Valid(t) = parse_task_file(&path) {
            match t.verify.as_deref().map(str::trim) {
                None | Some("") => {
                    return Err(format!(
                        "cannot start {id}: needs-verify — write the done-test first: \
                         `meshwork set {id} --verify '<cmd>'`, then start (mw-6wdpz1b)",
                        id = args.id
                    ));
                }
                Some(verify) => red_check(&root, &args.id, verify),
            }
        }
    }
    let claimant = super::notes::resolve_author(&root, args.author.as_deref())?;
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
/// no new surface); execution stays behind the MW-E5 trust gate, so
/// untrusted text never runs — that skip is loud instead of silent.
fn red_check(root: &std::path::Path, id: &str, verify: &str) {
    if !(crate::trust::env_trusted() || crate::trust::is_approved(root, id, verify)) {
        eprintln!(
            "note: red-check skipped for {id} — verify unapproved for this \
             clone (MW-E5; approve at close, or MESHWORK_TRUST=1)"
        );
        return;
    }
    let exit = std::process::Command::new("sh")
        .args(["-c", verify])
        .current_dir(root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_or(-1, |s| s.code().unwrap_or(-1));
    match exit {
        0 => eprintln!(
            "warning: red-check: {id}'s verify is already green (exit 0) — \
             it cannot detect the work; tighten it, or close if the work is \
             done (mw-175bn4c)"
        ),
        127 => eprintln!(
            "warning: red-check: {id}'s verify exits 127 under sh -c — \
             close's shell won't have agent-shell functions; recast in \
             grep/test/cargo (mw-175bn4c)"
        ),
        _ => {}
    }
}

pub(crate) fn block(args: &BlockArgs, json: bool) -> Result<(), String> {
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

pub(crate) fn drop(args: &IdArg, json: bool) -> Result<(), String> {
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
        None,
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
    let Some(path) = find_task_file(&tasks_dir, id) else {
        return Err(format!("{id} not found in {}", tasks_dir.display()));
    };
    let task = match parse_task_file(&path) {
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
    let terminal = matches!(to, Status::Done | Status::Dropped);
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
