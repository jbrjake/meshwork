---
id: mw-a8tv
title: "prime grows handoff sections: next-task detail + last-N done; retire hand-written HANDOFF.md"
status: open
category: product/prime
verify: cargo test e2e::prime_handoff_sections
docs:
  - DESIGN-meshwork.md#§-7-session-integration   # MW-D3 budget
  - REQUIREMENTS-meshwork.md#§-d-context-discipline   # MW-D3, MW-D5
created: 2026-08-06
---
Owner ruling 2026-08-06: HANDOFF must not be hand-written — it duplicates
graph state (the 08-06 session hand-edited docs/HANDOFF.md twice restating
what tasks already record). Materialize it: prime grows two sections inside
the same 6KB budget (MW-D3) — (1) detail block for the top ready task
(category, needs, verify, docs: refs, body head); (2) "recently done": last
~5 closed tasks (id, title, done-date from log lines), newest first. One
view, not two: a separate handoff verb would split session-start reading and
§6 is frozen (new verb = owner §6 edit; richer prime output is not a surface
change — re-bless the prime golden via --bless).
Lands in the same commit: delete docs/HANDOFF.md; drop CLAUDE.md ritual step
4's HANDOFF clause and baseline override 2 (HANDOFF byte budget); update the
adoption skill step 4 (pilot repos delete HANDOFF.md, not trim). Blocks
mw-ntt5 — the pilot should install the post-HANDOFF world.

## log
- 2026-08-06 created
