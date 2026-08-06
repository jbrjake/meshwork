---
id: bz-s3q1
title: Schema qualifier cleanup
status: open
category: reader/schema
verify: "cargo test -p beta-reader qualifier::"
seq: 10
created: 2026-08-02
---
Strip redundant qualifiers before the planner sees them.
