---
id: mw-p2q2fxd
title: Register clock and the thirteen views in the SQL session, list them in q's error path, read stats.window_days
category: core/query
needs: [mw-sg8phqk]
seq: 440
verify: 'all(run cargo test tables::views_registered, run cargo test conformance::views_golden)'
docs:
  - docs/PROPOSAL-analytics.md#§-5-1-q
  - docs/PROPOSAL-analytics.md#§-6-4-the-stats-table
  - FORMAT.md#§-config-toml
status: open
created: 2026-09-07T16:32Z
---
`clock` as a one-row MemTable; views created from the published SQL after the six tables and the
UDF, `[stats] window_days` (default 7) substituted into `p_win`. Register views only in sessions
that query them (`q`, `portfolio q`, later `stats` and the lint findings), never in the gated
`ready`/`prime` sessions — DataFusion plans a view body at registration and the cost is
unmeasured; add a perf row for `q` over `pulse` cold at 1K. The `q` error path's queryable-tables
line gains the view names. The conformance test goes green at both stamps and on the linked
fixture; `mine_views.py --check` stays as the cross-store probe.

## log
- 2026-09-07T16:32Z created
