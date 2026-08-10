//! `meshwork set` (mw-0f4j, README spec): field edits on an existing task
//! without opening the file. Hand-editing stays legal (MW-A1) — this verb
//! just means it is never the *only* path (supersedes the §7b hand-edit
//! ruling for `seq:`, `docs:`, `handoff:`; extended to `category:`,
//! `verify:`, `title:` by the §6 ruling 2026-08-10, mw-f1x71yg — nine
//! pilot sessions python-rewrote task files for exactly these edits).
//! Edits are surgical (edit.rs), so union merges stay clean and
//! hand-written `# …` comments survive.

use crate::edit::{append_block_item, set_block, set_scalar};
use crate::store::find_task_file;
use crate::write::yaml_scalar;

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
    /// Doc link `path#§-anchor` to append; repeatable (MW-F1).
    #[arg(long = "docs", value_name = "LINK")]
    docs: Vec<String>,
    /// Handoff voice to the next session; replaces the block (DESIGN §7b).
    #[arg(long, value_name = "TEXT")]
    handoff: Option<String>,
    /// Category slash-path (MW-B4). Grown under the §6 ruling 2026-08-10
    /// (mw-f1x71yg).
    #[arg(long = "cat", value_name = "PATH")]
    cat: Option<String>,
    /// Verify command `close` runs (MW-E2); replacing it re-arms the MW-E5
    /// approval gate automatically (content-hash TOFU).
    #[arg(long, value_name = "CMD")]
    verify: Option<String>,
    /// One-line title. The filename slug is cosmetic and never renamed.
    #[arg(long, value_name = "TEXT")]
    title: Option<String>,
}

pub(crate) fn run(args: &SetArgs, json: bool) -> Result<(), String> {
    if args.seq.is_none()
        && args.docs.is_empty()
        && args.handoff.is_none()
        && args.cat.is_none()
        && args.verify.is_none()
        && args.title.is_none()
    {
        return Err(
            "nothing to set — pass --seq, --docs, --handoff, --cat, --verify, and/or --title"
                .to_string(),
        );
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
    if let Some(cat) = &args.cat {
        text = set_scalar(&text, "category", Some(&yaml_scalar(cat)))?;
        set_fields.push("category");
    }
    if let Some(verify) = &args.verify {
        text = set_scalar(&text, "verify", Some(&yaml_scalar(verify)))?;
        set_fields.push("verify");
    }
    if let Some(title) = &args.title {
        let title = title.replace(['\n', '\r'], " ");
        text = set_scalar(&text, "title", Some(&yaml_scalar(&title)))?;
        set_fields.push("title");
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
