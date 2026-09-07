---
id: mw-68tg7wa
title: Let start red-check native DSL predicates regardless of approval, and make a pre-emptive --approve echo the text
category: core/verify
seq: 590
verify: run cargo test start_redcheck_runs_native_dsl_unapproved
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
status: open
created: 2026-09-07T16:32Z
---
The cheap pre-work check is skipped for any verify another clone wrote and the expensive
post-work one is waved through: 43 of 44 closes carried `--approve` in four dogfooding sessions.
Native predicates are pure reads and already run ungated at close; run them at `start` too.
`close --approve` on a verify this clone has never seen refused prints the verify text before
proceeding.

## log
- 2026-09-07T16:32Z created
