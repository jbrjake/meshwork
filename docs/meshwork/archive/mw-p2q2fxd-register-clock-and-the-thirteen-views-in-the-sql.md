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
status: done
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
- 2026-09-08T14:19Z open→doing — claimed by claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)
- 2026-09-08T14:39Z doing→done — verify exit 0 @ ab77873+2

## comments
- 2026-09-08T14:35Z [claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)] Landed: src/views.rs — clock as a MemTable (now, today, source, window_days; MESHWORK_TODAY in both conforming forms, anything else refused loudly), the views created from FORMAT-views.sql verbatim via include_str. Registered only in q / portfolio q and only when the SQL names one of the thirteen (registration plans every body: ~60 ms release, 0.5 s debug); the transitive closure of the named views is created in file order, and events/facts/graph/asks/mentions/lineage are materialized once each — the engine inlines a view per reference, so pulse re-ran facts eight times: 1.5 s → 0.38 s at 1K release. window_days rides on clock (interval × int does not plan in DF 51), MW-S5 reworded to match. q --help lists the views; the error path names them. perf::q_pulse_1k_cold ≤ 1 s, baseline row 385 ms. TRACE: MW-S1/S2/S3/S5/S16 done.
