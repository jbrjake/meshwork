---
id: az-cw55
title: Cache warmup pass before bench
status: open
category: tools/bench
labels: [perf]
needs: [az-q2r4]
verify: cargo bench -p alpha-spill -- warmup
seq: 30
created: 2026-08-04
---
Benches lie without warmup; make the harness do it.
