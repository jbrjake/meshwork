//! Pin hygiene (MW-T3) and drift (MW-T4, MW-T6) over every live task's
//! `covers:`. An entry the tool did not write — a bare ref, a missing
//! hash, a hash that is not the hex `cover` writes, or a ref that is not
//! a clause ref at all — is an error, and the finding names the command
//! that mints a real pin. A real pin whose clause reads differently now
//! is `spec-drift`, a warning: the fix is a human re-reading the clause
//! and re-pinning, never a silent update. The clause is hashed from
//! disk, so an unversioned spec drifts just as visibly — what version
//! control would add is who moved it. A done task's pins are history;
//! a pin whose clause is gone is `spec audit`'s dangling row.

use crate::lint::{finding, Finding, Severity};
use crate::parse::{ParsedTask, Status};
use crate::store::RepoStore;

/// The pin checks over every live task.
pub fn check(store: &RepoStore, out: &mut Vec<Finding>) {
    for entry in &store.entries {
        let ParsedTask::Valid(t) = &entry.parsed else {
            continue;
        };
        if !matches!(t.status, Status::Open | Status::Doing | Status::Blocked) {
            continue;
        }
        for c in &t.covers {
            let sha = match (crate::spec::parse_ref(&c.reference), c.sha.as_deref()) {
                (Ok(_), Some(sha)) if crate::spec::is_sha(sha) => sha,
                (parsed, sha) => {
                    let why = match (parsed, sha) {
                        (Err(e), _) => e,
                        (Ok(_), None) => "no sha".to_string(),
                        (Ok(_), Some(sha)) => format!("sha `{sha}` is not the hex the tool writes"),
                    };
                    out.push(finding(
                        Severity::Error,
                        "covers-malformed",
                        &t.id,
                        format!(
                            "covers entry `{}` was not written by cover ({why}) — \
                             `meshwork cover {} {} --repin` mints the pin",
                            c.reference, t.id, c.reference
                        ),
                    ));
                    continue;
                }
            };
            let Ok(clause) = crate::spec::resolve(&store.root, &c.reference) else {
                continue;
            };
            if clause.sha != sha {
                out.push(finding(
                    Severity::Warning,
                    "spec-drift",
                    &t.id,
                    format!(
                        "clause `{}` reads differently from its pin (pinned {}, now {}) — \
                         re-read it, then `meshwork cover {} {} --repin`",
                        c.reference,
                        &sha[..12],
                        &clause.sha[..12],
                        t.id,
                        c.reference
                    ),
                ));
            }
        }
    }
}
