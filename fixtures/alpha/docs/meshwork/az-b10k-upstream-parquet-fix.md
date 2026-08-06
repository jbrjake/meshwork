---
id: az-b10k
title: Upstream fix for parquet reader
status: blocked
category: engine/exec
needs: [az-d0n3]
verify: "cargo test -p alpha-exec parquet::"
created: 2026-08-01
blocked-reason: needs datafusion 52 release; unblock = bump dep and rerun parquet suite
---
Blocked on an upstream release; nothing local to do.

## log
- 2026-08-02 open→blocked — upstream release pending
