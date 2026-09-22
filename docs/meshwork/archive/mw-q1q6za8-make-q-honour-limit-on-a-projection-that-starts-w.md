---
id: mw-q1q6za8
title: Make q honour LIMIT on a projection that starts with path — today it returns the whole table
status: done
category: core/query
verify: run cargo test e2e::query_limit_leading_path
docs:
  - docs/DESIGN-meshwork.md#§-6-cli-surface
seq: 575
created: 2026-09-21T15:55Z
---

`q` returns the whole table when a `LIMIT` sits on a projection that starts with `path` and
carries a second column. Same store, same binary, text and `--json` alike:

```
SELECT path, created FROM tasks LIMIT 2        → 217 rows
SELECT path, title FROM tasks LIMIT 2          → 217 rows
SELECT path, created, id FROM tasks LIMIT 2    → 217 rows
SELECT created, path FROM tasks LIMIT 2        → 2 rows
SELECT path FROM tasks LIMIT 2                 → 2 rows
SELECT id, path FROM tasks LIMIT 2             → 2 rows
SELECT seq, created FROM tasks LIMIT 2         → 2 rows
```

`run_query` in `src/cli/query.rs` hands the SQL to DataFusion 51 unchanged, so the loss is in
the plan, not in a rewrite; the `tasks` table is the suspect — reproduce on a view and on the
`edges` table to see whether it is the MemTable's partitioning, the `path` column's position,
or a LIMIT pushdown DataFusion applies per partition. A `q` that ignores `LIMIT` on some
projections is a reader that lies quietly; the fix is either a session config or a version
bump, and the test pins the row count on the failing shape.

## log
- 2026-09-21T15:55Z created
- 2026-09-21T16:39Z handoff by claude (b80d763e-77c7-4aa5-8ead-40c1daaa1288)
- 2026-09-22T13:31Z open→doing — claimed by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
- 2026-09-22T13:42Z doing→done — verify exit 0 @ 02a6aa2+5

## comments
- 2026-09-22T13:42Z [claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)] Root cause was not batching: the tasks MemTable is one partition, one batch. DataFusion 51's physical ProjectionPushdown swaps a column-reordering projection into MemorySourceConfig and rebuilds it via try_new, which drops fetch; 54.x clones the source through the swap. Fix: tables::session_state filters the rule by name. The bump that removes the filter is mw-3p3sd2p.
