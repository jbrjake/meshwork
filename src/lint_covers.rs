//! Pin hygiene (MW-T3): a `covers:` entry the tool did not write — a bare
//! ref, a missing hash, a hash that is not the hex `cover` writes, or a
//! ref that is not a clause ref at all — is an error on a live task, and
//! the finding names the command that mints a real pin. Drift (a real pin
//! whose clause moved) is a different finding and a different task.

use crate::lint::{finding, Finding, Severity};
use crate::parse::{ParsedTask, Status};
use crate::store::RepoStore;

/// The hand-written-pin check over every live task.
pub fn check(store: &RepoStore, out: &mut Vec<Finding>) {
    for entry in &store.entries {
        let ParsedTask::Valid(t) = &entry.parsed else {
            continue;
        };
        if !matches!(t.status, Status::Open | Status::Doing | Status::Blocked) {
            continue;
        }
        for c in &t.covers {
            let why = match (crate::spec::parse_ref(&c.reference), c.sha.as_deref()) {
                (Err(e), _) => e,
                (Ok(_), None) => "no sha".to_string(),
                (Ok(_), Some(sha)) if !crate::spec::is_sha(sha) => {
                    format!("sha `{sha}` is not the hex the tool writes")
                }
                (Ok(_), Some(_)) => continue,
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
        }
    }
}
