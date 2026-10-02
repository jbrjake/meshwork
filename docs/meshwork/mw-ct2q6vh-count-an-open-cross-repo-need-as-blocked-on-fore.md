---
id: mw-ct2q6vh
title: "Count an open cross-repo need as blocked on foreign, not unresolved, in prime's graph line"
category: core/render
seq: 210
verify: run cargo test open_foreign_need_counts_as_foreign
docs:
  - docs/PLAN-demo-notes.md#meshwork-prerequisites
  - FORMAT.md#pulse--one-row-per-repo-the-weather
status: open
created: 2026-10-02T15:21Z
---
prime's graph line reads `blocked on foreign 0 · unresolved 1` for a task whose cross-repo `needs` target is registered, checked out and open.

`prime` and single-repo `q` build their session from `query::terminal_foreign` (src/cli/query.rs), which injects only done and dropped foreign rows. `graph::compute` then finds no row for an open target and counts it under both `needs_open` and `needs_unresolved` (src/graph.rs, the `dst == None` arm). `pulse::compute` reports it unresolved.

Reserve unresolved for targets the registry does not know or whose checkout or task file is absent. Count an open, resolvable target as blocked on foreign. Keep FORMAT-views.sql and the Rust graph columns in step; the differential test pins them.

The notes demo prints this line in every notes beat, where the cli's TITLE needs sync's open per-field merge.

## log
- 2026-10-02T15:21Z created
