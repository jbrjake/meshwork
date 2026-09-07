---
id: mw-9x1g9r1
title: Add lineage and cites lines to show, from the views
category: core/render
needs: [mw-p2q2fxd]
seq: 660
verify: run cargo test show_lineage_line
docs:
  - docs/PROPOSAL-analytics.md#§-5-5-show
  - docs/PROPOSAL-analytics.md#§-4-8-lineage
status: open
created: 2026-09-07T16:32Z
---
After `commits:`: `lineage: spawned 25 (1 live, depth 4) · children 0 · mentioned by 7` and
`cites: 8 closed (…)`, both capped, both omitted when zero. `show` has no perf gate; read the
views. MW-S10's second half.

## log
- 2026-09-07T16:32Z created
