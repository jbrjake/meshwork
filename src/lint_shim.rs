//! The committed shim, `docs/meshwork/meshwork`, against the canonical text
//! the plugin ships: the backstop for a session whose plugin hook could not
//! run. A pinned project — one carrying `.meshwork-version` — runs every
//! verb through its shim, so a shim that is missing, differs from the
//! canonical bytes, or is not executable is damage: `lint` names it
//! (`shim-missing`, `shim-stale`), `prime` leads with it, and `lint --fix`
//! rewrites it. A project without a pin builds meshwork from source and
//! runs it however it likes (this repo's own shim execs `target/debug/`),
//! so without a pin the shim is nobody's finding.

use crate::lint::{finding, Finding, Severity};
use crate::store::RepoStore;
use std::path::{Path, PathBuf};

/// The one canonical shim text, shipped with the plugin as `hooks/meshwork`
/// and embedded here so `init` writes it and `lint` compares against it.
pub const CANONICAL: &str = include_str!("../hooks/meshwork");

/// The release pin at the repo root; its presence makes the shim
/// canonical's business.
pub const PIN: &str = ".meshwork-version";

/// The shim's path, repo-relative, as every message names it.
pub const SHIM: &str = "docs/meshwork/meshwork";

/// Where the shim lives under `root`.
#[must_use]
pub fn shim_path(root: &Path) -> PathBuf {
    root.join("docs").join("meshwork").join("meshwork")
}

/// Whether `root` pins a release — the one condition under which the shim
/// is expected to be canonical.
#[must_use]
pub fn is_pinned(root: &Path) -> bool {
    root.join(PIN).is_file()
}

/// How the shim departs from canonical.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Drift {
    /// No file at the shim's path.
    Missing,
    /// The bytes differ; the first differing line, one-based.
    Text(usize),
    /// Canonical bytes, but no execute bit.
    NotExecutable,
}

impl Drift {
    /// The lint code.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Drift::Missing => "shim-missing",
            Drift::Text(_) | Drift::NotExecutable => "shim-stale",
        }
    }

    /// What is wrong and what mends it — lint's row and prime's line say
    /// the same words.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Drift::Missing => "missing in a pinned project, and every verb runs through it \
                               \u{2014} lint --fix writes the canonical shim"
                .to_string(),
            Drift::Text(line) => format!(
                "not the canonical shim (first difference at line {line}) \u{2014} lint --fix \
                 rewrites it; lint --explain shim-stale shows the diff"
            ),
            Drift::NotExecutable => {
                "the canonical shim, but not executable \u{2014} lint --fix sets the mode"
                    .to_string()
            }
        }
    }
}

fn executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        true
    }
}

/// The first line, one-based, where the two texts differ; one past the
/// shorter text when one is a prefix of the other.
fn first_difference(actual: &str, canonical: &str) -> usize {
    let mut ours = actual.lines();
    let mut theirs = canonical.lines();
    let mut line = 1;
    loop {
        match (ours.next(), theirs.next()) {
            (Some(mine), Some(canon)) if mine == canon => line += 1,
            _ => return line,
        }
    }
}

/// The drift of the shim under `root`, or `None` when it is canonical and
/// executable — or when `root` is unpinned, since then nothing is expected
/// of it.
#[must_use]
pub fn drift(root: &Path) -> Option<Drift> {
    if !is_pinned(root) {
        return None;
    }
    let path = shim_path(root);
    let Ok(actual) = std::fs::read(&path) else {
        return Some(Drift::Missing);
    };
    if actual != CANONICAL.as_bytes() {
        let text = String::from_utf8_lossy(&actual);
        return Some(Drift::Text(first_difference(&text, CANONICAL)));
    }
    (!executable(&path)).then_some(Drift::NotExecutable)
}

/// `lint`'s pass: one warning on the shim when it drifts.
pub fn check(store: &RepoStore, out: &mut Vec<Finding>) {
    if let Some(d) = drift(&store.root) {
        out.push(finding(Severity::Warning, d.code(), SHIM, d.message()));
    }
}

/// The shim against canonical, line by line: `-N` the shim's line, `+N`
/// the canonical one, only where they differ.
#[must_use]
pub fn diff_lines(root: &Path) -> Vec<String> {
    let actual = std::fs::read(shim_path(root))
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default();
    let ours: Vec<&str> = actual.lines().collect();
    let theirs: Vec<&str> = CANONICAL.lines().collect();
    let mut out = Vec::new();
    for at in 0..ours.len().max(theirs.len()) {
        match (ours.get(at), theirs.get(at)) {
            (Some(mine), Some(canon)) if mine == canon => {}
            (mine, canon) => {
                if let Some(mine) = mine {
                    out.push(format!("-{} {mine}", at + 1));
                }
                if let Some(canon) = canon {
                    out.push(format!("+{} {canon}", at + 1));
                }
            }
        }
    }
    out
}

/// `lint --fix`'s repair: the canonical shim, executable, when the project
/// is pinned and the shim drifts. Returns what it did, or `None` when
/// there was nothing to mend.
///
/// # Errors
/// The write or the mode change failing.
pub fn repair(root: &Path) -> Result<Option<&'static str>, String> {
    let Some(d) = drift(root) else {
        return Ok(None);
    };
    let path = shim_path(root);
    let what = match d {
        Drift::Missing => "written",
        Drift::Text(_) => "rewritten to the canonical text",
        Drift::NotExecutable => "made executable",
    };
    if d != Drift::NotExecutable {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&path, CANONICAL).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    mark_executable(&path)?;
    Ok(Some(what))
}

/// The shim is run, not sourced: `chmod 755` where the platform has modes.
///
/// # Errors
/// The mode change failing.
pub fn mark_executable(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("{}: {e}", path.display()))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_difference_is_one_based_and_past_the_end_for_a_prefix() {
        assert_eq!(first_difference("a\nb\n", "a\nc\n"), 2);
        assert_eq!(first_difference("a\n", "a\nb\n"), 2);
        assert_eq!(first_difference("x\n", "a\n"), 1);
    }

    #[test]
    fn unpinned_root_has_no_drift() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(drift(dir.path()), None);
        std::fs::write(dir.path().join(PIN), "v0.1.0\n").unwrap();
        assert_eq!(drift(dir.path()), Some(Drift::Missing));
        assert_eq!(repair(dir.path()).unwrap(), Some("written"));
        assert_eq!(drift(dir.path()), None);
        assert_eq!(repair(dir.path()).unwrap(), None);
    }
}
