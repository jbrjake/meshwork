# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** **M0 complete** (2026-08-04, one session): parser, IDs, five-table SQL contract, init/add/show/transitions/close, ready (normative SQL) + q, lint + --fix, merge scenarios 1–3, offline-everything (PATH = git+sh only), cache-delete-safe. Gate green throughout: 68 tests, coverage ≥80%, goldens ready-alpha.json + lint-broken.json committed via the bless flow. TRACE: 27 planned remain, all mapped to M1+ tests; B1/K1 flip with dep_edit (1.1) / comment_attach (1.4).

**Decisions:** all in DESIGN §15 + per-commit notes. Session-critical ones: MESHWORK_BLESS=1 implements --bless; MESHWORK_ID_SEED/MESHWORK_TODAY are the determinism hooks; e2e part-files via include! keep flat `e2e::` test paths; unit tests live in the suite for TRACE-exact names.

**Open threads:** M0 stop-line reached — sazed pilot is *usable* but the plan continues top-to-bottom; pilot itself is 1.9 (manual, needs owner).

**Next concrete step:** PLAN 1.1 — `dep add`/`dep rm` (B1): edge edits without opening the file.
verify: `cargo test e2e::dep_edit` exits 0.
