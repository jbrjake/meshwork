//! MW-E5 trust gate (mw-9rc4vs6, DESIGN §12b): task files arrive via git
//! merge and are untrusted input, so a shell `verify:` runs only after the
//! operator of THIS clone approved its exact text — trust-on-first-use,
//! the direnv-allow pattern. Approvals are SHA-256 content hashes over
//! (task id, verify text) in the gitignored `.cache/`: per-clone state
//! that can never arrive via merge, exactly because merged content is
//! what's untrusted. `MESHWORK_TRUST=1` is the deliberate whole-checkout
//! grant for CI/gate/test contexts. Git authorship is never consulted.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Approval-hash file, one lowercase hex hash per line. Lives under the
/// self-gitignoring `.cache/` (DESIGN §1) — losing it merely re-gates.
fn approvals_path(root: &Path) -> PathBuf {
    root.join("docs")
        .join("meshwork")
        .join(".cache")
        .join("trusted-verifies")
}

/// SHA-256 over `id NUL verify` — binding the approval to the task keeps
/// an approved text on one task from blessing the same text smuggled onto
/// another with a different blast radius.
fn approval_hash(id: &str, verify: &str) -> String {
    use std::fmt::Write as _;
    let mut h = Sha256::new();
    h.update(id.as_bytes());
    h.update([0]);
    h.update(verify.as_bytes());
    let mut hex = String::with_capacity(64);
    for b in h.finalize() {
        let _ = write!(hex, "{b:02x}");
    }
    hex
}

/// The reviewed-checkout grant: `MESHWORK_TRUST=1`, set per invocation by
/// an operator (or CI) vouching for every `verify:` in the checkout.
#[must_use]
pub fn env_trusted() -> bool {
    std::env::var("MESHWORK_TRUST").is_ok_and(|v| v.trim() == "1")
}

/// Has this clone's operator approved exactly this (id, verify text)?
/// Unreadable or absent state is simply "no" — conservative, never an
/// error (the cache is never a dependency, MW-A2).
#[must_use]
pub fn is_approved(root: &Path, id: &str, verify: &str) -> bool {
    let hash = approval_hash(id, verify);
    std::fs::read_to_string(approvals_path(root))
        .is_ok_and(|text| text.lines().any(|l| l.trim() == hash))
}

/// Record approval for (id, verify text) in this clone. Idempotent.
///
/// # Errors
/// Filesystem failures creating `.cache/` or appending the hash.
pub fn record_approval(root: &Path, id: &str, verify: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    if is_approved(root, id, verify) {
        return Ok(());
    }
    let path = approvals_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(f, "{}", approval_hash(id, verify))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approve_roundtrip_and_revocation_on_change() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert!(!is_approved(root, "az-k7f3", "true"));
        record_approval(root, "az-k7f3", "true").unwrap();
        assert!(is_approved(root, "az-k7f3", "true"));
        // Exact-text binding: any change re-gates.
        assert!(!is_approved(root, "az-k7f3", "true "));
        // Task-id binding: same text on another task is its own decision.
        assert!(!is_approved(root, "az-zzzz", "true"));
        // Idempotent: no duplicate lines.
        record_approval(root, "az-k7f3", "true").unwrap();
        let text = std::fs::read_to_string(approvals_path(root)).unwrap();
        assert_eq!(text.lines().count(), 1);
    }
}
