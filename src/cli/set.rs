//! `meshwork set` (mw-0f4j, README spec): field edits on an existing task
//! without opening the file. Hand-editing stays legal (MW-A1) — this verb
//! just means it is never the *only* path (supersedes the §7b hand-edit
//! ruling for `seq:`, `docs:`, `handoff:`; extended to `category:`,
//! `verify:`, `title:` by the §6 ruling 2026-08-10, mw-f1x71yg — nine
//! pilot sessions python-rewrote task files for exactly these edits; and
//! to `to:`, `answers:`, `relates:`, the body, `parent:`,
//! `discovered-from:` and a `--docs <old> <new>` replacement by MW-L1,
//! ruled 2026-09-20 — 723 hand-edits across 210 sessions were the cost of
//! their absence). Edits are surgical (edit.rs), so union merges stay
//! clean and hand-written `# …` comments survive.

use crate::edit::{
    append_block_item, append_section_entry, remove_scalar, replace_block_item, set_block,
    set_body, set_list, set_scalar,
};
use crate::parse::ParsedTask;
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
    #[command(flatten)]
    docs: DocsEdits,
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
    /// Body prose, replacing the description above the tail sections;
    /// `@<file>` reads the file, `-` reads stdin, an empty payload clears it.
    #[arg(long, value_name = "TEXT|@FILE|-")]
    body: Option<String>,
    /// Same-repo parent id, set or replaced; a parent never crosses repos.
    #[arg(long, value_name = "ID")]
    parent: Option<String>,
    /// Provenance, set or replaced: the task this one was discovered from.
    #[arg(long = "from", value_name = "ID")]
    from: Option<String>,
    /// Soft link to a related task (`repo#id` crosses repos), appended;
    /// repeatable.
    #[arg(long = "relates", value_name = "ID")]
    relates: Vec<String>,
    /// Address this task to another repo as an ask (`repo` or `repo#id`),
    /// set or replaced; it stays in this store and surfaces in theirs.
    #[arg(long = "to", value_name = "REPO")]
    to: Option<String>,
    /// The ask (`repo#id`) this task answers, set or replaced; nothing is
    /// sent.
    #[arg(long = "answers", value_name = "GID")]
    answers: Option<String>,
}

/// `--docs` occurrences, each its own group: one link appends, two
/// (`<old> <new>`) replace one in place. The derive flattens repeated
/// values into one list, so the grouping is read off the matches here.
#[derive(Clone, Default)]
struct DocsEdits(Vec<Vec<String>>);

impl DocsEdits {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn iter(&self) -> impl Iterator<Item = &Vec<String>> {
        self.0.iter()
    }
}

impl clap::FromArgMatches for DocsEdits {
    fn from_arg_matches(m: &clap::ArgMatches) -> Result<Self, clap::Error> {
        Ok(Self(
            m.get_occurrences::<String>("docs")
                .map(|occ| occ.map(|group| group.cloned().collect()).collect())
                .unwrap_or_default(),
        ))
    }

    fn update_from_arg_matches(&mut self, m: &clap::ArgMatches) -> Result<(), clap::Error> {
        *self = Self::from_arg_matches(m)?;
        Ok(())
    }
}

