---
id: mw-s3905fv
title: "add accepts a body — the description has no CLI path today"
category: core/lifecycle
relates: [mw-rz4ey2h, mw-t01ek6s]
verify: run cargo test e2e::add_body
seq: 300
docs:
  - docs/DESIGN-meshwork.md#§-6-cli-surface
status: done
created: 2026-08-08T16:42Z
blocked-reason:
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
- 2026-08-20T11:43Z open→blocked — surface delta awaiting its own owner ruling: which body path (stdin when piped / --body @file / batch-of-one idiom). The 2026-08-10 ruling covered mw-rz4ey2h+mw-f1x71yg only; fafd694 is the precedent for landing without the nod
- 2026-08-21T21:12Z blocked→open
- 2026-08-23T19:33Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-23T19:37Z doing→done — verify exit 0 @ 1384a8a+6

## comments
- 2026-08-21T21:12Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-21: body lands like handoff and comment — add --body "text"|@file|- per the mw-rz4ey2h spellings. Surface delta sanctioned; DESIGN §6 add row grows the flag with the implementation.
