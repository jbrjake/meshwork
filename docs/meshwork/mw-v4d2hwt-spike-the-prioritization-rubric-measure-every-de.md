---
id: mw-v4d2hwt
title: "Spike the prioritization rubric — measure every derived dimension's discriminating power on the live stores, then propose a term-by-term key"
status: open
category: capability/rank
verify: exists docs/SPIKE-prioritization-rubric.md
docs:
  - docs/PROPOSAL-prioritization.md#§-1-what-the-stores-say-the-2026-09-01-baseline
  - docs/cost-baseline.md
  - docs/PROPOSAL-analytics.md#§-4-5-graph-structure-over-live-needs
  - docs/PLAN-proposals-implementation.md#§-3-rulings
seq: 385
created: 2026-09-20T23:50Z
handoff: |
  Nothing is proven about any dimension yet. The only measurements in the
  record are unlock > 0 on
  35 of 457 live tasks (8%), the inheritance experiment's four findings,
  and the 24.2x cost spread
  in docs/cost-baseline.md. Everything else in the rubric is unmeasured.
  
  Start from scripts/mine_rank.py: it already computes the §1 baseline
  table, the discrete-hazard
  table, lanes, and the Sidney/inheritance experiment, read-only, per
  repo. What it does NOT yet
  measure is exactly the list this task adds — sibling and umbrella
  completion, doc cohesion (live
  tasks sharing a docs: anchor with closed ones), and per-task cost joined
  from cost-baseline.md.
  scripts/mine_cost.py is the cost miner and its --check is its self-test.
  
  The views R-A made contract carry most of the inputs: graph (unlock,
  depth, lane), facts and
  spans (age_h, queue_h, service_h), lineage (spawn trees, umbrellas),
  asks. Read them with `q` or
  `portfolio q` — both are reads and neither prunes (MW-S14). Never
  `portfolio ready`/`next` for
  analysis; those still prune sequence.md.
  
  The one trap, because it is what sank the last attempt: every experiment
  must be able to come out
  differently. If a dimension's weight is derived from the current
  placement, the result reproduces
  the current placement and means nothing — that is what `w = f(place)`,
  `p = 1` did. State each
  dimension's null before measuring it, and report the dimensions that
  split nothing as struck,
  with their numbers, rather than dropping them silently.
  
  Owner-scoped and still open: he named t-shirt-sized effort as a
  candidate. Measured cost by
  category is inside REQUIREMENTS §3's fence (an observation, not an
  estimate, per the R-A scope
  ruling); an authored size is not. Test whether the measured proxy does
  the work, and if it does
  not, bring it to him as a ruling with numbers — do not resolve it by
  writing the field in.
  
  Deliverable is docs/SPIKE-prioritization-rubric.md; the verify is exists
  on that path, red now.
  No code, no format change. mw-ryd25rq rules on what this reports;
  mw-ex5x0y2 and mw-jzga8yj get
  rewritten or dropped to match.
---

Bands over categories were rejected 2026-09-20; this replaces them. Why the old conclusion does
not hold — the circular Sidney run, the 24.2× cost falsifier that fired and was not followed — is
in the plan's §3 R-C, with the owner's words on `mw-ryd25rq`.

Produce `docs/SPIKE-prioritization-rubric.md`, carrying:

1. **One row per candidate dimension, each derived and never authored**, with the view it reads
   and its discriminating power measured on the nine live stores: unlock count and transitive
   depth (`graph`), age and queue time (`facts`/`spans`), sibling and umbrella completion — the
   "almost done" boost (`lineage`), doc cohesion (live tasks sharing a `docs:` anchor with closed
   ones), and measured cost by category (`docs/cost-baseline.md`). A dimension that splits nothing
   on real stores is struck here, by number.
2. **A key that renders term by term, never a scalar.** `placed by:` becomes named terms with
   their contributions. A rank that cannot say why it ranked is out of scope by construction —
   that is what makes a rubric admissible where a solver was not.
3. **Weights declared as policy, portfolio-wide, not fitted and not per-repo.** One shared file
   every store reads. `seq` and `sequence.md` stay the owner's override and always win.
4. **A falsifier that can run**: the rubric's order against today's on all nine stores, naming
   every disagreement and the term that caused it, so each can be judged by reading it.
5. **The authored-effort question, answered with numbers.** The owner named t-shirt-sized effort
   as a candidate dimension. Test whether measured cost by category does the work an authored size
   would; if it cannot, bring the authored field to a ruling rather than assuming it out.

No code, no format change, no ruling: this reports numbers and proposes a shape. `mw-ryd25rq`
rules on it; `mw-ex5x0y2` and `mw-jzga8yj` are rewritten or dropped to match.
`scripts/mine_rank.py` already carries the graph and hazard halves.

## log
- 2026-09-20T23:50Z created
- 2026-09-21T00:10Z handoff by claude (6104c7c3-6cf9-4c90-90ac-e4ec66924a6d)
