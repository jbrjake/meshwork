---
id: mw-3jwwh5d
title: prime stamps store provenance (HEAD + dirty count)
status: done
category: core/session
verify: cargo test e2e::prime_provenance
docs:
  - DESIGN-meshwork.md#§-7-session-integration
seq: 22
created: 2026-08-06
---
One digest line, e.g.
`store @ 3f5ff64 · 2 uncommitted task edits · 1 ahead of origin`
(git status/rev-list scoped to docs/meshwork/). An incoming session on
another machine or clone sees staleness relative to the remote instead
of discovering it mid-work — most of cross-machine coherence for the
cost of one git invocation. Degrades silently (line omitted) when git
or remote info is unavailable; counts against the 6KB cap like
everything else (MW-D5). Filed from the 2026-08-06 review.

## log
- 2026-08-06 created
- 2026-08-07T00:40Z open→doing — claimed by claude
- 2026-08-07T00:42Z doing→done — verify exit 0
