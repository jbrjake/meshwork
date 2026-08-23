//! `meshwork set` (mw-0f4j, README spec): field edits on an existing task
//! without opening the file. Hand-editing stays legal (MW-A1) — this verb
//! just means it is never the *only* path (supersedes the §7b hand-edit
//! ruling for `seq:`, `docs:`, `handoff:`; extended to `category:`,
//! `verify:`, `title:` by the §6 ruling 2026-08-10, mw-f1x71yg — nine
//! pilot sessions python-rewrote task files for exactly these edits).
//! Edits are surgical (edit.rs), so union merges stay clean and
//! hand-written `# …` comments survive.

use crate::edit::{append_block_item, remove_scalar, set_block, set_scalar};
use crate::store::find_task_file;
use crate::write::yaml_scalar;

/// Wrap width for `handoff:` block lines — readable files, readable `»`
/// rendering in prime (DESIGN §7b).
const HANDOFF_WRAP: usize = 72;

#[derive(clap::Args)]
pub(crate) struct SetArgs {
    /// Task id.
    id: String,
    /// Per-repo order weight, lower sooner; gaps of 10.
    #[arg(long, value_name = "N")]
    seq: Option<i64>,
    /// Doc link `path#§-anchor` to append; repeatable.
    #[arg(long = "docs", alias = "doc", value_name = "LINK")]
    docs: Vec<String>,
    /// Handoff voice to the next session; replaces the block.
    /// `@<file>` reads the file, `-` reads stdin — prose never transits
    /// shell quoting.
    #[arg(long, value_name = "TEXT|@FILE|-")]
    handoff: Option<String>,
    /// Category slash-path.
    #[arg(long = "cat", alias = "category", value_name = "PATH")]
    cat: Option<String>,
    /// Verify command `close` runs; text you author here is approved for
    /// this clone at write — other clones still prompt on it.
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

    // Refuse controls before touching the file (mw-3tzfqmq).
    for (field, value) in [("category", &args.cat), ("verify", &args.verify)] {
        if let Some(value) = value {
            crate::cli::reject_controls(field, value, false)?;
        }
    }
    if let Some(title) = &args.title {
        crate::cli::reject_controls("title", &title.replace(['\n', '\r'], " "), false)?;
    }
    for link in &args.docs {
        crate::cli::reject_controls("docs link", link, false)?;
    }

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
        let payload = crate::cli::prose_payload(handoff)?;
        crate::cli::reject_controls("handoff", &payload, true)?;
        // Empty clears the key outright (mw-gbep3j8): a dangling
        // `handoff: |` with nothing under it reads as absent everywhere
        // else — writing it just leaves a vestige to hand-clean.
        if payload.trim().is_empty() {
            text = remove_scalar(&text, "handoff")?;
        } else {
            text = set_block(&text, "handoff", &wrap(&payload, HANDOFF_WRAP))?;
        }
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
    // Approve-at-mint (§12b as amended 2026-08-21, mw-51x0wty): replacing
    // the text is authoring it — record this clone's approval; every
    // other clone re-gates on the new text by construction. Best-effort.
    if let Some(verify) = &args.verify {
        if let Err(e) = crate::trust::record_approval(&root, &args.id, verify) {
            eprintln!("warning: could not record verify approval: {e}");
        }
    }

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
