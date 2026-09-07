---
id: mw-073zekp
title: Warn needs-behind — a prerequisite placed later than something that needs it, naming both ids and both places
category: capability/rank
needs: [mw-zwgp6x7]
seq: 570
verify: run cargo test lint::needs_behind
docs:
  - docs/PROPOSAL-prioritization.md#§-5-requirements
  - docs/PROPOSAL-prioritization.md#§-1-what-the-stores-say
status: open
created: 2026-09-07T16:32Z
---
Nine contradictions portfolio-wide today, four of them findings a human wants (this repo's parked
mirror under the v1 gate; sazed's unranked prerequisite of a seq-72 task). Report only; the
owner's fix differs per case. Fixture: a parked prerequisite under a steered dependent. MW-R6b.

## log
- 2026-09-07T16:32Z created
