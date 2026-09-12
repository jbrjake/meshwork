//! Lint over the prose (mw-xb9prd6): the shapes a session's own text
//! takes when it is laundering authority. Lexical heuristics, warnings,
//! and each finding says it is one — the reader judges the intent.

use crate::lint::{finding, Finding, Severity};
use crate::parse::{Status, Task};

/// Phrases that invoke the owner's authority (matched case-insensitively).
const RULING_PHRASES: &[&str] = &["owner ruled", "owner ruling", "owner call"];

/// A quoted span: the owner's own words, the evidence a ruling rests on.
fn has_quote(text: &str) -> bool {
    let straight = text.matches('"').count() >= 2;
    let curly = text.contains('\u{201c}') && text.contains('\u{201d}');
    straight || curly
}

/// `ruling-without-quote`: a live task whose body, handoff or comments
/// invoke an owner ruling and quote nothing — a paraphrase wearing the
/// owner's authority.
pub(crate) fn check(valid: &[&Task], out: &mut Vec<Finding>) {
    for t in valid {
        if matches!(t.status, Status::Done | Status::Dropped) {
            continue;
        }
        let mut text = t.description.clone();
        text.push('\n');
        text.push_str(t.handoff.as_deref().unwrap_or_default());
        for c in &t.comments {
            text.push('\n');
            text.push_str(&c.text);
        }
        let lower = text.to_ascii_lowercase();
        let Some(phrase) = RULING_PHRASES.iter().find(|p| lower.contains(*p)) else {
            continue;
        };
        if has_quote(&text) {
            continue;
        }
        out.push(finding(
            Severity::Warning,
            "ruling-without-quote",
            &t.id,
            format!(
                "says `{phrase}` and quotes nothing — quote the owner's words, or it reads as \
                 the agent's paraphrase (a heuristic: the quote is the evidence)"
            ),
        ));
    }
}
