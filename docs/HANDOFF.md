# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 0.8 (2026-08-04). `ready` runs DESIGN §5's normative SQL (DataFusion handles the correlated NOT EXISTS pair fine); LIMIT moved to render time because the `… and N more (use --all)` marker needs the true total. `q` = raw SQL, text table + typed JSON cells ({"columns","rows"}). Golden machinery live: `common::assert_golden` byte-compares `fixtures/golden/*`; regeneration ONLY via `MESHWORK_BLESS=1 cargo test` (implements the plan's `--bless`) + reviewed diff. `ready-alpha.json` committed (13 rows, hand-verified against corpus semantics). TRACE: B6, C1, C3, D1, D2, J4 → done (37 planned left).

**Decisions:** e2e tests live in include!'d part-files (e2e_lifecycle.rs, e2e_query.rs) so test paths stay flat `e2e::<name>` (TRACE/gate grep full paths) under the file caps. Note: cargo fmt does NOT format include! part-files — keep them tidy by hand. Single-repo `ready` still treats cross-repo needs as unresolved-blocking; PLAN 2.3 adds registry lookup and will re-bless ready-alpha.json (expected diff: az-x9b2 becomes ready).

**Open threads:** none new.

**Next concrete step:** PLAN 0.9 — `lint`: schema, needs/parent cycles, cross-repo parent, blocked-without-reason, duplicate IDs, duplicate frontmatter keys, dangling edges, missing verify (warn); `--fix`: re-slug fewer-inbound side + rewrite same-repo refs, repair duplicate keys (A4, A6, B2, B3, I2).
verify: `cargo test e2e::lint_broken_corpus` exits 0.
