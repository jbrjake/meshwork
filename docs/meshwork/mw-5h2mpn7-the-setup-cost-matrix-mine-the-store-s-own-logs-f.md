---
id: mw-5h2mpn7
title: The setup-cost matrix — mine the store's own logs for the first empirical agent context-switch numbers (before the reveal)
status: open
category: analysis
verify: exists docs/setup-cost-matrix.md
docs:
  - ../REVIEW-fresh-eyes-2026-08-14.md
  - ../DESIGN-thought-mill.md
seq: 20
created: 2026-08-17T03:16Z
handoff: |
  Fresh-eyes Gold III: agent context-switch cost is logged and thrown away
  every session — except here, where the store keeps it. The dataset
  already exists: task logs (created/start/close timestamps, claimed-by),
  comments, and prime digests across six adopter repos and ~1,100 commits.
  Build a read-only miner (a `q`-driven script or a small bin) that
  produces the setup-cost matrix: per-repo session ramp (time from session
  start to first task act), cross-repo switch cost, task-touch fan-out per
  session, aging-vs-touch curves. Land it as docs/setup-cost-matrix.md
  with the method stated and every number carrying its denominator.
  Sequenced BEFORE reveal prep on purpose: the reveal then ships with a
  headline number instead of a bare repo. These numbers are also
  thought-mill rung 0's first empirical input — DESIGN-thought-mill.md
  stages no work by owner ruling, and this matrix is what makes that
  decant call data instead of vibes.
---

## log
- 2026-08-17T03:16Z created
