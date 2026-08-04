# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 0.6 (2026-08-04). `src/edit.rs`: `set_scalar` (replace/insert one frontmatter line, everything else byte-preserved) + `append_section_entry` (append at end of `## log`/`## comments`, creating log before comments). Transitions start/block/drop/reopen: legal moves exactly per DESIGN §6 (reopen: blocked|doing|done→open; dropped terminal), block's `--reason` clap-required, reopen clears blocked-reason to the bare key (matches normative example), every transition appends one dated `from→to` log entry. e2e::transitions asserts the one-line-diff shape directly. TRACE: MW-E1, MW-E3 → done.

**Decisions:** transitions refuse invalid files (can't know current status) with a pointer to lint --fix. `drop` allowed from open|doing|blocked only; `done` is not droppable.

**Open threads:** MW-J4 planned until --bless (0.8); fixtures.rs 510 (warn).

**Next concrete step:** PLAN 0.7 — `close`: runs `verify:` via `sh -c` from repo root, records exit+date in log, closes on 0 only; `--waive "reason"` writes `waived` (E2).
verify: `cargo test e2e::close_gating` exits 0.
