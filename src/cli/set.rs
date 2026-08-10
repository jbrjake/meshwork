//! `meshwork set` (mw-0f4j, README spec): field edits on an existing task
//! without opening the file. Hand-editing stays legal (MW-A1) — this verb
//! just means it is never the *only* path (supersedes the §7b hand-edit
//! ruling for `seq:`, `docs:`, `handoff:`). Edits are surgical (edit.rs),
//! so union merges stay clean and hand-written `# …` comments survive.

use crate::edit::{append_block_item, set_block, set_scalar};
use crate::store::find_task_file;

/// Wrap width for `handoff:` block lines — readable files, readable `»`
/// rendering in prime (DESIGN §7b).
const HANDOFF_WRAP: usize = 72;

#[derive(clap::Args)]
pub(crate) struct SetArgs {
    /// Task id.
    id: String,
    /// Per-repo order weight, lower sooner; gaps of 10 (MW-G4).
    #[arg(long, value_name = "N")]
    seq: Option<i64>,
    /// Doc link `path#§-anchor` to append; repeatable (MW-F1). `--doc` is
    /// a hidden alias (mw-5hrb22q).
    #[arg(long = "docs", alias = "doc", value_name = "LINK")]
    docs: Vec<String>,
    /// Handoff voice to the next session; replaces the block (DESIGN §7b).
    #[arg(long, value_name = "TEXT")]
    handoff: Option<String>,
}

pub(crate) fn run(args: &SetArgs, json: bool) -> Result<(), String> {
    if args.seq.is_none() && args.docs.is_empty() && args.handoff.is_none() {
        return Err("nothing to set — pass --seq, --docs, and/or --handoff".to_string());
    }
    let root = crate::cli::require_store_root()?;
    let tasks_dir = crate::store::tasks_dir(&root);
    let Some(path) = find_task_file(&tasks_dir, &args.id) else {
        return Err(format!("{} not found in {}", args.id, tasks_dir.display()));
    };

    let mut text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let mut set_fields: Vec<&str> = Vec::new();
    if let Some(seq) = args.seq {
        text = set_scalar(&text, "seq", Some(&seq.to_string()))?;
        set_fields.push("seq");
    }
    for link in &args.docs {
        text = append_block_item(&text, "docs", link)?;
    }
    if !args.docs.is_empty() {
        set_fields.push("docs");
    }
    if let Some(handoff) = &args.handoff {
        text = set_block(&text, "handoff", &wrap(handoff, HANDOFF_WRAP))?;
        set_fields.push("handoff");
    }
    std::fs::write(&path, text).map_err(|e| e.to_string())?;

    if json {
        crate::cli::emit_json(
            "set",
            &serde_json::json!({ "id": args.id, "set": set_fields }),
        );
    } else {
        for field in &set_fields {
            println!("{} {field} set", args.id);
        }
    }
    Ok(())
}

/// Word-wrap to `width` columns; explicit newlines in the input are kept.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut out = Vec::new();
    for para in text.lines() {
        let mut line = String::new();
        for word in para.split_whitespace() {
            if !line.is_empty() && line.len() + 1 + word.len() > width {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        out.push(line);
    }
    out
}
