//! `spec list <doc>` and `spec audit <doc>` (mw-y1qwnz3, MW-T5): coverage
//! and drift over one spec document, as a report — clause by clause for
//! `list`, and for `audit` the five questions in one answer: clauses no
//! task pins (unclaimed), clauses only dropped tasks pin (orphaned), live
//! tasks whose pin drifted (stale), done tasks whose pin drifted (re-open
//! candidates — surfaced, never reopened), and pins naming a clause the
//! document no longer carries (dangling). The single-repo verb sees this
//! store's pins; `portfolio spec` unions every store's, which is the only
//! place a cross-repo pin shows.

use crate::spec::{Audit, Pin};

#[derive(clap::Args)]
pub(crate) struct SpecArgs {
    #[command(subcommand)]
    pub(crate) action: SpecAction,
}

#[derive(clap::Subcommand)]
pub(crate) enum SpecAction {
    /// Every anchored clause of a document with its hash and who covers it.
    List {
        /// The document: a repo-relative path, or <repo>#<path>.
        doc: String,
    },
    /// Unclaimed, orphaned, stale, re-open candidates, dangling — one report.
    Audit {
        /// The document: a repo-relative path, or <repo>#<path>.
        doc: String,
    },
}

impl SpecAction {
    pub(crate) fn doc(&self) -> &str {
        match self {
            SpecAction::List { doc } | SpecAction::Audit { doc } => doc,
        }
    }
}

pub(crate) fn run(args: &SpecArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let store = crate::store::load_repo(&root).map_err(|e| e.to_string())?;
    let audit = crate::spec::audit(
        &root,
        &store.repo,
        std::slice::from_ref(&store),
        args.action.doc(),
    )?;
    emit(&args.action, &audit, json, "spec", None);
    Ok(())
}

/// Print `list` or `audit` over an audit already taken; `extra` rides
/// the JSON data (the union's skips).
pub(crate) fn emit(
    action: &SpecAction,
    audit: &Audit,
    json: bool,
    verb: &str,
    extra: Option<(&str, serde_json::Value)>,
) {
    let (name, mut data) = match action {
        SpecAction::List { .. } => ("list", list_json(audit)),
        SpecAction::Audit { .. } => ("audit", audit_json(audit)),
    };
    if json {
        if let Some((key, value)) = extra {
            data[key] = value;
        }
        crate::cli::emit_json(&format!("{verb} {name}"), &data);
        return;
    }
    let text = match action {
        SpecAction::List { .. } => list_text(audit),
        SpecAction::Audit { .. } => audit_text(audit),
    };
    print!("{}", crate::cli::sanitize(&text));
}

fn short(sha: &str) -> &str {
    &sha[..sha.len().min(12)]
}

fn pin_json(p: &Pin) -> serde_json::Value {
    serde_json::json!({ "gid": p.gid, "status": p.status.as_str(), "ref": p.reference })
}

fn list_json(audit: &Audit) -> serde_json::Value {
    let clauses: Vec<_> = audit
        .clauses
        .iter()
        .map(|c| {
            serde_json::json!({ "id": c.id, "heading": c.heading, "sha": c.sha,
                "covered_by": audit.covering(&c.id).into_iter().map(pin_json).collect::<Vec<_>>() })
        })
        .collect();
    serde_json::json!({ "doc": audit.doc, "clauses": clauses })
}

fn list_text(audit: &Audit) -> String {
    use std::fmt::Write as _;
    let mut out = format!("{}: {} clauses\n", audit.doc, audit.clauses.len());
    for c in &audit.clauses {
        let pins = audit.covering(&c.id);
        let by = if pins.is_empty() {
            "unclaimed".to_string()
        } else {
            format!(
                "covered by {}: {}",
                pins.len(),
                pins.iter()
                    .map(|p| format!("{} ({})", p.gid, p.status.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let _ = writeln!(out, "  {}  {}  @{}  {by}", c.id, c.heading, short(&c.sha));
    }
    out
}

fn drift_json(rows: &[(&Pin, &crate::spec::Clause)]) -> Vec<serde_json::Value> {
    rows.iter()
        .map(|(p, c)| {
            serde_json::json!({ "gid": p.gid, "status": p.status.as_str(), "ref": p.reference,
                "pinned": p.sha, "now": c.sha })
        })
        .collect()
}

fn audit_json(audit: &Audit) -> serde_json::Value {
    serde_json::json!({
        "doc": audit.doc,
        "clauses": audit.clauses.len(),
        "pins": audit.pins.len(),
        "unclaimed": audit.unclaimed().iter().map(|c| c.id.clone()).collect::<Vec<_>>(),
        "orphaned": audit.orphaned().iter().map(|(c, pins)| serde_json::json!({
            "id": c.id, "by": pins.iter().map(|p| pin_json(p)).collect::<Vec<_>>() })).collect::<Vec<_>>(),
        "stale": drift_json(&audit.drifted(true)),
        "reopen_candidates": drift_json(&audit.drifted(false)),
        "dangling": audit.dangling().into_iter().map(pin_json).collect::<Vec<_>>(),
    })
}

fn audit_text(audit: &Audit) -> String {
    use std::fmt::Write as _;
    let tasks: std::collections::BTreeSet<&str> =
        audit.pins.iter().map(|p| p.gid.as_str()).collect();
    let mut out = format!(
        "{}: {} clauses, {} pins across {} tasks\n",
        audit.doc,
        audit.clauses.len(),
        audit.pins.len(),
        tasks.len()
    );
    let unclaimed = audit.unclaimed();
    let _ = writeln!(
        out,
        "unclaimed ({}): {}",
        unclaimed.len(),
        list_or_none(unclaimed.iter().map(|c| c.id.clone()))
    );
    let orphaned = audit.orphaned();
    let _ = writeln!(
        out,
        "orphaned ({}): {}",
        orphaned.len(),
        list_or_none(orphaned.iter().map(|(c, pins)| format!(
            "{} \u{2190} {}",
            c.id,
            pins.iter().map(|p| p.gid.as_str()).collect::<Vec<_>>().join(", ")
        )))
    );
    for (label, live) in [("stale", true), ("re-open candidates", false)] {
        let rows = audit.drifted(live);
        let _ = writeln!(
            out,
            "{label} ({}): {}",
            rows.len(),
            list_or_none(rows.iter().map(|(p, c)| format!(
                "{} {} pinned {} now {}",
                p.gid,
                p.reference,
                p.sha.as_deref().map_or("nothing", short),
                short(&c.sha)
            )))
        );
    }
    let dangling = audit.dangling();
    let _ = writeln!(
        out,
        "dangling ({}): {}",
        dangling.len(),
        list_or_none(
            dangling
                .iter()
                .map(|p| format!("{} {}", p.gid, p.reference))
        )
    );
    out
}

/// `a · b · c`, or `none`.
fn list_or_none(items: impl Iterator<Item = String>) -> String {
    let items: Vec<String> = items.collect();
    if items.is_empty() {
        "none".to_string()
    } else {
        items.join(" \u{b7} ")
    }
}
