---
id: az-x9b2
title: Cross-repo consumer bump
status: open
category: engine/exec
needs: [beta#bz-c0r3]
verify: cargo build -p alpha-exec
seq: 40
created: 2026-08-03
---
Depends on beta's core reader v2 (done there) — resolves via the registry
even from single-repo commands (MW-B3).
