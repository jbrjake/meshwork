//! Key derivation for the reserved `.cache/tasks.jsonl` projection
//! (FORMAT.md Projection; mw-n0r5jwm). The decision, recorded as code
//! before any cache exists: the freshness key is a hash of the store's
//! *content* — every projection input as sorted (relative path, bytes)
//! pairs — never mtime, never size/count heuristics, never inode
//! metadata. `git checkout` rewrites mtimes wholesale on every branch
//! switch, so an mtime key thrashes in exactly the worktree-heavy
//! workflow meshwork targets; the projection is a pure function of
//! content, so content is the key. Nothing builds the cache yet; when
//! it lands it MUST key on this.

use sha2::{Digest, Sha256};
use std::path::Path;

/// SHA-256 hex over the store's projection inputs: `config.toml` and
/// every `*.md` in `docs/meshwork/` and `docs/meshwork/archive/`, as
/// (relative path, contents) pairs in bytewise path order, each field
/// length-prefixed so boundaries can never alias. Deterministic across
/// platforms, clocks, and checkouts: same bytes, same key.
///
/// # Errors
/// Filesystem failures reading the store; an absent store directory is
/// an error, not an empty key — a cache must never validate against a
/// store it cannot see.
pub fn projection_key(root: &Path) -> std::io::Result<String> {
    use std::fmt::Write as _;
    let mw = root.join("docs").join("meshwork");
    let mut inputs: Vec<String> = vec!["config.toml".to_string()];
    for (dir, prefix) in [(mw.clone(), ""), (mw.join("archive"), "archive/")] {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) if !prefix.is_empty() => continue, // archive/ is optional
            Err(e) => return Err(e),
        };
        for entry in entries {
            let name = entry?.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".md") {
                inputs.push(format!("{prefix}{name}"));
            }
        }
    }
    inputs.sort();

    let mut h = Sha256::new();
    for rel in &inputs {
        // A listed file vanishing mid-walk degrades to absence — the
        // next key sees whichever state settles.
        let Ok(bytes) = std::fs::read(mw.join(rel)) else {
            continue;
        };
        h.update((rel.len() as u64).to_le_bytes());
        h.update(rel.as_bytes());
        h.update((bytes.len() as u64).to_le_bytes());
        h.update(&bytes);
    }
    let mut hex = String::with_capacity(64);
    for b in h.finalize() {
        let _ = write!(hex, "{b:02x}");
    }
    Ok(hex)
}
