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
