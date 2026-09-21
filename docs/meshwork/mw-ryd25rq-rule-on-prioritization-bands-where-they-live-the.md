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
handoff: |
  The spike is in: docs/SPIKE-prioritization-rubric.md (commit 30744ac),
  and this task is
  now the only thing between the numbers and a build. Read §0 for the
  five-line answer and
  §7 for what is being asked; everything else is the evidence behind
  those.
  
  What the ruling has to say, item by item:
  - The key shape (§3): rank space — seq is the base position,
  sequence.md wins outright, an
  unplaced task starts mid-list, inherit substitutes the base, four terms
  in positions, every
  term rendered in `placed by:` and `why`. The alternative (seq points)
  was struck by its own
  falsifier: on sazed a two-step term moved seq-4 work 27 places.
  - The weights (§4), declared once in the portfolio repo (rubric.toml
  beside repos.toml,
  defaults compiled in). The number to set with care is `almost` (−2
  positions × done share)
  — it is the term doing the work (21 of 35 attributed moves) and on a
  hub doc it moves every
  live task citing it as a group (§2's hub table). `age` is 0 by policy;
  C.4 holds on the
  hazard.
  - The effort field (§6): measured cost by category explains η² = 0.11
  of per-task token
  variance, covers 37% of the ready set, and its spread went 24.2× →
  9.6× in 13 days. It does
  not stand in for an authored size. Three shapes are laid out: authored
  size as a fenced §3
  exception with a declared weight; no size and the cost term struck; a
  non-category proxy
  (none found). This is the owner's call, not ours.
  - Struck by number and not needing a ruling unless he disagrees: depth
  (duplicates unlock),
  queue_h (empty on ready), age as a boost, points space, path-level doc
  cohesion.
  - The `owner` label as the exemption: this repo's reveal marker
  mw-59f0t1q is unlabelled and
  the falsifier moved it 1 → 3. If the exemption is ruled, label the
  marker in the same commit.
  
  After the ruling, in this order: record R-C RULED in
  docs/PLAN-proposals-implementation.md
  §3 (this task's verify), DESIGN §15 decision, REQUIREMENTS §R rows;
  rewrite mw-ex5x0y2 into
  the rank-space key + placed_by term string (no [bands]) and trim
  mw-jzga8yj to triage + WIP;
  file the rubric.toml reader and the `ready --json` placement schema bump
  as their own tasks.
  
  To re-run the falsifier under the ruled weights before any Rust:
  python3 scripts/mine_cost.py --json cost.json
  MESHWORK_COST_JSON=cost.json python3 scripts/mine_rubric.py --falsify
  --weights almost=-1.5,cost=0
---
Owner-gated. Eight items, recommendations in the plan's R-C table. C.7 (transcripts as the
cost source) is one ruling with three consumers: rungs 5–6's gate, the field study's pricing,
the weft's WF12. C.6 also wants the one-line call on the mirror chain under the v1 gate.

## log
- 2026-09-07T16:32Z created
- 2026-09-21T15:56Z handoff by claude (b80d763e-77c7-4aa5-8ead-40c1daaa1288)

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
