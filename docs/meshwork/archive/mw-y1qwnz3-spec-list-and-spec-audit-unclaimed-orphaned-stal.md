---
id: mw-y1qwnz3
title: spec list and spec audit — unclaimed, orphaned, stale, re-open candidates, dangling; portfolio variant
category: capability/spec
needs: [mw-jvyerng]
seq: 760
verify: run cargo test e2e::spec_audit
docs:
  - docs/PROPOSAL-spec-traceability.md#§-c-coverage-and-rulings
status: done
created: 2026-09-07T16:32Z
---
With R-F item F.3, after `spec-drift` has fired on a real task at least once: `spec list <doc>`
enumerates a doc's anchored sections with hashes; `spec audit <doc>` answers the five questions
in one report; `portfolio spec audit` unions it. MW-J5's traceability matrix as a query.
`cli_surface_frozen` re-blessed. MW-T3/T8.

## log
- 2026-09-07T16:32Z created
- 2026-09-22T14:23Z open→doing — claimed by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
- 2026-09-22T14:29Z doing→done — verify exit 0 @ e49dcc7+5
