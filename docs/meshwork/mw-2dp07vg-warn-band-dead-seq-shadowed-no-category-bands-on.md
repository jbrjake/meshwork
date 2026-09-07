---
id: mw-2dp07vg
title: Warn band-dead, seq-shadowed, no-category (bands only) and sequence-stale
category: capability/rank
needs: [mw-ex5x0y2]
seq: 620
verify: run cargo test lint::band_dead
docs:
  - docs/PROPOSAL-prioritization.md#§-5-requirements
status: open
created: 2026-09-07T16:32Z
---
A rule no task matches; a `seq` equal to the band it would have received; a live task with no
category once a band table makes categories load-bearing; an overlay entry blocked while a
later one is ready, or a tranche heading whose entries have all pruned away (registry-aware
pass). Warnings all. MW-R4/R5.

## log
- 2026-09-07T16:32Z created
