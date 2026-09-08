---
id: mw-38hwtvd
title: Price the findings — tokens per session, per task by category, main chain vs subagents
category: analysis
seq: 250
verify: exists docs/cost-baseline.md
docs:
  - docs/ASKS-analytics-and-field-study.md#§-3-the-asks
  - docs/PROPOSAL-prioritization.md#§-7-build-ladder
status: done
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
- 2026-09-08T13:49Z open→doing — claimed by claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)
- 2026-09-08T14:04Z doing→done — verify exit 0 @ 599a182+1

## comments
- 2026-09-08T14:04Z [claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)] Landed: scripts/mine_cost.py (--check ok, --write) and docs/cost-baseline.md. Headlines: median 461k fresh tokens per session at 99% cache hit; 20% of fresh tokens on subagents (46 sessions), whose transcripts live beside the session in <session>/subagents/ — the study's scripts never read them; per task by category medians span 24× (skill 9k → kwaan 227k, 8 categories n ≥ 7), so the rung-0 gate on rungs 5–6 is met and recorded in PROPOSAL-prioritization §1; hand-edits cost 650 API turns; 72 meshwork calls past 5 min, 52 of them on verbs that run nothing — approval holds. Owner, mid-session: the product is the CLI; this is prototyping. Gate green (observed).
