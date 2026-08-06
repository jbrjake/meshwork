---
id: az-h8s3
title: Handle zero-row batches
status: open
category: engine/exec
labels: [bug, p1]
verify: "cargo test -p alpha-exec zero_row::"
created: 2026-08-02
---
Zero-row batches currently panic the governor.
