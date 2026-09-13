---
id: mw-073zekp
title: Warn needs-behind — a prerequisite placed later than something that needs it, naming both ids and both places
category: capability/rank
needs: [mw-zwgp6x7]
seq: 570
verify: run cargo test lint::needs_behind
docs:
  - docs/PROPOSAL-prioritization.md#§-5-requirements
  - docs/PROPOSAL-prioritization.md#§-1-what-the-stores-say
status: done
created: 2026-09-07T16:32Z
---
Nine contradictions portfolio-wide today, four of them findings a human wants (this repo's parked
mirror under the v1 gate; sazed's unranked prerequisite of a seq-72 task). Report only; the
owner's fix differs per case. Fixture: a parked prerequisite under a steered dependent. MW-R6b.

## log
- 2026-09-07T16:32Z created
- 2026-09-12T17:23Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T17:24Z handoff by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-13T13:37Z doing→done — verify exit 0 @ fb7598b+1

## comments
- 2026-09-13T13:37Z [claude (95c1ceaa-ef91-4ac1-90f8-d8922fd3bdd4)] Landed: needs-behind in src/lint_views.rs over the graph helpers (g_up, g_live), one finding per prerequisite naming its best-placed live same-repo dependent and both places, unset seq rendered unranked; test lint::needs_behind (direct, transitive, unranked, in-order silent, terminal dependent silent); DESIGN §6 lint row. On this store it names the five parked mirror tasks (mw-cvw8, wm9w, vmzg, ws31, a413) behind mw-v4ej at seq 150 — the deliberate park the requirement predicted; report only. Gate green (verify_meshwork.sh exit 0, observed).
