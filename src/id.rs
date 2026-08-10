//! ID generation (PLAN 0.2, MW-A4; length 4→7 per mw-1b09): `<alias>-<7-char
//! base32>` slugs, Crockford-lowercase alphabet (no i/l/o/u), 32^7 ≈ 34.4B
//! combinations. Length applies to minting only — parsing never validates
//! it, so pre-mw-1b09 4-char IDs remain legal and stores mix lengths freely.
//!
//! Collision policy: creation collision-checks against local files and
//! re-rolls. Parallel clones share no state and CAN mint the same ID —
//! that's `lint`'s remedy (post-merge duplicate detection), not ours.
//! Deterministic seeding exists so the e2e duplicate-ID scenario can force
//! the collision on purpose.

use std::path::Path;

/// Crockford base32, lowercase: digits + letters minus i, l, o, u.
pub const ALPHABET: &str = "0123456789abcdefghjkmnpqrstvwxyz";

/// Attempts before `mint_unique` declares the local space exhausted.
const MAX_ATTEMPTS: u32 = 4096;

/// Small deterministic RNG (splitmix64 core) — the pinned dep posture
/// (MW-J1) has no `rand`, and reproducibility matters more than randomness
/// quality for 35-bit draws.
#[derive(Debug, Clone)]
pub struct IdGen {
    state: u64,
}

impl IdGen {
    /// Deterministic generator — same seed, same ID sequence.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Seed from a decimal string when given (the `MESHWORK_ID_SEED` hook
    /// the binary wires through); anything else falls back to entropy.
    #[must_use]
    pub fn from_seed_str(seed: Option<&str>) -> Self {
        match seed.and_then(|s| s.trim().parse::<u64>().ok()) {
            Some(seed) => Self::with_seed(seed),
            None => Self::from_entropy(),
        }
    }

    /// Non-deterministic generator: wall-clock nanos, PID, and a global
    /// counter — the counter guarantees two in-process generators differ.
    #[must_use]
    pub fn from_entropy() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| {
                u64::try_from(d.as_nanos() & u128::from(u64::MAX)).unwrap_or(0)
            });
        let mix = COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::with_seed(nanos ^ (u64::from(std::process::id()) << 32) ^ (mix << 1))
    }

    fn next_u64(&mut self) -> u64 {
        // splitmix64 (public domain, Vigna).
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Mint the next `<alias>-<7 chars>` ID. No collision check — see
    /// [`mint_unique`].
    #[must_use]
    pub fn next_id(&mut self, alias: &str) -> String {
        let bytes = ALPHABET.as_bytes();
        let mut draw = self.next_u64();
        let mut suffix = String::with_capacity(7);
        for _ in 0..7 {
            let idx = usize::try_from(draw & 31).unwrap_or(0);
            suffix.push(bytes[idx] as char);
            draw >>= 5;
        }
        format!("{alias}-{suffix}")
    }
}

/// Filename slug from a title: lowercase alphanumeric runs joined by `-`,
/// capped at 48 chars. Cosmetic and never load-bearing (DESIGN §2).
#[must_use]
pub fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut pending_dash = false;
    for c in title.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            pending_dash = false;
            slug.push(c.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
        if slug.len() >= 48 {
            break;
        }
    }
    if slug.is_empty() {
        "task".to_string()
    } else {
        slug
    }
}

/// The alias charset contract (mw-a6jdf5s): `[a-z0-9]+`, nothing else. ID
/// recovery from an invalid file takes the first two dash-segments of the
/// stem, so a dashed alias (`my-repo`) silently mis-recovers every id.
#[must_use]
pub fn valid_alias(alias: &str) -> bool {
    !alias.is_empty()
        && alias
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// Mint an ID that no local task file already uses (MW-A4): re-roll while
/// `<dir>/<id>-*.md` (or `<id>.md`) exists. A missing dir collides with
/// nothing.
///
/// # Errors
/// After 4096 colliding draws, errors out loudly ("space exhausted") rather
/// than spinning — at that density something is deeply wrong with the store.
pub fn mint_unique(alias: &str, tasks_dir: &Path, gen: &mut IdGen) -> std::io::Result<String> {
    for _ in 0..MAX_ATTEMPTS {
        let id = gen.next_id(alias);
        if !id_taken(&id, tasks_dir)? {
            return Ok(id);
        }
    }
    Err(std::io::Error::other(format!(
        "id space exhausted for alias `{alias}` after {MAX_ATTEMPTS} attempts in {}",
        tasks_dir.display()
    )))
}

fn id_taken(id: &str, tasks_dir: &Path) -> std::io::Result<bool> {
    // Archived ids count as taken — never reused (MW-A4, mw-45e2qf4).
    Ok(id_in_dir(id, tasks_dir)? || id_in_dir(id, &tasks_dir.join("archive"))?)
}

fn id_in_dir(id: &str, dir: &Path) -> std::io::Result<bool> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(e),
    };
    let prefix = format!("{id}-");
    let exact = format!("{id}.md");
    for entry in entries {
        let name = entry?.file_name();
        let name = name.to_string_lossy();
        if name.starts_with(&prefix) || name == exact {
            return Ok(true);
        }
    }
    Ok(false)
}
