//! `id::` — ID generation (PLAN 0.2, MW-A4; length 4→7 per mw-1b09):
//! `<alias>-<7-char base32>`, Crockford-lowercase alphabet, collision
//! re-roll against local files, seedable for tests.

use meshwork::id::{mint_unique, IdGen, ALPHABET};

fn assert_legal(id: &str, alias: &str) {
    let suffix = id
        .strip_prefix(&format!("{alias}-"))
        .unwrap_or_else(|| panic!("id `{id}` lacks `{alias}-` prefix"));
    assert_eq!(suffix.len(), 7, "7-char random component: {id}");
    for c in suffix.chars() {
        assert!(
            ALPHABET.contains(c),
            "illegal char `{c}` in `{id}` (alphabet excludes i/l/o/u)"
        );
    }
}

#[test]
fn format_alias_base32() {
    let mut gen = IdGen::with_seed(42);
    for _ in 0..500 {
        assert_legal(&gen.next_id("az"), "az");
    }
}

#[test]
fn seed_reproducible() {
    let a: Vec<String> = {
        let mut g = IdGen::with_seed(7);
        (0..10).map(|_| g.next_id("sz")).collect()
    };
    let b: Vec<String> = {
        let mut g = IdGen::with_seed(7);
        (0..10).map(|_| g.next_id("sz")).collect()
    };
    assert_eq!(
        a, b,
        "same seed, same sequence — the e2e duplicate-ID merge scenario depends on this"
    );
}

#[test]
fn seed_str_hook() {
    // The binary wires MESHWORK_ID_SEED through this; tests pass it directly.
    let mut a = IdGen::from_seed_str(Some("7"));
    let mut b = IdGen::with_seed(7);
    assert_eq!(a.next_id("az"), b.next_id("az"));
    // Unparseable or absent seed falls back to entropy — still legal IDs.
    assert_legal(&IdGen::from_seed_str(None).next_id("az"), "az");
    assert_legal(
        &IdGen::from_seed_str(Some("not-a-number")).next_id("az"),
        "az",
    );
}

#[test]
fn entropy_gens_differ_in_process() {
    // A global counter feeds entropy seeding, so two gens minted in the same
    // process can never mirror each other.
    let mut a = IdGen::from_entropy();
    let mut b = IdGen::from_entropy();
    let sa: Vec<String> = (0..4).map(|_| a.next_id("az")).collect();
    let sb: Vec<String> = (0..4).map(|_| b.next_id("az")).collect();
    assert_ne!(sa, sb);
}

/// MW-A4: creation collision-checks against local files and re-rolls.
#[test]
fn collision_reroll() {
    let dir = tempfile::tempdir().unwrap();
    let (first, second) = {
        let mut g = IdGen::with_seed(99);
        (g.next_id("az"), g.next_id("az"))
    };
    assert_ne!(first, second);
    // Occupy the first ID the seeded generator would mint.
    std::fs::write(dir.path().join(format!("{first}-taken.md")), "x").unwrap();
    let minted = mint_unique("az", dir.path(), &mut IdGen::with_seed(99)).unwrap();
    assert_eq!(minted, second, "must re-roll past the taken ID");
}

#[test]
fn mint_without_dir_is_fine() {
    // A fresh repo may not have tasks/ yet; nothing to collide with.
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("tasks");
    let id = mint_unique("az", &missing, &mut IdGen::with_seed(1)).unwrap();
    assert_legal(&id, "az");
}

#[test]
fn exhausted_space_errors() {
    // Saturate by making every draw collide: seed a gen, pre-create files
    // for its next 4096 draws, and expect a loud error instead of a hang.
    let dir = tempfile::tempdir().unwrap();
    let mut g = IdGen::with_seed(5);
    for _ in 0..4096 {
        let id = g.next_id("az");
        let p = dir.path().join(format!("{id}-x.md"));
        std::fs::write(p, "x").unwrap();
    }
    let err = mint_unique("az", dir.path(), &mut IdGen::with_seed(5)).unwrap_err();
    assert!(err.to_string().contains("exhausted"), "err: {err}");
}
