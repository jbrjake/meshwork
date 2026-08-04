---
id: az-t5k1
title: Bisect spill batch size
status: doing
category: engine/spill
labels: [perf, p0]
parent: az-st0r
verify: "cargo test -p alpha-spill -- --exact spill::cliff_600m"
created: 2026-08-02
---
Leaf of the 5-deep chain. Bisect batch size until the cliff moves.

## log
- 2026-08-03 open→doing — repro landed, bisecting spill batch size
