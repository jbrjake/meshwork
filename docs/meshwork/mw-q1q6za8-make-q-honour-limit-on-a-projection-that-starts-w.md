---
id: mw-q1q6za8
title: Make q honour LIMIT on a projection that starts with path — today it returns the whole table
status: open
category: core/query
verify: run cargo test e2e::query_limit_leading_path
docs:
  - docs/DESIGN-meshwork.md#§-6-cli-surface
seq: 575
created: 2026-09-21T15:55Z
handoff: |
  Reproduced on this store at 0.4.0 with the debug binary: `q "SELECT
  path, created FROM
  tasks LIMIT 2"` returns every row (217), text and --json alike; `SELECT
  created, path …
  LIMIT 2` and `SELECT path FROM tasks LIMIT 2` return 2. The trigger is a
  projection whose
  FIRST column is `path` with any second column (`path, title` and `path,
  created, id` fail
  the same way). `run_query` in src/cli/query.rs hands the SQL to
  DataFusion 51 verbatim, so
  nothing of ours rewrites it.
  
  Where to look first: src/tables.rs builds the `tasks` MemTable — check
  whether it registers
  more than one partition or batch (the archive bundle documents,
  mw-bvxpeef, arrive as
  separate rows and may land as a second batch). A LIMIT that DataFusion
  pushes below a
  projection per partition, then forgets to re-apply globally, is the
  shape that fits "first
  column decides it". Try `EXPLAIN SELECT path, created FROM tasks LIMIT
  2` through `q` and
  compare with the `created, path` plan; then try a single-batch MemTable
  (`MemTable::try_new(schema, vec![vec![batch]])`) and a `SessionConfig`
  with
  `target_partitions = 1`. Whichever fixes it, the test is
  `e2e::query_limit_leading_path`:
  both column orders, text and --json, row count 2 on the alpha fixture
  — and a fixture with
  an archive bundle, since that is the likely second batch.
  
  Not the `path` column's content: the same query with `id, path` is fine,
  so it is position,
  not a value.
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
