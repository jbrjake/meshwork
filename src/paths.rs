//! Repo-confined path resolution (mw-2pz0zqc; the DESIGN §12b-adjacent
//! surface). `docs:` links, attachment paths, and verify-DSL paths are
//! attacker-supplied strings from merged task files; every read joins
//! them onto the repo root through here. Lexical refusal (absolute, any
//! `..` segment) catches the textual escapes; the canonicalize-when-
//! exists comparison catches symlink escapes — a lexically clean path
//! whose real location leaves the repo refuses on the mismatch. A target
//! that does not exist passes on the lexical check alone: there is
//! nothing to read through, and refusing would turn every dead link into
//! the wrong error.

use std::path::{Path, PathBuf};

/// Join `rel` onto `root`, refusing every way out of the repo.
///
/// # Errors
/// When `rel` is absolute, contains a `..` segment, or resolves (through
/// any symlink in its chain) to a real location outside `root`.
pub fn confine(root: &Path, rel: &str) -> Result<PathBuf, String> {
    if Path::new(rel).is_absolute() || rel.split('/').any(|seg| seg == "..") {
        return Err(format!("unsafe path: {rel}"));
    }
    let joined = root.join(rel);
    // symlink_metadata sees a dangling symlink too — anything present on
    // disk must prove its real location stays inside the repo.
    if joined.symlink_metadata().is_ok() {
        let resolved = joined
            .canonicalize()
            .map_err(|e| format!("unsafe path: {rel} escapes the repo ({e})"))?;
        let resolved_root = root
            .canonicalize()
            .map_err(|e| format!("repo root unresolvable: {e}"))?;
        if !resolved.starts_with(&resolved_root) {
            return Err(format!("unsafe path: {rel} escapes the repo"));
        }
    }
    Ok(joined)
}

#[cfg(test)]
mod tests {
    use super::confine;

    #[test]
    fn confine_lexical_and_symlink() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("repo");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(dir.path().join("outside.txt"), "x").unwrap();
        std::fs::write(root.join("inside.txt"), "x").unwrap();

        assert!(confine(&root, "inside.txt").is_ok());
        assert!(
            confine(&root, "missing/nested.md").is_ok(),
            "dead links pass"
        );
        assert!(confine(&root, "/etc/hosts").is_err());
        assert!(confine(&root, "../outside.txt").is_err());
        assert!(confine(&root, "a/../../outside.txt").is_err());

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.path().join("outside.txt"), root.join("leak")).unwrap();
            assert!(confine(&root, "leak").is_err(), "symlink escape refused");
            std::os::unix::fs::symlink(root.join("inside.txt"), root.join("ok")).unwrap();
            assert!(confine(&root, "ok").is_ok(), "inside symlink passes");
        }
    }
}
