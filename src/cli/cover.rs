//! `cover <task> <ref> [--repin]` (mw-psbn61z, MW-T3): write a clause pin —
//! the ref plus the SHA-256 of the clause's text as it reads right now —
//! into the task's `covers:`. A pin says "I implement this clause as it
//! read when I said so"; a second `cover` of the same ref refuses unless
//! `--repin`, which re-reads the clause after a human has. `--repin` with
//! no ref re-reads every pin the task carries. A hand-written pin is what
//! lint's `covers-malformed` names, and `--repin` is its fix.

use crate::parse::ParsedTask;

#[derive(clap::Args)]
pub(crate) struct CoverArgs {
    /// The task to pin.
    task: String,
    /// A clause ref: <path>#sp-<slug>, or <repo>#<path>#sp-<slug> for a
    /// doc in a registered sibling repo.
    reference: Option<String>,
    /// Re-read the clause and replace its pin — every pin of the task
    /// when no ref is given.
    #[arg(long)]
    repin: bool,
}

pub(crate) fn run(args: &CoverArgs, json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let tasks_dir = root.join("docs").join("meshwork");
    let Some(located) = crate::archive::locate(&tasks_dir, &args.task) else {
        return Err(format!("{} not found", args.task));
    };
    let task = match located.parse() {
        ParsedTask::Valid(t) => t,
        ParsedTask::Invalid(inv) => {
            return Err(format!(
                "{} is invalid ({}) — repair first",
                args.task, inv.error
            ))
        }
    };

    let mut pins: Vec<(String, String)> = task
        .covers
        .iter()
        .map(|c| (c.reference.clone(), c.sha.clone().unwrap_or_default()))
        .collect();
    let mut written: Vec<(String, String)> = Vec::new();
    match (&args.reference, args.repin) {
        (Some(text), repin) => {
            let reference = crate::spec::parse_ref(text)?.render();
            let clause = crate::spec::resolve(&root, &reference)?;
            match pins.iter().position(|(r, _)| *r == reference) {
                Some(i) if repin => pins[i].1.clone_from(&clause.sha),
                Some(i) if pins[i].1 == clause.sha => {
                    return Err(format!(
                        "{} already covers {reference} at this hash — nothing to do",
                        args.task
                    ))
                }
                Some(_) => {
                    return Err(format!(
                        "{} already covers {reference}, and the clause reads differently \
                         now — re-read it, then `meshwork cover {} {reference} --repin`",
                        args.task, args.task
                    ))
                }
                None => pins.push((reference.clone(), clause.sha.clone())),
            }
            written.push((reference, clause.sha));
        }
        (None, true) => {
            if pins.is_empty() {
                return Err(format!("{} covers nothing — nothing to re-pin", args.task));
            }
            for (reference, sha) in &mut pins {
                let clause = crate::spec::resolve(&root, reference)?;
                sha.clone_from(&clause.sha);
                written.push((reference.clone(), clause.sha));
            }
        }
        (None, false) => {
            return Err(format!(
                "cover needs a clause ref: meshwork cover {} <path>#sp-<slug> — or --repin \
                 to re-read every pin the task carries",
                args.task
            ))
        }
    }

    let entries: Vec<Vec<(&str, String)>> = pins
        .iter()
        .map(|(r, s)| vec![("ref", r.clone()), ("sha", s.clone())])
        .collect();
    let mut text = crate::edit::set_entry_list(&located.read()?, "covers", &entries)?;
    let stamp = crate::clock::stamp();
    for (reference, sha) in &written {
        text = crate::edit::append_section_entry(
            &text,
            "log",
            &format!("{stamp} cover {reference} @{}", short(sha)),
        );
    }
    located.write(&text)?;

    if json {
        let pin = |(r, s): &(String, String)| serde_json::json!({ "ref": r, "sha": s });
        crate::cli::emit_json(
            "cover",
            &serde_json::json!({ "id": args.task,
                "pinned": written.iter().map(pin).collect::<Vec<_>>(),
                "covers": pins.iter().map(pin).collect::<Vec<_>>() }),
        );
    } else {
        for (reference, sha) in &written {
            println!(
                "{} covers {} @{}",
                args.task,
                crate::cli::sanitize(reference),
                short(sha)
            );
        }
    }
    Ok(())
}

/// The first twelve hex characters — enough to read a diff by.
fn short(sha: &str) -> &str {
    &sha[..sha.len().min(12)]
}
