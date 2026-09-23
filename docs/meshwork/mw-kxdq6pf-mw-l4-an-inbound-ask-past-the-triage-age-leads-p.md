---
id: mw-kxdq6pf
title: MW-L4 — an inbound ask past the triage age leads prime's next block (a ruled MUST with no task; the mechanism behind 56 asks closed with no answer)
status: open
category: core/render
verify: run cargo test prime_leads_with_a_stale_ask
docs:
  - docs/REQUIREMENTS-meshwork.md
created: 2026-09-23T15:13Z
---

MW-L4 is a MUST in `docs/REQUIREMENTS-meshwork.md`: "An inbound ask past the triage age leads
`prime`'s next block, with the local `next →` rendered below it. The order is computed per render;
no state records that an ask has led, and no ask is suppressed by having led." It was ruled in on
2026-09-20 as the field study's Tier 1 #1 and no task carries it (`portfolio search "MW-L4"` → no
matches).

`prime` leads with the local queue's head today (`src/cli/prime.rs`: `next_task` is `ready[0]`), and
the inbox renders beside it, never ahead of it. That is the mechanism behind the 2026-09-21 count
(`portfolio/sequence.md` fact 4): 56 of 256 cross-repo asks closed or dropped with no answer ever
recorded.

Work: when an inbound ask is older than the triage age (the threshold `pulse`'s `past_triage`
already uses), prime's next block leads with it: the asking repo, the id, its age, one line of its
title. The local `next →` renders below it. The order is computed per render and no state is written.
The test pins both orders: a stale ask leads, and a fresh ask does not.

## log
- 2026-09-23T15:13Z created
