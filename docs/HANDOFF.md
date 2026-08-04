# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 0.2 (2026-08-04). Parser (0.1) strict + corpus-pinned. IDs (0.2): `src/id.rs` — splitmix64 in-crate (pinned dep posture has no `rand`), Crockford-lowercase 4-char suffix (`ALPHABET` exported), `IdGen::{with_seed,from_seed_str,from_entropy}` (entropy mixes a global counter so in-process gens always differ), `mint_unique` re-rolls against `<id>-*.md`/`<id>.md` in the tasks dir, errors loudly after 4096 colliding draws.

**Decisions:** `MESHWORK_ID_SEED` is the seed hook the binary will wire via `IdGen::from_seed_str(env)` when verbs land (0.4/0.5) — e2e duplicate-ID merge (0.10) forces collisions with it. Unit tests stay in the suite for TRACE-exact test paths.

**Open threads:** MW-J4 planned until --bless (0.8); fixtures.rs 510 lines (warn). MW-A4 stays planned until `e2e::merge_duplicate_id` (0.10).

**Next concrete step:** PLAN 0.3 — ingestion → Arrow `MemTable`s → DataFusion `SessionContext`: five tables incl. `waived`, `ord`, `resolved`, child→parent edge direction (DESIGN §3–4; C1).
verify: `cargo test tables::` exits 0.
