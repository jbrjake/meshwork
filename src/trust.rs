//! MW-E5 trust gate (mw-9rc4vs6, DESIGN §12b): task files arrive via git
//! merge and are untrusted input, so a shell `verify:` runs only after the
//! operator of THIS clone approved its exact text — trust-on-first-use,
//! the direnv-allow pattern. Approvals are `id TAB text` records in the
//! gitignored `.cache/`: per-clone state that can never arrive via merge,
//! exactly because merged content is what's untrusted. Plaintext, not a
//! hash, so lint can show approved-vs-current when a verify is edited
//! after approval (mw-yyf1bab — the silent-weakening attack); bare
//! SHA-256 lines from the hash era still gate-pass, they just can't feed
//! the diff. `MESHWORK_TRUST=1` is the deliberate whole-checkout grant
//! for CI/gate/test contexts. Git authorship is never consulted.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Approvals file, one `id TAB verify-text` record per line (legacy
/// lines: bare lowercase hex hash). Lives under the self-gitignoring
/// `.cache/` (DESIGN §1) — losing it merely re-gates.
fn approvals_path(root: &Path) -> PathBuf {
    root.join("docs")
        .join("meshwork")
        .join(".cache")
        .join("trusted-verifies")
}

/// SHA-256 over `id NUL verify` — the hash-era line format, still honored
/// on read so pre-existing approvals don't silently re-gate. The id
/// binding (here and in the text format) keeps an approved text on one
/// task from blessing the same text smuggled onto another with a
/// different blast radius.
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
    std::fs::read_to_string(approvals_path(root)).is_ok_and(|text| {
        text.lines().any(|l| match l.split_once('\t') {
            Some((lid, ltext)) => lid == id && ltext == verify,
            None => l.trim() == hash,
        })
    })
}

/// What this clone approved, per task id — the newest text-format record
/// wins (re-approval supersedes). Hash-era lines carry no id and cannot
/// appear here; they still gate-pass, they just predate the diff. Feeds
/// the `verify-changed-since-approval` lint/prime finding (mw-yyf1bab).
#[must_use]
pub fn approved_texts(root: &Path) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    let Ok(text) = std::fs::read_to_string(approvals_path(root)) else {
        return out;
    };
    for line in text.lines() {
        if let Some((id, approved)) = line.split_once('\t') {
            out.insert(id.to_string(), approved.to_string());
        }
    }
    out
}

/// Record approval for (id, verify text) in this clone. Idempotent.
/// Appended, never rewritten — earlier records for the same id stay as
/// history; the newest wins for the diff, and every recorded text keeps
/// gate-passing (approval is of a text, not a revocation of older ones).
///
/// # Errors
/// Filesystem failures creating `.cache/` or appending the record.
pub fn record_approval(root: &Path, id: &str, verify: &str) -> std::io::Result<()> {
    use std::io::Write as _;
    if is_approved(root, id, verify)
        && approved_texts(root).get(id).map(String::as_str) == Some(verify)
    {
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
    writeln!(f, "{id}\t{verify}")
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

    /// mw-yyf1bab: approvals are readable back per id, newest text wins,
    /// and hash-era lines keep gate-passing without ever feeding the map.
    #[test]
    fn approved_texts_newest_wins_and_legacy_hashes_still_pass() {
        use std::fmt::Write as _;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        record_approval(root, "az-k7f3", "cargo test old").unwrap();
        record_approval(root, "az-k7f3", "cargo test new").unwrap();
        let map = approved_texts(root);
        assert_eq!(
            map.get("az-k7f3").map(String::as_str),
            Some("cargo test new")
        );
        // Both approved texts keep gate-passing — approval, not revocation.
        assert!(is_approved(root, "az-k7f3", "cargo test old"));
        assert!(is_approved(root, "az-k7f3", "cargo test new"));
        // Re-approving an older text makes it newest again — the operator
        // just reviewed it; lint must not flag a false "changed".
        record_approval(root, "az-k7f3", "cargo test old").unwrap();
        let map = approved_texts(root);
        assert_eq!(
            map.get("az-k7f3").map(String::as_str),
            Some("cargo test old")
        );

        // A hash-era line: gate-passes, invisible to the map.
        let path = approvals_path(root);
        let mut text = std::fs::read_to_string(&path).unwrap();
        writeln!(text, "{}", approval_hash("az-old1", "make check")).unwrap();
        std::fs::write(&path, text).unwrap();
        assert!(is_approved(root, "az-old1", "make check"));
        assert!(!approved_texts(root).contains_key("az-old1"));
    }
}
