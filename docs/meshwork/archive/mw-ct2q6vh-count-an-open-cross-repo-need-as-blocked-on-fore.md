---
id: mw-ct2q6vh
title: "Count an open cross-repo need as blocked on foreign, not unresolved, in prime's graph line"
category: core/render
verify: run cargo test open_foreign_need_counts_as_foreign
docs:
  - FORMAT.md#graph--structure-over-live-needs
  - FORMAT.md#pulse--one-row-per-repo-the-weather
status: done
created: 2026-10-02T15:21Z
---
prime's graph line reads `blocked on foreign 0 · unresolved 1` for a task whose cross-repo `needs` target is registered, checked out and open.

`prime` and single-repo `q` build their session from `query::terminal_foreign` (src/cli/query.rs), which injects only done and dropped foreign rows. `graph::compute` then finds no row for an open target and counts it under both `needs_open` and `needs_unresolved` (src/graph.rs, the `dst == None` arm). `pulse::compute` reports it unresolved.

Reserve unresolved for targets the registry does not know or whose checkout or task file is absent. Count an open, resolvable target as blocked on foreign. Keep FORMAT-views.sql and the Rust graph columns in step; the differential test pins them.

## log
- 2026-10-02T15:21Z created
- 2026-10-04T15:34Z open→doing — claimed by claude (5566559a-b6fd-4780-8c73-3214f8c33f5d)
- 2026-10-04T15:46Z doing→done — verify exit 0 @ 506dc44+6

## comments
- 2026-10-02T17:02Z [claude (6fa0dcc3-62c5-4a7b-a6b1-de25f58287de)] Not part of the notes demo, which runs on the pinned release unchanged; deferred like the other bugs. A gate-green implementation (test open_foreign_need_counts_as_foreign red then green, full gate exit 0) sits in this clone's git stash, message starting 'P2 (mw-ct2q6vh)'; it changes FORMAT.md's graph and union-semantics text, so it needs the owner's go before it lands.
