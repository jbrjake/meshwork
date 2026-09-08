---
id: mw-sg8phqk
title: Publish FORMAT.md §Views with the multi-store conformance fixture blessed at two clock stamps
category: core/format
needs: [mw-w2920xb]
seq: 430
verify: 'all(contains FORMAT.md /^## Views/, exists fixtures/conformance/expected/views/pulse.json)'
docs:
  - docs/PROPOSAL-analytics.md#§-11-build-ladder
  - docs/PROPOSAL-analytics.md#§-7-budgets
  - docs/ASKS-analytics-and-field-study.md#§-3-the-asks
status: open
created: 2026-09-07T16:32Z
---
Spec first; the conformance test lands red. §Views: the thirteen names and columns; each
column's derivation in prose as the normative text (the weft's SQLite and the UI cannot run
DataFusion SQL); Appendix A as the reference implementation (`scripts/mine_views.py --sql` is
the source); the stamp guard; recursion bound 64 and the label-propagation stop; the closure
convention (done = current status, reopen ≠ closure, date-only = 00:00Z); the `sessions`/
`attention` sample-bias note (claims cover half of closed work); `clock` honouring
`MESHWORK_TODAY` in both conforming forms, loud on anything else, read-only idiom, and how it
differs from a read at a ref. Change `asks` before blessing: expose `answer_open`,
`answer_done`, `answered`, `unanswered` — state, not the suppression rule. Fixtures: the golden
store at two stamps; a two-store linked fixture (an ask A→B, an open answer, a done answer, a
cross-repo `needs`, a handoff naming a foreign id, a known survival curve with a censored tail)
with the union views blessed. README rows for `expected/views/`. Register nothing yet.

## log
- 2026-09-07T16:32Z created

## comments
- 2026-09-08T13:39Z [claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)] When §Views lands, add the note the event envelope task (mw-wx80vev) owes it: scripts/mine_sessions.py --events emits {repo, gid, kind, at, actor, note} — the events view's column subset — plus session and detail; the store side of the same timeline is SELECT repo, gid, kind, at, actor, note FROM events. The envelope is documented in the script's header.
