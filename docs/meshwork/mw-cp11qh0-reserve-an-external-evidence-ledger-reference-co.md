---
id: mw-cp11qh0
title: Reserve an external evidence-ledger reference convention
status: open
category: core/format
needs: [mw-dg5j1sv]
verify: grep -qi 'ledger' FORMAT.md
docs:
  - REQUIREMENTS-meshwork.md#§-3-non-goals
seq: 86
created: 2026-08-06
---
Owner-accepted 2026-08-06 (review; convention only — the §3 fences
hold: no integration, no execution, no network). Reserve, in
FORMAT.md, a syntax for referencing events in EXTERNAL append-only
evidence ledgers from log lines and comments (e.g.
`[evt:<ledger>:<hash8>]`), plus the reverse convention: an external
event carrying a meshwork gid. Any attestation tool can then join
observed events to task history with plain SQL — no coupling, just an
agreed key. Choosing the syntax is this task; the reservation itself
is one spec paragraph. Free today, unretrofittable across two
append-only histories later.

## log
- 2026-08-06 created
