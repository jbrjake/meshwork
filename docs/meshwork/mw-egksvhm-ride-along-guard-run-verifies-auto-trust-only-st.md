---
id: mw-egksvhm
title: "Ride-along guard: run verifies auto-trust only store-only task provenance"
status: open
category: core/verify
parent: mw-6895bkg
verify: out=$(cargo test ride_along 2>&1) && echo "$out" | grep -qE "ok\. [1-9][0-9]* passed"
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
seq: 175
created: 2026-08-14T15:39Z
---

## log
- 2026-08-14T15:39Z created

## comments
- 2026-08-14T15:39Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Spec (owner ruling 2026-08-14, recorded on mw-6895bkg): given a task id, walk git log --follow over its file (root and archive/ paths). For each commit, judge the FULL delta of the merge that landed it — first-parent walk to find the landing merge; squash/fast-forward = the commit itself (git diff-tree covers it; for a true merge commit judge diff M^1..M so a PR split across inner commits is still seen whole). If any touched path falls outside docs/meshwork/, provenance is mixed → the task's run predicates go behind the trust gate; store-only history → run executes approval-free. Uncommitted task file = authored by this clone's operator → passes. Conservative by design: ANY commit touching the task file counts, even seq-only edits. Zero network, local refs only (MW-J6); degrade toward gating, never toward trust, when git is unreadable. Red-checked verify 2026-08-14 (observed exit 1).
