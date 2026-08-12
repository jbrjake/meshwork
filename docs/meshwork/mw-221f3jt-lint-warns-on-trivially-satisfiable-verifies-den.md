---
id: mw-221f3jt
title: "lint warns on trivially-satisfiable verifies (denylist heuristic)"
category: core/verify
relates: [mw-175bn4c, mw-6wdpz1b]
verify: cargo test lint::trivial_verify_warn
seq: 210
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline
status: open
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). Denylist heuristic: bare `true`, `echo`,
`touch`, `test -f <path>` where the same path appears nowhere else in
the repo. Warn only — catches most of the golf, and the false positives
are cheap. Third sibling of the vacuous-verify pair: mw-6wdpz1b covers
the ABSENT verify, mw-175bn4c the present-but-already-green one; this
covers present-and-trivial, statically — no execution, so no trust-gate
interaction. Rule on all three together.

## log
- 2026-08-08T14:09Z created

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Two adoption-week data points. sazed's flagship Q21 task closed on verify grep -q 'Q21' docs/batch-door.md — satisfiable since the day it was filed (fc237a1a); only agent discipline made the close honest. And leras's store carried four rot classes at once — 28 rg-verifies exiting 127 under close's sh -c, zero-match cargo filters, 20 prose verifies, and 2 grep anchors that went stale-green when a refactor moved the anchored code — while lint reported 0 errors 0 warnings throughout. The stale-green class argues the denylist should pair with a run-open-verifies-and-flag-greens sweep (mw-dx4pndb's dry-run verb would be its engine).
