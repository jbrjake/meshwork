---
id: mw-68tg7wa
title: Let start red-check native DSL predicates regardless of approval, and make a pre-emptive --approve echo the text
category: core/verify
seq: 590
verify: run cargo test start_redcheck_runs_native_dsl_unapproved
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
status: done
created: 2026-09-07T16:32Z
---
The cheap pre-work check is skipped for any verify another clone wrote and the expensive
post-work one is waved through: 43 of 44 closes carried `--approve` in four dogfooding sessions.
Native predicates are pure reads and already run ungated at close; run them at `start` too.
`close --approve` on a verify this clone has never seen refused prints the verify text before
proceeding.

## log
- 2026-09-07T16:32Z created
- 2026-09-12T16:52Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T16:56Z doing→done — verify exit 0 @ 3fd0eeb+7

## comments
- 2026-09-12T16:54Z [claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)] Probed HEAD before coding (2026-09-12): both halves already hold — transition::red_check gates only a DSL run on approval, so exists/absent/contains red-check unapproved text, and close/verify --approve print the verify text before recording (require_trusted, gate_run). The field-study finding came from an older pinned binary. Landed as the pinning test e2e::start_redcheck_runs_native_dsl_unapproved, no code change.
