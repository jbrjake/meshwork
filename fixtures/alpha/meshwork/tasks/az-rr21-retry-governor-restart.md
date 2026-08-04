---
id: az-rr21
title: Retry governor restart on wake
status: open
category: engine/spill
labels: [bug, p2]
discovered-from: az-c0m9
verify: "cargo test -p alpha-spill restart::"
created: 2026-08-04
---
Found chasing the p99 regression; separate fix, separate task.
