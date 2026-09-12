---
id: mw-axhze9m
title: Lint findings the views make cheap — handoff-cites-closed, implicit-edge, discovered-cycle, seq-collision, close-attempts
category: core/hygiene
needs: [mw-p2q2fxd]
seq: 640
verify: 'all(run cargo test lint::handoff_cites_closed, run cargo test lint::seq_collision)'
docs:
  - docs/PROPOSAL-analytics.md#§-5-4-lint
status: done
created: 2026-09-07T16:32Z
---
Each one query over `mentions`, `lineage`, `graph`, `facts`: 85 live handoffs naming a closed
task; 107 live→live mentions with no edge (report only; the tokenizer falsifier is in the
proposal's §13); `spawn_depth` at the bound; two live tasks on one `seq` in one repo (sazed 113);
≥ 2 failed closes on a live task. Warnings all. The lint session registers the views. MW-S9.

## log
- 2026-09-07T16:32Z created
- 2026-09-12T17:02Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T17:06Z doing→done — verify exit 0 @ be51f78+10
