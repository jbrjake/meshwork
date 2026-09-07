---
id: mw-38hwtvd
title: Price the findings — tokens per session, per task by category, main chain vs subagents
category: analysis
seq: 250
verify: exists docs/cost-baseline.md
docs:
  - docs/ASKS-analytics-and-field-study.md#§-3-the-asks
  - docs/PROPOSAL-prioritization.md#§-7-build-ladder
status: open
created: 2026-09-07T16:32Z
---
One pass over records the scripts already parse: `message.usage` (input, output, cache read,
cache write) per session, split by `isSidechain`, attributed to tasks by the `start`/`close`
ids in the session's meshwork calls and to categories by the store. Outputs: `scripts/mine_cost.py
--check`; `docs/cost-baseline.md` — the deliverable, the setup-cost matrix's sibling, with every
denominator, which the weft's WF12 reads instead of mining twice; a priced column on the study's
Tier 1/Tier 2 findings; and the one number rung 0 owes the prioritization ladder — does cost
vary ≥ 3× across categories with n ≥ 7 — recorded in that proposal's §1 baseline table. Gated by
R-C item C.7 only for the *committed* data; the script itself is read-only and needs no ruling.

## log
- 2026-09-07T16:32Z created