impl clap::Args for DocsEdits {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd.arg(
            clap::Arg::new("docs")
                .long("docs")
                .alias("doc")
                .value_name("LINK [NEW]")
                .num_args(1..=2)
                .action(clap::ArgAction::Append)
                .help(
                    "Doc link `path#§-anchor` to append — or two links, `<old> <new>`, \
                     replacing one in place; repeatable",
                ),
        )
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

fn nothing_to_set(args: &SetArgs) -> bool {
    args.seq.is_none()
        && args.docs.is_empty()
        && args.handoff.is_none()
        && args.cat.is_none()
        && args.verify.is_none()
        && args.title.is_none()
        && args.body.is_none()
        && args.parent.is_none()
        && args.from.is_none()
        && args.relates.is_empty()
        && args.to.is_none()
        && args.answers.is_none()
}

/// Refuse controls before touching the file (mw-3tzfqmq) — every
/// frontmatter-bound field takes the same door.
fn reject_control_fields(args: &SetArgs) -> Result<(), String> {
    for (field, value) in [
        ("category", &args.cat),
        ("verify", &args.verify),
        ("parent", &args.parent),
        ("discovered-from", &args.from),
        ("to", &args.to),
        ("answers", &args.answers),
    ] {
        if let Some(value) = value {
            crate::cli::reject_controls(field, value, false)?;
        }
    }
    if let Some(title) = &args.title {
        crate::cli::reject_controls("title", &title.replace(['\n', '\r'], " "), false)?;
    }
    for link in args.docs.iter().flatten() {
        crate::cli::reject_controls("docs link", link, false)?;
    }
    for id in &args.relates {
        crate::cli::reject_controls("relates", id, false)?;
    }
    Ok(())
}

/// The edge targets `set` names, validated exactly as `add` validates its
/// own (mw-tkgvsdz): a same-repo target must exist, a cross-repo one is
/// the registry's to warn about.
fn check_targets(args: &SetArgs, tasks_dir: &std::path::Path) -> Result<(), String> {
    if args.parent.as_deref().is_some_and(|p| p.contains('#')) {
        return Err("parent must stay in-repo — hierarchy never crosses repos; \
                    use sequence.md tranches for portfolio grouping"
            .to_string());
    }
    let mut targets: Vec<(&str, &str)> = Vec::new();
    targets.extend(args.parent.iter().map(|p| ("parent", p.as_str())));
    targets.extend(args.from.iter().map(|f| ("discovered-from", f.as_str())));
    targets.extend(args.relates.iter().map(|r| ("relates", r.as_str())));
    targets.extend(args.answers.iter().map(|a| ("answers", a.as_str())));
    super::add::check_edge_targets(tasks_dir, &targets, &[])?;
    if let Some(to) = &args.to {
        super::add::warn_unresolvable_to(to, "");
    }
    Ok(())
}

pub(crate) fn run(args: &SetArgs, json: bool) -> Result<(), String> {
    if nothing_to_set(args) {
        return Err(
            "nothing to set — pass --seq, --docs, --handoff, --cat, --verify, --title, \
                    --body, --parent, --from, --relates, --to, and/or --answers"
                .to_string(),
        );
    }
    let root = crate::cli::require_store_root()?;
    let tasks_dir = crate::store::tasks_dir(&root);
    let Some(located) = crate::archive::locate(&tasks_dir, &args.id) else {
        return Err(format!("{} not found in {}", args.id, tasks_dir.display()));
    };
    reject_control_fields(args)?;
    if let Some(verify) = &args.verify {
        super::add::refuse_malformed(verify)?;
    }
    check_targets(args, &tasks_dir)?;

    let text = located.read()?;
    // What the lists hold today — an append must not lose it.
    let current = match located.parse() {
        ParsedTask::Valid(t) => t,
        ParsedTask::Invalid(inv) => {
            return Err(format!(
                "{}: INVALID — {}; repair before editing (lint --fix)",
                args.id, inv.error
            ))
        }
    };
    let (text, set_fields) = apply_fields(args, &root, text, &current)?;
    located.write(&text)?;
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

/// Every field edit in flag order over the file text; returns the new
/// text and the names of the fields it set, for the report.
fn apply_fields(
    args: &SetArgs,
    root: &std::path::Path,
    mut text: String,
    current: &crate::parse::Task,
) -> Result<(String, Vec<&'static str>), String> {
    let mut set_fields: Vec<&'static str> = Vec::new();
    if let Some(seq) = args.seq {
        text = set_scalar(&text, "seq", Some(&seq.to_string()))?;
        set_fields.push("seq");
    }
    for group in args.docs.iter() {
        text = match group.as_slice() {
            [link] => append_block_item(&text, "docs", link)?,
            [old, new] => replace_block_item(&text, "docs", old, new)?,
            _ => return Err("--docs takes one link to append, or <old> <new> to replace".into()),
        };
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
            // MW-S10 (mw-2n6zkx9): the voice gets an author and an age —
            // a log line prime renders as `[handoff by <author>, Nd]`, so
            // a previous session's voice never reads as an owner ruling.
            let mut entry = format!("{} handoff", crate::clock::stamp());
            if let Some(author) = crate::cli::notes::resolve_author(root, None)? {
                entry.push_str(" by ");
                entry.push_str(&author);
            }
            text = append_section_entry(&text, "log", &entry);
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
    if let Some(body) = &args.body {
        let payload = crate::cli::prose_payload(body)?;
        crate::cli::reject_controls("body", &payload, true)?;
        text = set_body(&text, &payload)?;
        set_fields.push("body");
    }
    if let Some(parent) = &args.parent {
        text = set_scalar(&text, "parent", Some(&yaml_scalar(parent)))?;
        set_fields.push("parent");
    }
    if let Some(from) = &args.from {
        text = set_scalar(&text, "discovered-from", Some(&yaml_scalar(from)))?;
        set_fields.push("discovered-from");
    }
    if !args.relates.is_empty() {
        let mut items = current.relates.clone();
        for id in &args.relates {
            if !items.contains(id) {
                items.push(id.clone());
            }
        }
        text = set_list(&text, "relates", &items)?;
        set_fields.push("relates");
    }
    if let Some(to) = &args.to {
        text = set_scalar(&text, "to", Some(&yaml_scalar(to)))?;
        set_fields.push("to");
    }
    if let Some(answers) = &args.answers {
        text = set_scalar(&text, "answers", Some(&yaml_scalar(answers)))?;
        set_fields.push("answers");
    }
    Ok((text, set_fields))
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
