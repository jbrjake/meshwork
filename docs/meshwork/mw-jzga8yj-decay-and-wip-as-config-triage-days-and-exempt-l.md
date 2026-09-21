---
id: mw-jzga8yj
title: Decay and WIP as config — triage_days and exempt_labels, the wip cap, open-decayed and wip-over-cap, the headline line
category: capability/rank
needs: [mw-bwwd75h, mw-ryd25rq, mw-v4d2hwt]
seq: 630
verify: run cargo test lint::open_decayed
docs:
  - docs/PROPOSAL-prioritization.md#§-5-requirements
  - docs/PROPOSAL-analytics.md#§-4-10-flow
status: open
created: 2026-09-07T16:32Z
---
`[decay] triage_days` (14) and `exempt_labels` (`owner`); `[wip] cap`. The pulse's queue line
reads them; the headline's first line reads `doing 10 (cap 3 — OVER)` when capped; `lint` warns
`open-decayed` per task past the age and `wip-over-cap`. Nothing is blocked by any of it. The
hazard table is the `hazard` view; the fixture with the known survival curve pins the fit.
MW-R10, R12–R14.

## log
- 2026-09-07T16:32Z created
