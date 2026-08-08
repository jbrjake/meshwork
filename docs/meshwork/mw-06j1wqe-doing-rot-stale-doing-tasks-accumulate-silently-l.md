---
id: mw-06j1wqe
title: "Doing-rot: stale doing tasks accumulate silently — lint + prime pressure"
category: core/lifecycle
relates: [mw-dkwf26w]
verify: cargo test lint::stale_doing_warn
seq: 320
docs:
  - DESIGN-meshwork.md#§-7-session-integration
status: open
created: 2026-08-08T16:42Z
---
Field evidence (sazed, 2026-08-08): 10 of 160 tasks sit in `doing`,
8 with no claimant, and at least one is finished in fact but never
closed (its title literally says "DOOR ADOPTED 08-05"). Nothing pushes
back: prime lists in-progress but the list only grows; ready ignores
doing entirely. Wanted: lint warns on a doing task whose newest log line
is older than N days, and on unclaimed doing; decide at implementation
whether prime annotates staleness on the in-progress section. Sister of
mw-dkwf26w (no-verify warning must cover doing) — same blind spot,
different signal. Rule on the pair together.

## log
- 2026-08-08T16:42Z created
