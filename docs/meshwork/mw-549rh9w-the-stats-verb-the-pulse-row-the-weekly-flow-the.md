---
id: mw-549rh9w
title: The stats verb — the pulse row, the weekly flow, the hazard table, spans, lanes, top-10s, the placement histogram; portfolio stats
category: capability/stats
needs: [mw-p2q2fxd, mw-xg67266]
seq: 680
verify: 'all(run cargo test e2e::stats_tables, run cargo test perf::stats_1k_cold)'
docs:
  - docs/PROPOSAL-analytics.md#§-5-3-stats
  - docs/PROPOSAL-prioritization.md#§-7-build-ladder
status: open
created: 2026-09-07T16:32Z
---
With R-D item D.1: `stats [--window 7d|28d] [--json]` and `portfolio stats`, every table a
canned SELECT over a view, `--json` in the MW-C3 envelope, `check-perf.sh` gaining the ≤ 1 s
at 1K row. The placement histogram is `placed_by` once `q-bands` exists (a note until then).
`scripts/mine_rank.py` keeps only the Sidney experiment; its baseline tables come from `stats
--json`. DESIGN §6 row; `cli_surface_frozen` re-blessed.

## log
- 2026-09-07T16:32Z created
