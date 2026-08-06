//! `comment` + `attach` (PLAN 1.4; MW-K1–K3). Comments are append-only
//! with self-professed identity, recorded as claimed; attachments live in
//! git under `docs/meshwork/attachments/<id>/` and are referenced from
//! frontmatter. Excerpt-first: lint warns past 1MB.

use crate::edit::{append_section_entry, set_list};
use crate::parse::{parse_task_file, ParsedTask};
use crate::store::find_task_file;
use std::path::PathBuf;

#[derive(clap::Args)]
pub(crate) struct CommentArgs {
    /// Task id.
    id: String,
    /// Comment text; newlines become continuation lines.
    text: String,
    /// Author identity — a free string, a claim (MW-K1). Falls back to
    /// the `MESHWORK_AUTHOR` env var, then config `default_author`.
    #[arg(long = "as", value_name = "AUTHOR")]
    author: Option<String>,
}

#[derive(clap::Args)]
pub(crate) struct AttachArgs {
    /// Task id.
    id: String,
    /// File to copy into docs/meshwork/attachments/<id>/.
    path: PathBuf,
    /// Overwrite an existing attachment of the same name.
    #[arg(long)]
    force: bool,
}

pub(crate) fn comment(args: &CommentArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("docs").join("meshwork");
    let Some(path) = find_task_file(&tasks_dir, &args.id) else {
        return Err(format!("{} not found", args.id));
    };

    let author = match &args.author {
        Some(a) => a.clone(),
        None => match std::env::var("MESHWORK_AUTHOR")
            .ok()
            .filter(|v| !v.trim().is_empty())
        {
            Some(a) => a.trim().to_string(),
            None => crate::store::load_config(&root)
                .map_err(|e| e.to_string())?
                .default_author
                .ok_or(
                    "no author: use --as <author>, set MESHWORK_AUTHOR, or add \
                     default_author to docs/meshwork/config.toml (MW-K1)",
                )?,
        },
    };
    if author.contains(']') || author.contains('\n') {
        return Err("author must not contain `]` or newlines — it delimits the format".into());
    }

    let today = crate::clock::today();
    let text_flat = args.text.replace('\r', "");
    let entry = format!("{today} [{author}] {}", text_flat.replace('\n', "\n  "));
    let file = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let file = append_section_entry(&file, "comments", &entry);
    std::fs::write(&path, file).map_err(|e| e.to_string())?;

    if json {
        crate::cli::emit_json(
            "comment",
            &serde_json::json!({ "id": args.id, "author": author, "date": today }),
        );
    } else {
        println!("{}: comment added as [{author}]", args.id);
    }
    Ok(())
}

pub(crate) fn attach(args: &AttachArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("docs").join("meshwork");
    let Some(task_path) = find_task_file(&tasks_dir, &args.id) else {
        return Err(format!("{} not found", args.id));
    };
    let task = match parse_task_file(&task_path) {
        ParsedTask::Valid(t) => t,
        ParsedTask::Invalid(inv) => {
            return Err(format!(
                "{} is invalid ({}) — repair first",
                args.id, inv.error
            ))
        }
    };
    if !args.path.is_file() {
        return Err(format!("{} is not a readable file", args.path.display()));
    }
    let name = args
        .path
        .file_name()
        .ok_or("attachment path has no file name")?
        .to_string_lossy()
        .into_owned();

    let dest_dir = root
        .join("docs")
        .join("meshwork")
        .join("attachments")
        .join(&args.id);
    let dest = dest_dir.join(&name);
    if dest.exists() && !args.force {
        return Err(format!(
            "{} already exists — pass --force to overwrite",
            dest.display()
        ));
    }
    std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
    let bytes = std::fs::copy(&args.path, &dest).map_err(|e| e.to_string())?;

    let rel = format!("attachments/{}/{name}", args.id);
    let mut list = task.attachments.clone();
    if !list.contains(&rel) {
        list.push(rel.clone());
    }
    let file = std::fs::read_to_string(&task_path).map_err(|e| e.to_string())?;
    let file = set_list(&file, "attachments", &list)?;
    std::fs::write(&task_path, file).map_err(|e| e.to_string())?;

    if bytes > 1_048_576 && !json {
        eprintln!(
            "note: {bytes} bytes >1MB — lint will warn; a 50-line excerpt usually \
             carries the signal (MW-K3)"
        );
    }
    if json {
        crate::cli::emit_json(
            "attach",
            &serde_json::json!({ "id": args.id, "path": rel, "bytes": bytes }),
        );
    } else {
        println!("attached {rel} ({bytes} bytes)");
    }
    Ok(())
}
