//! `asks` (mw-jf47g1h, MW-M2): the inbox in full, both directions — every
//! ask addressed to this repo that no done task has answered, and every
//! ask this repo owes elsewhere — each with its answer's state and its
//! age. The same read-time join `prime` and `ready` render capped; this
//! is the verb their footnotes name, uncapped by definition. The union is
//! read once; nothing is written and nothing is sent.

use crate::addressed::{answered_suffix, Ask, Outbound};

pub(crate) fn run(json: bool) -> Result<(), String> {
    let root = crate::cli::require_store_root()?;
    let store = crate::store::load_repo(&root).map_err(|e| e.to_string())?;
    let union = crate::addressed::union();
    let inbox = union
        .as_deref()
        .map_or_else(Vec::new, |s| crate::addressed::inbox_of(s, &store.repo));
    let asks_out = crate::addressed::outbound(&store, union.as_deref());
    let today = crate::clock::today();

    if json {
        emit(&inbox, &asks_out, &today, union.is_some());
    } else {
        print_text(&inbox, &asks_out, &today, union.is_some());
    }
    Ok(())
}

/// Rows in the `ready` spellings, so what a session read there matches
/// what it reads here; a section with nothing says so, and an inbox with
/// no registry says why it is empty.
fn print_text(inbox: &[Ask], asks_out: &[Outbound], today: &str, registry: bool) {
    println!("asks in ({}):", inbox.len());
    if inbox.is_empty() {
        println!(
            "  (none{})",
            if registry {
                ""
            } else {
                " \u{2014} no registry loaded; inbound asks live in other stores"
            }
        );
    }
    for a in inbox {
        let age = a
            .age_days(today)
            .map_or(String::new(), |d| format!("  ({d}d)"));
        println!(
            "  {}  {}{age}{}",
            a.gid,
            crate::cli::sanitize(&a.title),
            answered_suffix(a.answer.as_ref(), "  ")
        );
    }
    println!("asks out ({}):", asks_out.len());
    if asks_out.is_empty() {
        println!("  (none)");
    }
    for a in asks_out {
        let age = a
            .age_days(today)
            .map_or(String::new(), |d| format!("  ({d}d)"));
        println!(
            "  {}  \u{2192} {}  {}{age}{}",
            a.id,
            crate::cli::sanitize(&a.to),
            crate::cli::sanitize(&a.title),
            answered_suffix(a.answer.as_ref(), "  ")
        );
    }
}

/// `in` and `out`, each row with its age and its answer as `ready --json`
/// spells them; `registry` says whether the inbox could be read at all.
fn emit(inbox: &[Ask], asks_out: &[Outbound], today: &str, registry: bool) {
    let answer = |a: Option<&crate::addressed::Answer>| crate::cli::query::answer_json(a);
    let inbound: Vec<_> = inbox
        .iter()
        .map(|a| {
            serde_json::json!({ "gid": a.gid, "title": a.title, "created": a.created,
                "age_days": a.age_days(today), "answered_by": answer(a.answer.as_ref()) })
        })
        .collect();
    let outbound: Vec<_> = asks_out
        .iter()
        .map(|a| {
            serde_json::json!({ "id": a.id, "title": a.title, "to": a.to,
                "created": a.created, "age_days": a.age_days(today),
                "answered_by": answer(a.answer.as_ref()) })
        })
        .collect();
    crate::cli::emit_json(
        "asks",
        &serde_json::json!({ "registry": registry, "in": inbound, "out": outbound }),
    );
}
