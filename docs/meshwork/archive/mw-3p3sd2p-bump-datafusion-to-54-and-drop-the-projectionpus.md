---
id: mw-3p3sd2p
title: Bump DataFusion to 54 and drop the ProjectionPushdown filter from tables::session_state
status: done
category: core/query
discovered-from: mw-q1q6za8
verify: "all(contains Cargo.toml /^datafusion = \"5[4-9]/, lacks src/tables.rs ProjectionPushdown)"
docs:
  - docs/DESIGN-meshwork.md#§-15-decisions
seq: 800
created: 2026-09-22T13:36Z
---

`tables::session_state` filters DataFusion's physical `ProjectionPushdown` rule out of every session because the 51 series rebuilds an in-memory source without its `fetch` when the rule absorbs a column-reordering projection, which is why `SELECT path, created FROM tasks LIMIT 2` returned the whole table (mw-q1q6za8). DataFusion 54 carries the swap through a clone of the source, `fetch` included, with its own unit test `try_swapping_with_projection_preserves_fetch` in `datafusion-datasource`.

The bump is the whole change: `datafusion = "54"` (or later) in `Cargo.toml`, the crates declared as already-in-the-tree-via-datafusion (`sha2`, `regex`) re-checked against the new tree, `Cargo.lock` regenerated, and the filter plus its doc comment deleted from `src/tables.rs` so the default session is used again. `e2e::query_limit_leading_path` stays as the pin; it must pass on the default rules before the filter goes. `scripts/check-perf.sh` runs after the bump, since the planner changes underneath every canned query, and the gate's MW-C4 budgets hold or the bump waits.

## log
- 2026-09-22T13:36Z created
- 2026-09-22T14:34Z open→doing — claimed by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
- 2026-09-22T14:47Z doing→done — verify exit 0 @ dfca3c0+5

## comments
- 2026-09-22T14:47Z [claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)] Landed offline from the registry cache. One adaptation beyond the plan: 54's MemTable::try_new refuses batches whose schema differs from the planned one, and the recursive graph view produces a different nullability than it plans — views::materialize now takes the schema from the batches. Every view golden stayed byte-equal. Medians at 1K: q over pulse 385→167 ms, stats 419→190 ms; baseline reseeded.
