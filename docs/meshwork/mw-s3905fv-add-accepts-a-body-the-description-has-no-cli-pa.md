---
id: mw-s3905fv
title: "add accepts a body — the description has no CLI path today"
category: core/lifecycle
relates: [mw-rz4ey2h, mw-t01ek6s]
verify: run cargo test e2e::add_body
seq: 300
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
status: open
created: 2026-08-08T16:42Z
---
Field evidence (sazed): every substantive task filed post-migration went
`add`, then `cat >> <file>` or a python heredoc to attach the
description — there is no CLI path to a body at creation. Batch
documents carry bodies; a single add cannot. The `cat >>` route is also
the damage source the stray-prose lint task (filed alongside this one)
repairs. Options bounded by the §6 ruling: body from stdin when piped,
`--body @file`, or a documented batch-of-one idiom. Sibling of
mw-rz4ey2h (@file/stdin for --handoff/--comment) — rule on them
together.

## log
- 2026-08-08T16:42Z created
