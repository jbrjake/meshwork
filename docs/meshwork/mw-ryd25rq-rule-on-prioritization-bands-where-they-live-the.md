---
id: mw-ryd25rq
title: Rule on prioritization — bands, where they live, the default band, aging posture, unit value, inheritance opt-in, the token measurement, stats over lint --stats
category: meta/ruling
labels: [owner]
seq: 390
verify: contains docs/PLAN-proposals-implementation.md /R-C RULED 20[0-9]{2}-[0-9]{2}-[0-9]{2}/
docs:
  - docs/PLAN-proposals-implementation.md#§-3-rulings
  - docs/PROPOSAL-prioritization.md#§-10-rulings-requested
status: open
created: 2026-09-07T16:32Z
needs: [mw-v4d2hwt]
---
Owner-gated. Eight items, recommendations in the plan's R-C table. C.7 (transcripts as the
cost source) is one ruling with three consumers: rungs 5–6's gate, the field study's pricing,
the weft's WF12. C.6 also wants the one-line call on the mirror chain under the v1 gate.

## log
- 2026-09-07T16:32Z created

## comments
- 2026-09-20T23:51Z [claude (6104c7c3-6cf9-4c90-90ac-e4ec66924a6d)] Owner ruling 2026-09-20, in session: C.1/C.2/C.3 as posed are rejected, and the ruling now waits
  on mw-v4d2hwt. His words: "seq is not good enough for prioritization. it's too coarse and
  undifferentiated a mechanism"; "making people set up another repo specific config file sucks and
  i hate it"; "making it still the only prioritization value but with hard coded bands based on
  arbitrary categories sucks even more". What he asked for instead: "some kind of rubric that
  covers different dimensions that are universal to work and shared across repos in a way that can
  be reasoned about", weighing "the number of downstream tasks unlocked by completing it, the age
  of the task, the number of other tickets tied to the same design docs or parent tasks that have
  already been completed (boost if it's almost done)", and "weigh sequence against, i don't know,
  t-shirt sizing level of effort, amongst other things".
  
  Why the old conclusion does not hold. The §1 Sidney run used `w = f(place)`, `p = 1`; under unit
  processing times Smith's rule orders by weight, and the weight was the current placement, so
  "identical order" is the arithmetic, not a finding. The §8 falsifier fired the other way and was
  not followed: docs/cost-baseline.md measures a 24.2x spread in tokens per task across 8
  categories at n >= 7 and states the rung-0 question is answered yes, while §1 reading 2 declared
  cost flat on wall-clock service time -- the measure a 98%-queue lifetime distorts most. The only
  dimension tested in isolation, priority inheritance, found four real placement errors seq missed.
  
  Open, and the owner's to settle once the spike reports: he named t-shirt sizing, which is an
  authored estimate and therefore against REQUIREMENTS §3 as written; measured cost by category is
  an observation already inside the fence R-A drew on 2026-09-08. The spike must test whether the
  measured proxy does the work an authored size would; if it cannot, the authored field comes back
  to a ruling with numbers attached rather than being assumed out.
  
  Still rulable without the spike: C.4 (aging posture), C.6's mirror-chain contradiction, C.7
  (already landed as mine_cost.py + cost-baseline.md), C.8 (settled by R-D 2026-09-20).
