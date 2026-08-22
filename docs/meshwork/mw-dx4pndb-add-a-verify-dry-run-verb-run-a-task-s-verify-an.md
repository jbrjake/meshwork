---
id: mw-dx4pndb
title: "Add a verify dry-run verb: run a task's verify and report, close nothing"
status: open
category: core/verify
verify: ./meshwork --help | grep -q '^  verify '
relates:
  - mw-175bn4c
  - mw-yj2fq9x
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-12T20:48Z
seq: 200
blocked-reason:
handoff: |
  APPROVED 2026-08-22 (single-task only, no --all-open) — not yet
  worked, cleared for the next session. Implementation sketch: new verb
  verify <id> reporting the verify verdict, closing nothing, releasing
  nothing; reuse close.rs's exact §12b routing (classify →
  gate_run/require_trusted → verify_exec/run_shell) — extract the
  shared routing rather than duplicating it, close.rs run() is already
  near the too-many-lines lint ceiling. DESIGN §6 gains the verb row;
  check e2e::cli_surface_frozen. Verify is the --help grep for '^ verify
  '.
---
Red-checking verifies is now a proven ritual with no supported path:
leras sessions hand-rolled the same extraction loop six times
(`v=$(grep -m1 '^verify:' docs/meshwork/$id-*.md | cut -c9-); sh -c "$v"`)
across ~55 task-verify executions, and one task's verify IS that loop
over six other tasks (le-b39mm50). The hand-rolled form runs in the
*interactive* shell — the exact authoring-shell mismatch that let 28
rg-verifies ship fail-closed. A `meshwork verify <id> [--all-open]`
that runs the verify under the same `sh -c` close uses, reports exit
status, and closes nothing would make red-first authoring and
rot-sweeps one verb. Surface change — needs the DESIGN §6 owner
ruling; mw-175bn4c (red-at-add) covers authoring time, this covers
every moment after.

## log
- 2026-08-12T20:48Z created
- 2026-08-22T01:22Z open→blocked — awaiting DESIGN §6 owner ruling — a new verify verb is a frozen-surface change; evidence for it is in the task body
- 2026-08-22T01:43Z blocked→open

## comments
- 2026-08-22T01:43Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-22: APPROVED, single-task only. verify <id> runs the task's verify under close's exact §12b gate routing, reports exit status, closes nothing, releases nothing. No --all-open sweep — that waits for its own evidence and its own ruling.
