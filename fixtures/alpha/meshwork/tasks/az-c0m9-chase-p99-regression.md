---
id: az-c0m9
title: Chase p99 regression
status: doing
category: engine/spill
labels: [perf, p0]
verify: cargo bench -p alpha-spill p99
created: 2026-08-02
---
p99 degrades only with governed spill on.

## log
- 2026-08-02 open→doing

## comments
- 2026-08-02 [jon] p99 only degrades with governed-spill on; see the bench notes
- 2026-08-03 [claude/f10a7561] bisected to batch=64k; the cliff tracks the
  governor wakeup interval, not the batch size itself — repro attached
  to az-a7t2 as spill-p99-excerpt.log
- 2026-08-03 [jon] nice; keep batch at 64k and tune wakeup instead
- 2026-08-04 [claude/b2277c19] wakeup=250ms flattens the curve on 3 of 3 runs
