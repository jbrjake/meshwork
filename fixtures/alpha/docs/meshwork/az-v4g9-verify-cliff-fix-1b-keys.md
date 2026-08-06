---
id: az-v4g9
title: Verify cliff fix on 1B keys
status: open
category: engine/spill
labels: [perf, p0]
needs: [az-t5k1, az-d0w1]
verify: cargo bench -p alpha-spill cliff_1b
created: 2026-08-03
---
Two unmet deps; exercises multi-target needs lists.
