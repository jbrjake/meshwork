---
id: mw-3qvv4ar
title: Opt-in inheritance as placement, the placed-by line in prime's next block
category: capability/rank
needs: [mw-ex5x0y2]
seq: 610
verify: run cargo test ready_inherit_opt_in
docs:
  - docs/PROPOSAL-prioritization.md#§-5-requirements
status: open
created: 2026-09-07T16:32Z
---
`[bands] inherit = true` lets `inherit(t)` replace `place(t)` in the key when better, with
`placed_by = inherit:<dependent>`; off by default because it would have un-parked the mirror
against a ruling. `prime`'s next block prints `placed by: seq 180` / `band core/integrity=10 —
"<reason>"` / `floor (unlocks 3)`, sanitized like every text seam. MW-R6c, the prime half of
MW-R3.

## log
- 2026-09-07T16:32Z created
