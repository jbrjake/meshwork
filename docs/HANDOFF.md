# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 0.7 (2026-08-04). `close`: runs `verify:` via `sh -c` from the repo root (e2e proves it from a subdir), records `verify exit N` + date in the log on success AND failure, closes on exit 0 only; failed attempts leave the status untouched; a task with no `verify:` refuses to close without `--waive "<reason>"`; waive writes frontmatter `waived:` (SQL-checked via `WHERE waived IS NOT NULL` in the test) + log line. TRACE: MW-E2 → done (E1/E3 already).

**Decisions:** close allowed from open|doing|blocked; done/dropped refuse. Verify output is captured and re-emitted (stdout→stdout, stderr→stderr) in text mode, suppressed in --json mode (envelope stays clean).

**Open threads:** MW-J4 planned until --bless (0.8); fixtures.rs 510 (warn).

**Next concrete step:** PLAN 0.8 — `ready` (normative SQL: needs clause + container clause) + `q` + `--json` everywhere, stable versioned shape (B6, C1, C3, D1, D2). Golden machinery + `--bless` arrives here (ready-alpha.json), plus `e2e::caps_and_more_marker` and `e2e::json_stable_shapes`.
verify: `cargo test e2e::ready_golden e2e::json_stable_shapes` exits 0.
