//! `meshwork add` (PLAN 0.5): create a task file, print its id. Missing
//! `--verify` is a lint warning until set (MW-E2); `--from` records
//! provenance (MW-E4).

use crate::id::{mint_unique, slugify, IdGen};
use crate::write::yaml_scalar;
use std::fmt::Write as _;

#[derive(clap::Args)]
pub(crate) struct AddArgs {
    /// One-line title.
    #[arg(required_unless_present = "batch")]
    title: Option<String>,
    /// Several tasks at once from a file ("-" = stdin): concatenated task
    /// documents, `id:` omitted, local `handle:` names usable as @refs in
    /// needs/parent/from/relates — atomic, all files or none.
    #[arg(long, value_name = "FILE", conflicts_with_all = ["title", "cat", "label", "needs", "parent", "from", "verify", "seq", "docs", "body"])]
    batch: Option<String>,
    /// Print the would-be task file(s), write nothing.
    #[arg(long)]
    dry_run: bool,
    /// Category slash-path, e.g. engine/spill.
    #[arg(long = "cat", alias = "category", value_name = "PATH")]
    cat: Option<String>,
    /// Cross-cutting label; repeatable.
    #[arg(long = "label", value_name = "LABEL")]
    label: Vec<String>,
    /// Hard dependency id (`repo#id` crosses repos); repeatable.
    #[arg(long = "needs", value_name = "ID")]
    needs: Vec<String>,
    /// Same-repo parent id; a parent never crosses repos.
    #[arg(long, value_name = "ID")]
    parent: Option<String>,
    /// Provenance: the task this one was discovered from.
    #[arg(long = "from", value_name = "ID")]
    from: Option<String>,
    /// The close gate: a verify predicate — `exists`/`absent`/`contains`/
    /// `run cargo …`, or `all(…)`; `verify --help` has the grammar. Text
    /// that is not keyword-led is legacy shell behind the approval gate.
    #[arg(long, value_name = "PREDICATE")]
    verify: Option<String>,
    /// Per-repo order weight, lower sooner; gaps of 10.
    #[arg(long, value_name = "N")]
    seq: Option<i64>,
    /// Doc link `path#§-anchor`; repeatable.
    #[arg(long = "docs", alias = "doc", value_name = "LINK")]
    docs: Vec<String>,
    /// Body prose at creation. `@<file>` reads the file, `-` reads
    /// stdin — the payload never transits shell quoting.
    #[arg(long, value_name = "TEXT|@FILE|-")]
    body: Option<String>,
}

