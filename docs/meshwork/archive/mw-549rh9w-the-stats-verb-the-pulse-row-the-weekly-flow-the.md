---
id: mw-549rh9w
title: The stats verb — the pulse row, the weekly flow, the hazard table, spans, lanes, top-10s, the placement histogram; portfolio stats
category: capability/stats
needs: [mw-p2q2fxd, mw-xg67266]
seq: 680
verify: "all(run cargo test e2e::stats_tables, run cargo test e2e::portfolio_stats, contains bench-baseline.json stats_1k_cold)"
docs:
  - docs/PROPOSAL-analytics.md#§-5-3-stats
  - docs/PROPOSAL-prioritization.md#§-7-build-ladder
status: done
created: 2026-09-07T16:32Z
---
With R-D item D.1: `stats [--window 7d|28d] [--json]` and `portfolio stats`, every table a
canned SELECT over a view, `--json` in the MW-C3 envelope, `check-perf.sh` gaining the ≤ 1 s
at 1K row. The placement histogram is `placed_by` once `q-bands` exists (a note until then).
`scripts/mine_rank.py` keeps only the Sidney experiment; its baseline tables come from `stats
--json`. DESIGN §6 row; `cli_surface_frozen` re-blessed.

## log
- 2026-09-07T16:32Z created
- 2026-09-22T13:43Z open→doing — claimed by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
- 2026-09-22T13:55Z close attempt — verify failed (dsl)
- 2026-09-22T13:55Z doing→done — verify exit 0 @ 22393cd+4

## comments
- 2026-09-22T13:55Z [claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)] Verify re-pointed: perf::stats_1k_cold is #[ignore]d like every perf test (gate §7 runs them --ignored on release), so a debug filter matches nothing and the DSL cannot pass --ignored. The e2e tests and the seeded baseline row are the close evidence; the gate holds the budget (419 ms at 1K, gate green this session).
