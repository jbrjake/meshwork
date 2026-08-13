---
id: mw-06j1wqe
title: "Doing-rot: stale doing tasks accumulate silently — lint + prime pressure"
category: core/lifecycle
relates: [mw-dkwf26w]
verify: out=$(cargo test lint::stale_doing_warn 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
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

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Doing-rot is now measured, not hypothetical. sazed: 7 tasks imported as doing on 08-07 were still doing on 08-12 across all 34 sessions; the doing count never dropped below 8 (peak 12), eating ~8 lines of every 6KB prime and crowding also-ready down to 9 of 126 — in one prime the owner's seq-10 flagship was absent entirely, and the owner opened three separate sessions re-shouting priorities the store already carried (35e38bed, 1dc9fa1f, fc237a1a). leras: le-s3k2v7b imported [~] as doing with no claimant and sat in weather for all 5 sessions. Age display plus prime demotion would have surfaced every one of these.