pub(crate) fn run(args: &AddArgs, json: bool) -> Result<(), String> {
    if let Some(source) = &args.batch {
        return super::add_batch::run(source, args.dry_run, json);
    }
    let root = crate::cli::require_store_root()?;
    let config = crate::store::load_config(&root).map_err(|e| e.to_string())?;
    if args.parent.as_deref().is_some_and(|p| p.contains('#')) {
        return Err("parent must stay in-repo — hierarchy never crosses repos; \
                    use sequence.md tranches for portfolio grouping"
            .to_string());
    }

    let tasks_dir = root.join("docs").join("meshwork");
    let seed = std::env::var("MESHWORK_ID_SEED").ok();
    let mut idgen = IdGen::from_seed_str(seed.as_deref());
    let id = mint_unique(&config.alias, &tasks_dir, &mut idgen).map_err(|e| e.to_string())?;

    let today = crate::clock::stamp();
    let title = args
        .title
        .as_deref()
        .unwrap_or_default()
        .replace(['\n', '\r'], " ");
    reject_control_fields(args, &title)?;
    if let Some(verify) = &args.verify {
        refuse_malformed(verify)?;
    }
    let mut targets: Vec<(&str, &str)> = args.needs.iter().map(|n| ("needs", n.as_str())).collect();
    targets.extend(args.parent.iter().map(|p| ("parent", p.as_str())));
    targets.extend(args.from.iter().map(|f| ("discovered-from", f.as_str())));
    check_edge_targets(&tasks_dir, &targets, &[])?;
    warn_docs(&root, &args.docs);
    let fm = render_frontmatter(args, &id, &title, &today);

    // Body above the tail sections (mw-s3905fv, §6 ruling 2026-08-21):
    // the description finally has a CLI path at creation — before this,
    // every substantive body arrived by shell append, the damage source
    // the stray-tail lint repairs. Empty payload = no body block.
    let body = match &args.body {
        Some(raw) => {
            let payload = crate::cli::prose_payload(raw)?;
            crate::cli::reject_controls("body", &payload, true)?;
            if !raw.starts_with('@') && raw != "-" {
                warn_inline_body(&payload);
            }
            if payload.trim().is_empty() {
                String::new()
            } else {
                format!("{payload}\n\n")
            }
        }
        None => String::new(),
    };
    let file = format!("---\n{fm}---\n\n{body}## log\n- {today} created\n");
    let path = tasks_dir.join(format!("{id}-{}.md", slugify(&title)));
    let rel = format!(
        "docs/meshwork/{}",
        path.file_name().unwrap().to_string_lossy()
    );

    // §6: --dry-run prints the would-be file, writes nothing (mw-0wvndqa).
    if args.dry_run {
        if json {
            crate::cli::emit_json(
                "add",
                &serde_json::json!({ "id": id, "path": rel, "dry_run": true, "content": file }),
            );
        } else {
            println!("--- {rel}");
            print!("{file}");
        }
        return Ok(());
    }

    std::fs::create_dir_all(&tasks_dir).map_err(|e| e.to_string())?;
    std::fs::write(&path, file).map_err(|e| e.to_string())?;
    // Approve-at-mint (§12b as amended 2026-08-21, mw-2kgkn0j/mw-51x0wty):
    // authoring text through this clone's CLI IS the operator's approval —
    // record what `close --approve` would. Best-effort: a failed record
    // only restores the prompt (the cache is never a dependency, MW-A2).
    if let Some(verify) = &args.verify {
        if let Err(e) = crate::trust::record_approval(&root, &id, verify) {
            eprintln!("warning: could not record verify approval: {e}");
        }
    }

    if json {
        crate::cli::emit_json("add", &serde_json::json!({ "id": id, "path": rel }));
    } else {
        println!("{id}");
        println!("  {rel}");
        if args.verify.is_none() {
            eprintln!("note: no --verify set — lint will warn until it is");
        }
    }
    Ok(())
}

/// Refuse controls before anything is minted (mw-3tzfqmq) — every
/// frontmatter-bound field takes the same door.
fn reject_control_fields(args: &AddArgs, title: &str) -> Result<(), String> {
    crate::cli::reject_controls("title", title, false)?;
    for (field, value) in [
        ("category", &args.cat),
        ("parent", &args.parent),
        ("discovered-from", &args.from),
        ("verify", &args.verify),
    ] {
        if let Some(value) = value {
            crate::cli::reject_controls(field, value, false)?;
        }
    }
    for (field, values) in [
        ("label", &args.label),
        ("needs", &args.needs),
        ("docs link", &args.docs),
    ] {
        for value in values {
            crate::cli::reject_controls(field, value, false)?;
        }
    }
    Ok(())
}

