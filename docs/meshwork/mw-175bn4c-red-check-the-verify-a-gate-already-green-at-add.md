---
id: mw-175bn4c
title: "Red-check the verify: a gate already green at add/start cannot detect the work"
status: open
category: core/verify
verify: cargo test e2e::verify_red_check
discovered-from: mw-ntt5
seq: 125
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline
created: 2026-08-07T13:47Z
---
The pilot hit this class TWICE in one day. Migration: 5 of 30 executed
verifies passed vacuously on already-green state and had to be tightened
by hand. Work session: the session's most important task (the door fix)
was filed with `verify: grep -q '0.95' docs/engine-matrix.md` — a line
the agent had just written — so the fix-task could close with zero code
changed; its own repair commit names the class: "the exact
vacuous-verify failure the store exists to prevent." Wanted: a check
that a task's verify FAILS while the work is undone — red-first for
gates. Design constraints: running an arbitrary verify is exactly what
the MW-E5 trust gate exists to gate, so auto-executing at `add` is out;
candidates are an opt-in `add --red-check`/`start`-time check behind the
same approval, or a `lint` mode. Sister task
[[capture-before-verifiable-start-gates-on-verify-n]] (mw-6wdpz1b)
covers the ABSENT verify; this covers the PRESENT-but-already-green one.
Decide the two together — same surface ruling.

## log
- 2026-08-07T13:47Z created

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Field evidence: sa-va0tvyx was minted with a placeholder grep already green at mint (dbc4d8cd), Q21's doc-mention verify was green from birth and later closed on it, and leras faba7815's manual red-check found 3 of 27 verifies GREEN before any work — 2 of them rotted anchors. Red-at-add/start would have caught every one at authoring time.
