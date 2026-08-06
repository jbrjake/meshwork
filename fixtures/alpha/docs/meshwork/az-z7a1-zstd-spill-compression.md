---
id: az-z7a1
title: Zstd spill compression
status: open
category: engine/spill/compression
labels: [perf, p2]
needs: [az-q2r4]
verify: "cargo test -p alpha-spill zstd::"
created: 2026-08-03
---
Only worth measuring once the bench harness exists.
