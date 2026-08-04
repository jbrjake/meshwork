# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Bootstrap B0–B2 (2026-08-04): crate + pinned deps build clean; gate (`verify_meshwork.sh`, 8 sections) runs green with §§4,7,8 in loud-SKIP pending mode; TRACE.md seeded all-planned; baseline scaffolding (smoke/regression, hooks, file-length) in place; REQUIREMENTS/DESIGN/PLAN moved into this repo from the code root (they live here now).

**Decisions:** all pre-build decisions are in DESIGN §15; dep posture mirrors sahjhan (datafusion 51, edition 2021); serde_yaml_ng replaces archived serde_yaml (REQUIREMENTS J1).

**Open threads:** none.

**Next concrete step:** PLAN item B3 — fixture corpus skeleton + `fixtures::corpus_covers_features`.
verify: `cargo test fixtures::` exits 0.
