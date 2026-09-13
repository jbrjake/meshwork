---
id: mw-vffwacx
title: Rule on the session ritual — write flags as local writes, asks visible until the answer is terminal, the full inbox, ask parity with next, outbound asks out of ready, the skill with prime
category: meta/ruling
labels: [owner]
seq: 380
verify: contains docs/PLAN-proposals-implementation.md /R-B RULED 20[0-9]{2}-[0-9]{2}-[0-9]{2}/
docs:
  - docs/PLAN-proposals-implementation.md#§-3-rulings
  - docs/FIELD-STUDY-session-transcripts.md#§-2-1-the-inbox
  - docs/DESIGN-meshwork.md#§-15-decisions
status: open
created: 2026-09-07T16:32Z
handoff: |
  Every ready task is now owner-gated: this ruling (R-B) opens mw-pcjm4pb,
  mw-21qzw9n, mw-vwdm3ed, mw-zhdypdz and mw-1cmpywn; mw-t41d6ze (R-E)
  opens
  the DSL trio mw-ps4fzn2/mw-rwb77wp/mw-qhmek05, which also answers the
  two
  inbound asks (le-yppwtpq, marasi#ma-tpmcdnk — the latter now linked
  from
  mw-ps4fzn2). Read before ruling: the field study's §3 "The split,
  priced"
  table and §4 — the numbers keep inbox > write flags > DSL and demote
  the
  skill hook (flat per session), so mw-1cmpywn sits at seq 560 behind the
  DSL trio. Landed since the batch: needs-behind lint, the skill text
  (SKILL.md at 6.9KB of 8KB, four rules up top, inbox section around the
  asks view), and archive bundles (format 2, src/archive.rs; FORMAT.md
  §Bundles). This store now trips `archive-loose` (178 loose archived
  files): `lint --fix` would bundle them and make the store format 2 —
  sibling repos pinned at v0.4.0 would then refuse it in portfolio reads
  until a release carries format 2 and they re-pin, so that step is yours.
  When a ruling lands, write its `R-B RULED YYYY-MM-DD` marker in
  PLAN-proposals-implementation.md (the ruling tasks' verifies grep for
  it),
  then `ready` shows the unblocked wave.
---
Owner-gated. Six items, recommendations in the plan's R-B table; B.1 and B.2 amend DESIGN
§15.12's reading, B.4 is the product-intent conflict the study names. Every eruption in field
study §2.1–§2.3 traces to one of these.

## log
- 2026-09-07T16:32Z created
- 2026-09-13T14:13Z handoff by claude (95c1ceaa-ef91-4ac1-90f8-d8922fd3bdd4)
