---
id: az-d0n3
title: Arrow seam extraction
status: done
category: engine/exec
labels: [refactor]
verify: "cargo test -p alpha-exec seam::"
created: 2026-07-30
---
Extract the Arrow conversion seam so spill tests can fake batches.

## log
- 2026-07-30 open→doing
- 2026-08-01 doing→done — verify exit 0
