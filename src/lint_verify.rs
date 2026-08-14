//! Verify-shaped lint checks, split from `lint.rs` at the 500-line
//! target: the static tier of verify hygiene. Trivially-satisfiable
//! verifies (mw-221f3jt), plus the migration-pressure pair (mw-4aqmf0t,
//! DESIGN §12b): `verify-shell` warns on legacy shell text — legal
//! forever behind the MW-E5 gate, but loud so stores drift toward the
//! DSL — and `verify-malformed` warns on keyword-led text that will
//! refuse at close. Warnings all; nothing here executes anything.

use crate::lint::{finding, Finding, Severity};
use crate::parse::{Status, Task};
use crate::store::RepoStore;
use crate::verify_dsl::{classify, Classified};

/// All verify-shaped checks over the live tasks.
pub(crate) fn check(store: &RepoStore, valid: &[&Task], out: &mut Vec<Finding>) {
    for t in valid {
        if matches!(t.status, Status::Done | Status::Dropped) {
            continue;
        }
        let Some(v) = t.verify.as_deref().map(str::trim) else {
            continue;
        };
        if let Some(why) = trivial_reason(&store.root, v) {
            out.push(finding(
                Severity::Warning,
                "verify-trivial",
                &t.id,
                format!("verify `{v}` {why} — it cannot detect the work"),
            ));
        }
        match classify(v) {
            Classified::LegacyShell => out.push(finding(
                Severity::Warning,
                "verify-shell",
                &t.id,
                format!(
                    "verify `{v}` is legacy shell — runs only behind the per-clone \
                     MW-E5 gate; prefer the DSL (exists/absent/contains/run, DESIGN §12b)"
                ),
            )),
            Classified::Malformed(why) => out.push(finding(
                Severity::Warning,
                "verify-malformed",
                &t.id,
                format!("verify `{v}` is keyword-led but does not parse ({why}) — close will refuse it (DESIGN §12b)"),
            )),
            Classified::Dsl(_) => {}
        }
    }
}

/// mw-221f3jt: why a verify is trivially satisfiable, if it is. Exact
/// shapes only: compound commands (`&&`, `|`, `;`) are someone's real
/// gate — unjudged. The start red-check (mw-175bn4c) is the dynamic
/// tier; this static tier never executes and never touches the gate.
fn trivial_reason(root: &std::path::Path, v: &str) -> Option<String> {
    let toks: Vec<&str> = v.split_whitespace().collect();
    let present =
        |path: &str| !path.starts_with('/') && !path.contains("..") && root.join(path).exists();
    match toks.as_slice() {
        ["true" | ":"] | ["exit", "0"] => Some("always exits 0".into()),
        ["echo", ..] => Some("echo always exits 0".into()),
        ["touch", ..] => Some("touch satisfies itself".into()),
        ["test", "-f" | "-e", path] | ["exists", path] if present(path) => {
            Some(format!("is already green ({path} exists today)"))
        }
        _ => None,
    }
}
