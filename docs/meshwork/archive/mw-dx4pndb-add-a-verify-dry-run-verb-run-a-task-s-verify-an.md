---
id: mw-dx4pndb
title: "Add a verify dry-run verb: run a task's verify and report, close nothing"
status: done
category: core/verify
verify: ./meshwork --help | grep -q '^  verify '
relates:
  - mw-175bn4c
  - mw-yj2fq9x
docs:
  - docs/DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-12T20:48Z
seq: 200
blocked-reason:
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
- 2026-08-23T18:56Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-23T19:05Z doing→done — verify exit 0 @ d6badc3+9

## comments
- 2026-08-22T01:43Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-22: APPROVED, single-task only. verify <id> runs the task's verify under close's exact §12b gate routing, reports exit status, closes nothing, releases nothing. No --all-open sweep — that waits for its own evidence and its own ruling.
