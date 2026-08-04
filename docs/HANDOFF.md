# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** B0–B4 + 0.1 (2026-08-04). Crate is now lib + thin bin. `src/parse.rs`: strict serde frontmatter (Option<Vec> tolerates hand-edited empty keys), `## log`/`## comments` tail sections with two-space continuations, unknown keys warn (MW-A6), filename/id mismatch warns, textual dup-key scan runs *before* YAML so union-poison gets a precise diagnosis (MW-I1/I2), any hard failure → `ParsedTask::Invalid` with filename-recovered ID. `parse::corpus_parses_as_planted` pins parser↔corpus: alpha+beta all valid & warning-free, alpha-broken → exactly {ax-brk9, ax-un10} invalid. TRACE: MW-A6, MW-F1 → done.

**Decisions:** unit-tier tests live in the suite (tests/suite/<mod>.rs) so test paths match TRACE names exactly (`parse::x`, not `parse::tests::x` — gate §6 greps paths). DESIGN §2's normative example is embedded verbatim as the `roundtrip_hand_edited` input.

**Open threads:** MW-J4 planned until --bless lands (0.8); fixtures.rs 510 lines (warn).

**Next concrete step:** PLAN 0.2 — ID generation: `<alias>-<4-char base32>`, collision re-roll against local files; seedable RNG hook for tests (A4).
verify: `cargo test id::` exits 0.
