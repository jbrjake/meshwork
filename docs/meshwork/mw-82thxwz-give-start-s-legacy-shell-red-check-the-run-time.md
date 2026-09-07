---
id: mw-82thxwz
title: Give start's legacy-shell red-check the run timeout and a one-line notice
category: core/verify
relates: ["leras#le-3m0gpb1"]
seq: 200
verify: run cargo test start_redcheck_shell_timeout
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
status: open
created: 2026-09-07T16:32Z
---
`red_check` in `src/cli/transition.rs` runs an approved shell verify via `sh -c` with no timeout
and discards output; an unscoped `cargo test` compiles for minutes in silence and the agent kills
it and hand-flips `status: doing`. Route the shell path through `verify_exec::run_argv`'s timeout
(`RUN_TIMEOUT`), print `note: red-checking verify — may build` before it starts, and report a
timeout as a warning that still transitions. The leras hang report is answered from here.

## log
- 2026-09-07T16:32Z created
