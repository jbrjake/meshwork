---
id: mw-zwgp6x7
title: Compute the graph view's columns in Rust for the gated verbs, pinned to the SQL by a differential test; why prints lane, inherit and unlock
category: capability/rank
needs: [mw-p2q2fxd]
seq: 560
verify: 'all(run cargo test graph_rust_matches_view, run cargo test why_prints_placement)'
docs:
  - docs/PROPOSAL-analytics.md#§-4-5-graph
  - docs/PROPOSAL-prioritization.md#§-5-requirements
status: done
created: 2026-09-07T16:32Z
---
`ready` and `prime` are gated at 100 ms and from bands on will order by `unlock`; the SQL view
is the specification. One pass over live tasks and `needs` edges: `unlock`, `depth`, `inherit`
(same repo), `needs_behind`, `lane` over `needs` only, `lane_size`, `needs_open_foreign`,
`needed_by_foreign`. `e2e` differential test: `SELECT … FROM graph` on every fixture store equals
the Rust rows column by column under `MESHWORK_TODAY`. `why <id>` on a ready task prints its
lane, inherit and unlock; on an umbrella it says `hidden: N live children`. MW-R6/R7/R16.

## log
- 2026-09-07T16:32Z created
- 2026-09-12T16:14Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T16:21Z doing→done — verify exit 0 @ 732087b+10
