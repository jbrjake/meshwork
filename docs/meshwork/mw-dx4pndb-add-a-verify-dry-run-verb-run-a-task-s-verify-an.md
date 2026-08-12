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