/// The frontmatter body, key by key in the §2 order.
fn render_frontmatter(args: &AddArgs, id: &str, title: &str, today: &str) -> String {
    let mut fm = String::new();
    let _ = writeln!(fm, "id: {id}");
    let _ = writeln!(fm, "title: {}", yaml_scalar(title));
    fm.push_str("status: open\n");
    if let Some(cat) = &args.cat {
        let _ = writeln!(fm, "category: {}", yaml_scalar(cat));
    }
    if !args.label.is_empty() {
        let _ = writeln!(fm, "labels: [{}]", scalar_list(&args.label));
    }
    if !args.needs.is_empty() {
        let _ = writeln!(fm, "needs: [{}]", scalar_list(&args.needs));
    }
    if let Some(parent) = &args.parent {
        let _ = writeln!(fm, "parent: {}", yaml_scalar(parent));
    }
    if let Some(from) = &args.from {
        let _ = writeln!(fm, "discovered-from: {}", yaml_scalar(from));
    }
    if let Some(verify) = &args.verify {
        let _ = writeln!(fm, "verify: {}", yaml_scalar(verify));
    }
    if !args.docs.is_empty() {
        fm.push_str("docs:\n");
        for link in &args.docs {
            let _ = writeln!(fm, "  - {link}");
        }
    }
    if let Some(seq) = args.seq {
        let _ = writeln!(fm, "seq: {seq}");
    }
    let _ = writeln!(fm, "created: {today}");
    fm
}

/// mw-tkgvsdz: a same-repo edge target that does not exist is refused —
/// `add --from mw-mjwfxn` (a typo) once minted two dangling edges
/// silently. `known` carries ids not yet on disk (a batch's own). A
/// cross-repo target is the registry's business: warned when the
/// registry knows no such repo, or resolves it and finds no such task.
pub(crate) fn check_edge_targets(
    tasks_dir: &std::path::Path,
    targets: &[(&str, &str)],
    known: &[String],
) -> Result<(), String> {
    for (field, target) in targets {
        if let Some((repo, id)) = target.split_once('#') {
            warn_crossrepo_target(field, repo, id);
            continue;
        }
        if known.iter().any(|k| k == target)
            || crate::store::find_task_file(tasks_dir, target).is_some()
        {
            continue;
        }
        return Err(format!(
            "{field} target `{target}` does not exist in this store — check the id \
             (`meshwork search <words>`), or file the target first"
        ));
    }
    Ok(())
}

fn warn_crossrepo_target(field: &str, repo: &str, id: &str) {
    let Ok(Some(registry)) = crate::registry::quiet_load() else {
        return;
    };
    match registry.resolve(repo) {
        None => eprintln!(
            "warning: {field} target `{repo}#{id}` names no registered repo — it can never \
             resolve, and ready will block on it"
        ),
        Some((entry, _)) => {
            let Some(root) = &entry.path else { return };
            let sibling = root.join("docs").join("meshwork");
            if sibling.exists() && crate::store::find_task_file(&sibling, id).is_none() {
                eprintln!(
                    "warning: {field} target `{repo}#{id}` does not exist in {repo}'s store — \
                     ready will block on it until it does"
                );
            }
        }
    }
}

/// A docs link that dead-ends is said at add, not at the next lint.
pub(crate) fn warn_docs(root: &std::path::Path, docs: &[String]) {
    for link in docs {
        if let Some(err) = crate::docs::resolve(root, link).error {
            eprintln!("warning: docs: {link} — {err} (show --docs will dead-end)");
        }
    }
}

/// Inline prose that reached us carrying shell syntax may already have
/// been expanded on the way in — the pilot's backticked handoff was.
fn warn_inline_body(payload: &str) {
    if payload.contains('`') || payload.contains("$(") {
        eprintln!(
            "warning: --body text carries a backtick or $( — a shell may have expanded it on \
             the way in; author prose with --body @<file> or --body - (stdin), which never \
             transit shell quoting"
        );
    }
}

/// Refuse at authoring what close would refuse at the gate (mw-8e769q0):
/// a keyword-led verify that does not parse never reaches a file.
/// Legacy shell and well-formed DSL pass untouched.
pub(crate) fn refuse_malformed(verify: &str) -> Result<(), String> {
    match crate::verify_dsl::classify(verify) {
        crate::verify_dsl::Classified::Malformed(why) => {
            Err(crate::verify_dsl::malformed_refusal(verify, &why))
        }
        _ => Ok(()),
    }
}

fn scalar_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| yaml_scalar(s))
        .collect::<Vec<_>>()
        .join(", ")
}
