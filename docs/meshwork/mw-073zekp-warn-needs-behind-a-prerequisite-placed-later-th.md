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
status: doing
created: 2026-09-07T16:32Z
claimed-by: claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
handoff: |
  Started 2026-09-12 at session end, nothing coded yet; tree is green.
  Plan: one more query in src/lint_views.rs (register "mentions lineage
  facts graph" so the graph helpers exist), over the helper relations:
  SELECT p.gid, p.place, l.gid, l.place FROM g_up u JOIN g_live p ON
  p.gid = u.pre JOIN g_live l ON l.gid = u.dep AND l.repo = p.repo WHERE
  l.place < p.place ORDER BY p.gid, l.place, l.gid — group by pre, take
  the first (best-placed) dependent, finding `needs-behind` on the
  prerequisite: "placed at <place|unranked> behind <dep> at <place>,
  which needs it — report only; the owner's fix differs (a park, or a
  missing rank)". 999999 renders as unranked. Test lint::needs_behind in
  tests/suite/lint_channel.rs via write_task: zz-pre1 seq 900 needed by
  zz-dep1 seq 150 (finding names both ids and places); zz-pre2 unranked
  needed by zz-mid2 seq 500 needed by zz-dep2 seq 10 (transitive:
  findings on zz-pre2 naming zz-dep2 and "unranked", and on zz-mid2);
  zz-pre3 seq 10 needed by zz-dep3 seq 20 (silent). Then DESIGN §6 lint
  row gets the code, gate, close, commit. mw-8aw53z9 (skill text) is
  owner-voiced prose — propose, don't land unprompted.
---
Nine contradictions portfolio-wide today, four of them findings a human wants (this repo's parked
mirror under the v1 gate; sazed's unranked prerequisite of a seq-72 task). Report only; the
owner's fix differs per case. Fixture: a parked prerequisite under a steered dependent. MW-R6b.

## log
- 2026-09-07T16:32Z created
- 2026-09-12T17:23Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T17:24Z handoff by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
