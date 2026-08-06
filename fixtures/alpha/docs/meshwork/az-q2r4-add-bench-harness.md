---
id: az-q2r4
title: Add bench harness for spill
status: open
category: tools/bench
labels: [perf]
verify: cargo bench -p alpha-spill -- --list
created: 2026-08-01
---
Shared harness for all spill benches; N≥7 reps, median.
