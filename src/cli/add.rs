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
    /// Several tasks at once from a file ("-" = stdin): concatenated §2
    /// documents, `id:` omitted, local `handle:` names usable as @refs in
    /// needs/parent/from/relates — atomic, all files or none (mw-af4kbjy).
    #[arg(long, value_name = "FILE", conflicts_with_all = ["title", "cat", "label", "needs", "parent", "from", "verify", "seq", "docs"])]
    batch: Option<String>,
    /// Print the would-be task file(s), write nothing (mw-0wvndqa).
    #[arg(long)]
    dry_run: bool,
    /// Category slash-path, e.g. engine/spill (MW-B4).
    #[arg(long = "cat", value_name = "PATH")]
    cat: Option<String>,
    /// Cross-cutting label; repeatable (MW-B5).
    #[arg(long = "label", value_name = "LABEL")]
    label: Vec<String>,
    /// Hard dependency id (`repo#id` crosses repos); repeatable (MW-B1/B3).
    #[arg(long = "needs", value_name = "ID")]
    needs: Vec<String>,
    /// Same-repo parent id (MW-B1; parent never crosses repos, MW-B3).
    #[arg(long, value_name = "ID")]
    parent: Option<String>,
    /// Provenance: the task this one was discovered from (MW-E4).
    #[arg(long = "from", value_name = "ID")]
    from: Option<String>,
    /// Verify command `close` runs via `sh -c` (MW-E2).
    #[arg(long, value_name = "CMD")]
    verify: Option<String>,
    /// Per-repo order weight, lower sooner; gaps of 10 (MW-G4, mw-0f4j).
    #[arg(long, value_name = "N")]
    seq: Option<i64>,
    /// Doc link `path#§-anchor`; repeatable (MW-F1, mw-0f4j).
    #[arg(long = "docs", value_name = "LINK")]
    docs: Vec<String>,
}

pub(crate) fn run(args: &AddArgs, json: bool) -> Result<(), String> {
    if let Some(source) = &args.batch {
        return super::add_batch::run(source, args.dry_run, json);
    }
    let root = crate::cli::require_store_root()?;
    let config = crate::store::load_config(&root).map_err(|e| e.to_string())?;
    if args.parent.as_deref().is_some_and(|p| p.contains('#')) {
        return Err(
            "parent must stay in-repo — hierarchy never crosses repos (MW-B3); \
                    use sequence.md tranches for portfolio grouping"
                .to_string(),
        );
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
    let mut fm = String::new();
    let _ = writeln!(fm, "id: {id}");
    let _ = writeln!(fm, "title: {}", yaml_scalar(&title));
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

    let file = format!("---\n{fm}---\n\n## log\n- {today} created\n");
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

    if json {
        crate::cli::emit_json("add", &serde_json::json!({ "id": id, "path": rel }));
    } else {
        println!("{id}");
        println!("  {rel}");
        if args.verify.is_none() {
            eprintln!("note: no --verify set — lint will warn until it is (MW-E2)");
        }
    }
    Ok(())
}

fn scalar_list(items: &[String]) -> String {
    items
        .iter()
        .map(|s| yaml_scalar(s))
        .collect::<Vec<_>>()
        .join(", ")
}
