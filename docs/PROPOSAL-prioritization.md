# PROPOSAL — prioritization: layered routing over a computed floor

**Status: proposal, unruled.** Written 2026-09-01 from four sources: the nine registered stores
(queried read-only, per repo, every number below carries its denominator), the setup-cost
matrix (`docs/setup-cost-matrix.md`), the portfolio's own writing on the subject
(`portfolio/task-prioritization-notes.md`, `portfolio/DESIGN-graph-of-graphs.md` §5,
`portfolio/PROPOSAL-meshwork-for-refresh-sessions.md` §2.1/§2.5/§5, `portfolio/sequence.md`,
atlas `QUERIES.md` Q7–Q9 and `decisions/open-calls.md`), and a verification pass over the
scheduling-theory claims those notes rest on. The verification, the prior-art survey, the Rust
feasibility notes, and the baseline's method live in the companion,
`PROPOSAL-prioritization-evidence.md`; every theorem named below is either confirmed there with
a primary citation or marked as corrected.

**The thesis in one paragraph.** meshwork already has a total ordering (`sequence.md` →
`repos.toml` → `seq` → `created`) and the two inputs no other tracker has: a cycle-linted
precedence DAG and a timestamped lifecycle log. What it lacks is a *default* — an unsequenced
task sorts last, so a task must carry a hand-set `seq` to exist in the order at all, and 399 of
457 live tasks do. The proposal is not a solver that replaces the owner's ordering; the
portfolio's own review rejected that, correctly, because *a solver would launder the reasons.*
It is the second snippet's shape applied to ordering: a **sealed, total, computed floor** under
the two authored layers that exist today, a **category-rule layer** between them so policy is
declared once per category instead of once per ticket, and a **recorded placement** on every
ranked task naming which layer placed it — the one number that tells the owner whether he is
steering the tool or carrying it. Scheduling theory was supposed to supply the floor's *order*.
The baseline measured first (§1) says which of its results bite here: the queue law and the
hazard result do, loudly; the value-density results wait on inputs the store does not have —
and the proposal says so rather than building an algorithm in search of its inputs.

---

## 1. What the stores say — the 2026-09-01 baseline

Read-only, per-repo `meshwork q --json` over all nine registered stores (never a `portfolio`
verb: those autoprune `sequence.md`). Method and script in the evidence doc §F.

| measure | value | denominator |
|---|---|---|
| live tasks carrying a hand-set `seq` | **399 (87%)** | 457 live (open/doing/blocked) |
| live tasks with a category (bands could place them) | 301 (66%) | 457 — leras 5/64, oreseur 0/9, wyndam 0/15, tensoon 17/38 |
| closes with a `→doing` line (service time measurable) | 264 (57%) | 464 dated closes — sazed 42/167 |
| **recorded service time** (`doing→done`) | **median 0.20 h, p90 2.8 h; 80% ≤ 1 h** | 264 |
| category service medians, categories with n ≥ 7 | 0.03 h … 1.27 h | 9 categories |
| queue time (`created→doing`) | median 18.1 h, p90 285 h | 264 |
| per-task queue : service | **median 38×; median share of lifetime spent waiting 0.98** | 264 |
| **daily close rate by age** (discrete Kaplan–Meier: share of tasks open at age *a* and observable through *a*+1 d that closed `done` in that day; `dropped` tracked separately) | day 0 **27%** (17% inside 4 h) · day 1 **8%** · days 2–3 ≈ 4% · **days 4–15 ≈ 2–4%, flat** · days 16+ < 1% (n small) | at risk: 834 / 583 / 517 / 464 … 233 at day 14 |
| live tasks past 14 d (the triage age proposed in MW-R13) | **219 (48%)** — sazed 146/239, leras 48/64, meshwork 8/11 | 457; 210 of the 219 carry a `seq` |
| oldest task in any store | < 30 d | stores were born 2026-08-05 … 08-21; days ≥ 16 have short residual follow-up and wide intervals |
| live tasks with any live transitive dependent (`unlock > 0`) | **35 (8%)** | 457; `needs` edges with both ends live: 47 |
| live tasks in a multi-member `needs` lane | 67 (15%); with `parent` edges 106 (23%) | sazed 9 lanes (max 4), marasi 4 (max 8), tensoon 2; leras's 26-member lane is a `parent` umbrella |
| **Sidney decomposition run on the real stores** (`w = f(place)`, `p ≡ 1`, recursive Dinkelbach over Edmonds–Karp — evidence §A(c)) vs today's ready order | **identical top-8 in all six stores tried; identical whole order in five**; sazed reorders 75 mid-list positions, none in the top 8 | ready sets of 6 / 222 / 23 / 30 / 53 / 9 |
| **priority inheritance** (a ready task takes the best place among its live transitive dependents) vs today's order | surfaces **4 findings**: this repo's parked mirror task mw-cvw8 (900) is a prerequisite of the seq-150 v1 gate; in sazed an unranked task is needed by a seq-72 one, a 190 by a 30, a 710 by a 570 | same ready sets; 0 findings in marasi, tensoon, leras, portfolio |

Four readings, and they decide the shape of everything below:

1. **Every ticket carries policy.** The 2026-08-21 owner steer — *"clear its backlog, smart
   prioritization in place"* — was executed as thirty-five `seq` edits arranged in seven tiers
   (integrity 10–80, spec 90–140, gate 150, trust 160–180, capability 190–290, refactors 350+,
   mirror parked at 900+; `sequence.md` Rung M). Seven category rules would have said the same
   thing, and every task filed since would have inherited its tier instead of sorting last.
   leras exhausted three `seq` neighborhoods within 48 hours of migration (mw-908n9k2); the
   renumber that landed repairs a pressure that exists only because placement has no other
   lever. The snippet's diagnostic applies verbatim: *if `layer='task'` is 90% of decisions,
   your defaults are wrong and every ticket is carrying policy it shouldn't.*
2. **The work is already sliced to session size; the backlog is a parking lot.** A median task
   is worked for twelve minutes and waits eighteen hours; 98% of its life is queue. Recorded
   cost is flat — the spread across categories is minutes — so any rule that orders by cost
   has nothing to order by. What varies is *whether a task will ever be picked up*, and the
   hazard has two regimes: a fast population that closes within two days (about a third of
   everything filed), then a residual one closing at 2–4% per day — a rate at which a task open
   today has roughly even odds of still being open a month from now, longer than any store has
   existed. Whether
   that is a true decreasing hazard or a mixture of will-do and never-do tasks (frailty — the
   evidence doc §B says the data cannot yet tell them apart) **the operational answer is the
   same: age is a triage signal, never a rank boost.** Half the portfolio's live work is in the
   residual regime past two weeks, and 96% of it carries a `seq` — placed, and parked in place.
   The notes' layer 4 (*"sequencing is second-order"*) is not a caveat here; it is the finding.
3. **Structure speaks for a minority, and when it speaks it says something specific.** Eight
   percent of live tasks unblock anything; fifteen percent sit in a dependency lane. Readiness
   (MW-B6) already honors precedence for those. What the graph adds is the *lanes view* the
   refresh session needed (*"you can't just look at next tasks btw there are multiple
   threads"* — §2.1 there) and one sharp finding: **a prerequisite placed later than the work
   it blocks.** Four exist today, and one of them is a contradiction with a ruling — the
   deferred mirror block is what the v1 acceptance gate is waiting on. The full Sidney
   decomposition, run with the declared placement as weight, reproduces today's order and
   surfaces none of them; the human rule *a prerequisite is as urgent as its most urgent
   dependent* surfaces all four in one line each.
4. **The default is wrong-signed.** `coalesce(seq, 999999)` puts a new unsequenced task *below*
   work the owner parked at 900: in this repo's `ready` today, mw-6erbg4a (release notes, no
   seq) sorts after five indefinitely-deferred mirror tasks. In marasi-applied-r-and-d, 21 of
   24 live tasks are unsequenced; the tool has no opinion about any of them.

And the two field failures the baseline does not measure but the record does: the owner opened
three sessions restating priorities the store already carried while a stale `doing` list crowded
the seq-10 flagship out of prime (mw-06j1wqe); across 35 sazed sessions `dep` was used zero times
— *seq was the only lever agents reached for* (mw-jqj9qa9). A session that cannot see *why* a
task is next cannot tell steering from noise, and STATUS §9's hardest-won rule — *"record the
WHY of a priority, not only the priority … an ordering with no recorded rationale will be
re-litigated by every future refresh"* — is the constraint the whole design is built around.

What is *not* a failure: readiness. The precedence half of scheduling has been right in the
field since the pilot. This proposal touches only what happens *among* ready tasks, and what the
digest says about them.

---

## 2. What exists today, precisely

- **Readiness** — frozen SQL (DESIGN §5): open ∧ every `needs` target done/dropped ∧ no live
  child; unresolved cross-repo edges block conservatively (MW-G5).
- **Order** — `ORDER BY coalesce(seq, 999999), created` per repo; portfolio `sort_total`
  (`src/cli/portfolio.rs:85`) is `(sequence.md position, repos.toml rank, seq, created, id)`.
  `next` is row 0; `ready` presents the same order (mw-0vw7nj0); total and deterministic (MW-G4).
- **`seq`** — integers, gaps of 10, midpoint inserts, repo-level renumber on exhaustion (§15.2).
  *"seq IS the priority primitive — there is no priority field"* (DESIGN §7b).
- **`sequence.md`** — the owner's cross-repo overlay: tranche headings, rationale prose, *"Not
  chosen, explicitly"* sections; autopruned on every `portfolio` verb; `dangling-sequence` lint.
- **`prime`** — headline (counts + top-5 category rollup by min seq), weather, addressed asks,
  next (led by `handoff:`), also-ready with blocks-lines, recently done; 6144 bytes, gated.
- **The graph** — `needs` (cycle-linted), `parent`, `discovered-from`, `relates`, `answers`.
- **The log** — every transition with a minute stamp and, since the shim, a session identity.
  Queue time, service time, and per-session task sequences are already derivable by SQL — the
  baseline above is nothing else.
- **The fences** — REQUIREMENTS §3: no estimates, no time tracking, no burndown, no sprint
  semantics, no bespoke query language. DESIGN §6 frozen; one verb added post-freeze by ruling.
  The tool never writes outside the repo and the portfolio repo.

---

## 3. The frame, and what survives contact with the baseline

The notes' claim is that prioritization is a degenerate case of single-machine scheduling and
that ~70 years of results transfer. The evidence doc confirms the theorems (and corrects several
attributions). What matters here is which results still say something once meshwork's
constraints *and the baseline* are applied. Three constraints:

- **There is no `w_j`.** No value field, and DESIGN §7b rules one out. Value enters only through
  position — `seq` and `sequence.md` — which are ordinal, not ratio-scaled.
- **There is no `p_j`, and the measurable proxy is flat.** No estimates (§3), and the refresh
  proposal's rule stands: *mine what enforcement already writes; never ask an agent to log
  effort.* The log yields service time, and service time is twelve minutes with a category
  spread of minutes. The cost that *does* vary — tokens per task — lives in transcripts, not the
  store (the matrix mined it out-of-band: median 102k tokens to first act).
- **One worker class.** One owner plus fungible agent sessions; no skills, no resource types. By
  owner ruling of 2026-08-10 the constrained resource is tokens — *"i only have so many tokens."*

| result | needs | with meshwork's inputs, after the baseline |
|---|---|---|
| **Smith's rule** (`1‖Σw_jC_j`: sort by `w_j/p_j`) | `w`, `p` | With `w ≡ 1` it is SPT — optimal for mean completion time; with `p` a distribution, SEPT. With `w ≡ 1` *and* `p` flat, every order is equally good. **The theory transfers exactly in proportion to the information in `w` and `p`, and the store holds neither today.** Real `p` would be tokens (out of band); real `w` would be a declaration. |
| **Sidney decomposition** (`1|prec|Σw_jC_j`, 2-approx, tight; NP-hard even with unit `w` *or* unit `p` — Lawler 1978) | `w`, `p`, DAG | Vacuous on flat `p` and unit `w`. The estimate-free wiring that *does* work is `w = f(place)`, `p ≡ 1`: the ratio-maximal closure is *the schedulable prefix with the best mean declared priority, counting the prerequisites it drags along* (evidence §A(d)). Run on the real stores it reproduces today's order (§1) — the notes' falsifier *"Sidney blocks ≈ the owner's seq anyway"* fired the cheap way. It needs a per-repo weight scale (`seq` is per-repo; cross-repo closures mix scales), and it cannot be an `ORDER BY`: a max-flow is a Rust pass feeding a column. Rung 6, conditional. |
| **EDD / Moore–Hodgson** (deadlines) | `due` | No `due:` exists. The deadline-shaped class meshwork *has* is the addressed ask; mw-r6g9bhe already puts its age in the headline, as its own tier — the notes' own advice for mixed shapes. |
| **Dispatch rules needing no `w`** (MTS; LFT/MSLK and HEFT need durations) | DAG (+`p`) | HEFT's upward rank and MSLK are *longest weighted path* and *latest-minus-earliest finish* — both need durations the store lacks (evidence §A(d)); with unit costs they collapse to chain depth. Transitive live dependents (`unlock`) and **inherited place** (the best place among live dependents — the priority-inheritance rule from real-time scheduling) are cheap, explainable in one line, and speak for exactly the 8% where structure matters. A finding and a tie-breaker; not a ranking. |
| **Gittins / M-SERPT** (unknown sizes, index over *attained service*) | empirical size distribution, attained-service clock | The theorems are about time *worked*, not time *open*; meshwork's `doing` interval is that clock, and 57% of closes have one. Under a decreasing failure rate the size-blind-optimal policy (foreground-background) favors the *least-served* job — and under the frailty alternative the correct move is to find and drop the never-do mass. Both say: aging is a *decision*, never a boost. The approximation the notes call "SERPT, 2×" is M-SERPT at 5× in the M/G/1 (evidence §B). |
| **Queue law** (`ρ/(1−ρ)`, Little) | `doing` count, admission | 98% of a task's life is waiting. Admission — what is in the backlog at all, and what is `doing` — dominates any ordering of what is there. One headline line, and the triage queue, are the levers. |
| **Sequence-dependent setup** | measured `s_ij` | The notes' numbers (B&B ~40 jobs / 2 h, O(n⁴) and O(n³) heuristics) are real but belong to the *release-times* variant (Chou, Wang & Chang 2009 — evidence §C). The matrix's first measurement: cross-repo switching costs nothing extra (8.8 vs 11.1 min ramp) — the store carries the context. Same- vs cross-category is unmeasured. A tie-breaker at most, after measurement. |
| **Prefix-dependent setup** ("the open problem") | a cost over what the session already holds | Narrower than the notes claim: the prefix-*set* model exists as **tool switching** (Tang & Denardo 1988) — cost `\|T_j \ magazine\|` against a capacity-bounded working memory, optimal-in-polynomial-time *given an order* (KTNS), NP-hard to *sequence*. Mapped here: magazine = the session's context, `T_j` = a task's `docs:` refs and files. The store already knows which tasks a session claimed, hence which `docs:` it has loaded — rung 7's proxy. |
| **Packing** (NRP / RCPSP / CP-SAT) | capacities, `p` in the capacity's unit | No capacities, no skills; a session claims a median of 4 tasks; the ready set is precedence-closed already. And under a context window the problem is **paging, not knapsack** — context is neither additive nor known in advance (evidence §C). CP-SAT is the strongest general solver and RCPSP at n ≥ 100 is *still open* (PSPLIB j120: 89 of 600 proven) — the notes' "trivial in seconds" holds at n≈30, not at 50–300. A sorted list plus lanes *is* the packing. No solver dependency (§9). |
| **Naive greedy by `w/p` among ready tasks** | — | What every tracker does, and what a bare SEPT-within-band would be: **no published guarantee under precedence**, and an unbounded bad example (a cheap prerequisite of a heavy dependent loses to any standalone task). This is why `inherit` precedes any cost term in the key (MW-R6c, MW-R21). |
| **Learned dispatch; LLM-agent DAG schedulers** | `n > 10⁴`; sub-second call graphs | Wrong scale; the notes agree. The agent-orchestration papers (LLMCompiler, Autellix, LLMSched, the "Graph Harness" position paper — evidence §C) schedule LLM *call* graphs at serving latency, a different problem; the one lesson that transfers is *never build a static plan DAG* — which `prime`'s re-derived ready set already honors. |

**Prior art, briefly** (evidence §D, every row from a raw page). No tracker ships cost-of-delay
÷ duration natively; none implements Sidney; *ready* is binary everywhere. The two closest
relatives are exactly the two pieces this proposal keeps: Taskwarrior's `urgency.inherit` — a
blocking task takes the urgency of what it blocks, off by default, with a field record of users
hitting the case where a prerequisite still scores below its dependent — and Beads Viewer, an
agent-oriented sidecar whose whole purpose is to flag *where a human-set priority disagrees with
what the dependency structure implies.* Beads itself, the tracker built for agent sessions,
sorts by `priority, created` and its `hybrid` mode buries anything older than 48 hours — an
accidental decreasing-hazard policy nobody argued for. Manual override wins over every computed
score in every tool surveyed, and the one product that shipped an aging feature (Shortcut)
deprecated it.

So the floor's *order* is, honestly, thin — and that is fine, because the baseline says order is
not where the loss is. The loss is in three places the layers can reach: value is declared
thirty-five times instead of seven; half the backlog is parked without saying so; and the lanes
the graph knows are invisible. The theory's two hard results — the queue law and the hazard
regime — are exactly the ones that address those.

---

## 4. The shape: three layers, first placement wins, the floor is sealed

The second snippet's resolution order, applied to *where a task sits in the order*:

```
sequence.md entry        ← portfolio overlay, absolute                        (exists)
      ↓ absent
task seq:                ← per-task override                                  (exists)
      ↓ absent
category band            ← nearest-ancestor rule: engine/spill → engine → root  [NEW]
      ↓ no rule
floor                    ← computed, total, sealed — always terminates          [NEW]
```

**Placement** is first-match: the first layer that speaks places the task on the `seq` scale.
**Ordering within a placement** is always the floor's. The composite key:

```
key(t) = ( overlay(t),     -- sequence.md position; ∞ when absent          (portfolio only)
           place(t),       -- seq  |  band(category)  |  default band
           -unlock(t),     -- live transitive dependents, descending       (rung 1)
           created(t), id(t) )
                           -- p̂(t) and block(t) enter here ONLY if rungs 5–6 are ever built

placed_by(t) ∈ { overlay, seq, band:<rule>, inherit:<id> (opt-in, MW-R6c), floor }
```

Why this is the snippet's design and not merely a longer sort key:

- **The floor is total and sealed.** Every task gets a key; no layer can remove the floor and no
  task can opt out of it. The only per-task lever is `seq` — recorded, diffable, lint-visible,
  and named in the placement. There is no `skip:`; there is nothing to lint for abuse.
- **Layers only add specificity.** A band or a `seq` moves a task within the order; neither can
  make a blocked task ready, lift a WIP cap, or silence a decay finding. Readiness stays the
  frozen SQL; the safety properties sit below the layers.
- **The placement is recorded.** `prime`'s next block says `placed by: seq 180`, `placed by:
  band core/integrity=10 — "the tool must not misreport the stores it manages"`, or `placed by:
  floor (unlocks 3)`. Every ready row in `--json` carries `placement: {layer, rule, key}`; the
  tasks projection gains `placed_by`, so the health metric is one query:

  ```sql
  SELECT placed_by, count(*) FROM tasks WHERE status='open' GROUP BY 1
  ```

  Today: `seq 399 · floor 58`. The target is the reverse — a handful of `seq` (true
  exceptions), a few dozen `band`, the rest `floor`. If `seq` still carries most placements
  after bands exist, the floor is wrong for this owner and the seq'd tasks say where; if it
  falls to a few percent, the tool is carrying the order and the owner's words live in
  `sequence.md` and band reasons, where they belong. **The number decides the next rung.**
- **The WHY survives.** Rationale lives where it lives today — `sequence.md` prose, `handoff:`,
  comments — plus a one-line `reason` on a band rule. Floor placements are self-explaining by
  construction (a rule name, a number). That is the property the refresh proposal found missing
  in a solver, and it is the reason the parked-in-place failure (§1, reading 2) becomes visible:
  a task at `seq 240` that has not moved in three weeks says nothing; a band at 900 with a reason
  says *parked, and why*.

---

## 5. Requirements (proposed; `MW-R*`, RFC 2119)

### A. Placement layers

- **MW-R1 (MUST)** `docs/meshwork/config.toml` MAY carry a `[bands]` table mapping category
  prefixes to `seq`-scale integers, resolved by **whole-segment nearest ancestor** (MW-B4's
  prefix semantics), plus an optional `default`. Each rule MAY carry a one-line `reason`.
  The 08-21 tiers, as they would be written:

  ```toml
  [bands]
  default = 500                    # unruled, unsequenced work sits above parked, below steered
  "core/integrity" = { band = 10,  reason = "the tool must not corrupt or misreport the stores it manages" }
  "core/format"    = { band = 90,  reason = "spec correctness before capability" }
  "core"           = 200
  "skill"          = 250
  "plan/m3"        = { band = 900, reason = "mirror deferred indefinitely — owner ruling 2026-08-14" }
  ```

- **MW-R2 (MUST)** `place(t)` = `seq` when set, else the nearest matching band, else `default`.
  **Absent a `[bands]` table, `default` is today's 999999**: a store that has not opted in orders
  byte-identically to today (the mw-908n9k2 discipline — nothing observable reorders without a
  declared change).
- **MW-R3 (MUST)** Every ranked task carries `placed_by` in the tasks projection and in every
  `--json` listing that orders (`ready`, `prime`, `portfolio ready/next`); `prime`'s next block
  renders it as one line. Rule names and reasons pass through the same sanitizer as every other
  text seam.
- **MW-R4 (SHOULD)** `lint` warns `band-dead` (a rule no task, live or terminal, matches — the
  snippet's dead-category-rule rot detector), `seq-shadowed` (a `seq` equal to the band the task
  would have received — the override carries nothing), and, **only when `[bands]` exists**,
  `no-category` on live tasks (a band table makes categories load-bearing; today 34% of live
  tasks have none, and in leras 59 of 64). Warnings, never errors.
- **MW-R5 (SHOULD)** The registry-aware lint pass (mw-2nmsys2) gains `sequence-stale`: an
  overlay entry whose task is blocked or unready while a later entry is ready, and a tranche
  heading whose entries have all pruned away (*"this tranche's prose is describing finished
  work"* — refresh-sessions §2.5). Report only.

### B. The floor's graph fields (derived, nothing stored)

- **MW-R6 (MUST)** Derived columns on `tasks`, computed per invocation over live tasks and
  `needs` edges: `unlock` = live tasks that transitively `need` this one; `depth` = longest live
  `needs` chain below it; `inherit` = the smallest `place` among those dependents (the task's
  own `place` when none). Resolved cross-repo edges count; unresolved ones count as nothing
  (MW-G5 — absence is not evidence). `inherit` is same-repo only until a cross-repo scale
  ruling exists — `seq` and bands are per-repo weights.
- **MW-R6b (MUST)** `lint` warns `needs-behind`: a live task whose `inherit` is better than its
  `place` — i.e. a prerequisite placed later than something that needs it — naming both ids and
  both places. It reports a contradiction; it does not resolve one. The four cases in the
  baseline include a deliberate park (the deferred mirror under the v1 gate) and an unranked
  prerequisite of a top-ten task; the owner's fix differs, and only the owner knows which.
- **MW-R6c (SHOULD)** A store MAY opt in to `[bands] inherit = true`, after which `inherit(t)`
  replaces `place(t)` in the key when better, with `placed_by = inherit:<dependent id>`. Off by
  default: promotion by inheritance would have un-parked mw-cvw8 against a ruling. This is
  Taskwarrior's `urgency.inherit` (also off by default) with the placement recorded — the one
  graph-propagation rule any surveyed tracker ships (evidence §D).
- **MW-R7 (MUST)** `lane` = the connected component of the live graph over `needs` **only** —
  the baseline shows `parent` umbrellas join everything (leras: one 26-member component) — with
  the lane's head = its member with the earliest key. Lanes are what the refresh proposal called
  threads; the graph had them all along.
- **MW-R8 (MAY)** Articulation points and betweenness over the live `needs` graph as `q`-able
  columns; **never a ranking input**. On a graph this sparse nearly every interior node of a
  chain is a cut vertex; the atlas computes both over its concept graph (Q7) where they mean
  more. petgraph already provides Tarjan's articulation points (evidence doc §E).

### C. History-derived statistics (measured, never authored)

- **MW-R9 (MUST)** Service time (`doing→done` when a start was logged), queue time
  (`created→doing`), and age are `q`-able derived columns, computed from the `log` table per
  invocation, never persisted, never displayed as a forecast. **Nothing is authored and nothing
  is asked of any session** — this is the setup-cost matrix's method moved from a script into
  the projection.
- **MW-R10 (MUST)** The tool fits the **discrete daily close hazard** from its own history
  every run: for each age bin, among tasks open at that age *and observable through the end of
  the bin*, the share closed `done` inside it, with `dropped` as a competing event reported
  beside it — never the cumulative "ever closed" figure the setup-cost matrix printed, which
  the evidence doc shows is confounded with the store's own age. The table is `q`-able and one
  block of `lint --stats`. The **triage age** is configured (`[decay] triage_days`, default
  14), not fitted: the fit informs the choice, the owner makes it.
- **MW-R11 (MAY)** Category-conditional expected *remaining* service for `doing` tasks, on the
  attained-service clock (Σ time in `doing`), from a Kaplan–Meier fit over closed and
  still-open tasks — the M-SERPT quantity — surfaces as a column. It enters the order only
  under rung 5.

### D. Aging and WIP (decisions, not boosts)

- **MW-R12 (MUST)** Age NEVER raises a task's position, and the floor adds no anti-starvation
  term. Both live readings of the measured hazard agree: under a decreasing failure rate the
  size-blind-optimal policy favors the least-served job, and under frailty the old mass is the
  never-do mass — neither says *boost the old one*. A category that must not starve gets a
  band; starvation is policy, never automatic.
- **MW-R13 (MUST)** Open tasks past the triage age are a **decision queue, rolled up by
  category**, because at the baseline's scale (sazed: 146) a list of ids is not a decision
  surface and a band rule is. `prime` carries one line — `146 of 239 open past 14 d (residual
  close rate 3%/day): (none) 28 · method 15 · housekeeping 14 · +N — band, drop, or re-scope`
  — byte-capped like every section, and `lint` warns `open-decayed` per task. The three verbs
  named are the atlas's *assigned / struck / answered*; the tool never picks one. This is the
  one action both hazard hypotheses recommend (evidence §B): forced triage removes what
  foreground-background would deprioritize anyway, and it removes the never-do mass that would
  be generating the apparent decrease. Tasks carrying a configured exempt label (default
  `owner`) are excluded — the mw-59f0t1q shape: *"stays open as his marker, no agent action
  pending"* — so a marker is never nagged and never confused with rot.
- **MW-R14 (SHOULD)** `[wip] cap = N` in config; when set, the headline's first line reads
  `doing 10 (cap 3 — OVER)` and `lint` warns `wip-over-cap`. Nothing is blocked by it — a cap
  that refused `start` would be enforcement the tool does not do — but the digest says it
  before it says anything about order, because 98%-waiting says it matters more.

### E. Lanes in the digest

- **MW-R15 (SHOULD)** When a store has more than one multi-member lane, `prime` renders a
  `lanes` section — one line per lane, top 5 by the head's key: head id, member count, status
  mix, and `EXTERNALLY BLOCKED` when any member `needs` an unresolved or foreign-open task;
  singleton lanes collapse to a count. Byte-capped; also-ready shrinks to make room. **Surface
  ruling required** (§10): this is the one addition that changes what a session sees first.

### F. Explainability

- **MW-R16 (MUST)** `why <id>` additionally prints the task's placement and lane when the task
  is ready — the frontier of blockers was half the answer to *why is this not next*; the
  placement is the other half. No new verb.
- **MW-R17 (MUST)** Every derived quantity above is a column in the §4 projection, documented in
  FORMAT.md with its derivation, so the whole floor is auditable with `q` and reproducible by a
  third-party reader from the tables — the contract every other derived view honors.

### G. Fences (what this proposal does not do)

- **MW-R18 (MUST)** No authored value field (`weight`, `cod`, `due`). If the placed-by histogram
  after rung 2 still shows `seq` carrying most placements, that is the evidence to bring to a
  ruling on one; it is not assumed here.
- **MW-R19 (MUST)** No solver dependency, no capacity model, no session planner (§9).
- **MW-R20 (MUST)** No new verb in rungs 0–3. Rung 4's `lanes` section, `lint --stats`, and any
  `rank --explain` spelling need a §6 ruling and are not assumed.
- **MW-R21 (MUST)** Cost- and density-based ordering (SEPT, SERPT-in-key, Sidney blocks) is
  **conditional** on rung 0 finding a cost signal that varies — in tokens, since recorded time
  does not — or on a value field being ruled in. Until then the columns may exist; the key does
  not use them. When a cost term does enter, it enters **after** placement and inheritance,
  never as a bare greedy over the ready set: greedy `w/p` under precedence has no guarantee
  and an unbounded bad example (evidence §C, claim 11).

---

## 6. Surfaces — where it lands without touching §6

| surface | change | fence |
|---|---|---|
| `tasks` projection | derived columns `placed_by`, `place`, `unlock`, `depth`, `lane`, `service_h`, `queue_h`, `age_h`; FORMAT.md §Projection documents each with its derivation | MW-C1: SQL stays the only query surface; readers implement from FORMAT.md |
| `config.toml` | `[bands]` (rules, `default`, `inherit`), `[wip] cap`, `[decay]` (`triage_days`, `exempt_labels`) tables; unknown tables warn as today | data, never executed; no trust-boundary change |
| `ready`, `portfolio ready/next` | order key per §4; `--json` rows gain `placement` | envelope `schema` bumps (MW-C3) |
| `prime` | WIP line when capped; `placed by:` in next; triage rollup line; `lanes` section (ruling) | 6144 bytes unchanged and gated; every new line byte-clamped, loud when cut |
| `why` | placement, inheritance and lane appended for ready tasks | existing verb, additive |
| `lint` | `needs-behind`, `band-dead`, `seq-shadowed`, `no-category` (bands only), `sequence-stale`, `open-decayed`, `wip-over-cap` — warnings all; `lint --stats` prints the hazard table and the placed-by histogram | existing verb; `--stats` is a flag, needs the nod `--approve` got |
| perf | every derived field is O(n + m) over live tasks; the hazard fit is one pass over `log`; benched under MW-C4 (`ready` ≤ 100 ms @ 1K) | `scripts/check-perf.sh` drift wall |

REQUIREMENTS §3 needs two scope rulings, in the style of the three already recorded there:

- **Lifecycle statistics derived from the log are NOT the rejected time tracking / estimates /
  burndown.** Those fences ban things an agent is *asked to write* and charts a human is asked
  to read. Nothing here is authored, nothing is asked, nothing is forecast; the tool mines the
  transitions the lifecycle already writes — the precedent is `docs/setup-cost-matrix.md`, which
  the refresh proposal called *the only process numbers that mattered*.
- **Ordering the ready set for one session is NOT the rejected sprint semantics.** No capacity,
  no velocity, no ceremony, no iteration boundary; the unit of work is whatever a session claims.

---

## 7. Build ladder (every rung independently useful; red first; each with its verify)

The owner's design law for tranches — *put the falsifier before the investment* — applies to the
tool's own features. Rung 0 has already run once (§1); landing it as a script makes it re-run.

| rung | lands | verify | gated on |
|---|---|---|---|
| **0 — measure** | `scripts/mine_rank.py` — **landed beside this proposal, untracked**; read-only, the matrix's sibling; per-repo `q`, never a portfolio verb: the §1 table per repo, the hazard table, lanes, and the Sidney/inheritance experiment; evidence §F is its first output. Re-runs land as `docs/rank-baseline.md` with every denominator. **Plus the one number the store cannot give:** tokens per task by category, from transcripts, by the matrix's method — does it vary by ≥ 3× across categories with n ≥ 7? | `python3 scripts/mine_rank.py --check` exit 0 (observed 2026-09-01, nine stores); the token half is its own script with its own `--check` | nothing |
| **1 — graph fields** | `unlock`, `depth`, `inherit`, `lane` columns; `needs-behind` lint; `placed_by` under today's rules (`seq` / `floor`); FORMAT.md projection rows; `why` prints lane, placement and inheritance | `cargo test tables::derived_rank_columns` + `cargo test lint::needs_behind` (fixture: a parked prerequisite under a steered dependent) | — |
| **2 — bands + placement** | `[bands]` parsing, nearest-ancestor resolution, `default`; the §4 key; `placed by:` in prime; `placement` in JSON; `band-dead`, `seq-shadowed`, `no-category`, `sequence-stale` lint; the skill's ritual gains *express tiers as bands, keep seq for exceptions* and the migration note for stores that have tiered by seq | `cargo test e2e::ready_bands_placement` + prime golden re-blessed with a reviewed diff | 1 |
| **3 — decay + WIP** | discrete hazard table from `log` (competing risks, equalized follow-up); `service_h`/`queue_h`/`age_h` columns; `[decay] triage_days` + `exempt_labels`; `open-decayed` lint; the triage rollup line in prime; `[wip] cap` and the headline line | `cargo test rank::hazard_fit` (fixture store with a known survival curve, including a censored tail the fit must not count) + prime golden | 2 |
| **4 — lanes in the digest** | `lanes` section, `lint --stats` | `cargo test e2e::prime_lanes` + golden | §6 ruling |
| **5 — measured cost (conditional)** | if rung 0's token measurement varies: `p̂` per category (tokens, from the matrix's out-of-band mining, committed as data the way `bench-baseline.json` is), SEPT within band, SERPT for `doing`; the key gains `p̂(t)` | `cargo test rank::sept_within_band` | rung 0 result; **not built otherwise** |
| **6 — Sidney blocks (conditional)** | recursive Dinkelbach over live `needs` (petgraph Dinic's, already in the tree — evidence §E; the `needs` rows are already in the required arc orientation) with `w = f(place)` and `p ≡ 1`, or `p̂` from rung 5; `block` enters the key; benched | `cargo test rank::sidney_blocks` + `check-perf.sh` green @ 1K | rung 0 re-run after bands exist showing block order differs from the rung-2 order on real stores — **today it does not** (§1) — or rung 5, or an MW-R18 ruling |
| **7 — setup-cost tie-breaker** | extend `mine_setup_cost.py` with same- vs cross-category ramp from the store's own claim stamps; if the gap is real, *cohesion* breaks ties within the next 3: prefer the ready task whose `docs:` refs and category overlap the tasks this session has already claimed (the log's `claimed by` lines name the session) — the tool-switching cost `\|T_j \ magazine\|` computed from what the store already records, evidence §C | matrix regenerated; `cargo test rank::cohesion_tiebreak` only if the measurement supports it | 3, measurement |

Not on the ladder, deliberately: authored value fields (MW-R18), a solver (MW-R19), scheduling
over the atlas's graph (not a DAG — Q8/Q9; its condensation is a different project), learned
dispatch, and in-store token telemetry (tokens live in transcripts; the matrix mines them
out-of-band and that stays a script, committed as data if rung 5 ever needs it).

---

## 8. Falsifiers

| falsifier | cost to test | consequence |
|---|---|---|
| Tokens per task do not vary by category (< 3× across n ≥ 7 categories) | rung 0's transcript half | rungs 5–6 are never built; `p̂` stays an out-of-band curiosity; the ladder ends at 4 (+7) |
| After bands exist, `seq` still places > 50% of live tasks two weeks on | one query | the floor is wrong for this owner; bring the histogram and the seq'd tasks' comments to a ruling on an authored value field (MW-R18's trigger) |
| After bands exist, `seq` places < 5% and `sequence.md` keeps its shape | same | the design holds; `set --seq` leaves the skill's daily ritual |
| The triage rollup is ignored for a month (past-triage share only grows) | one query | it is read as a nag, not a decision; demote to `lint` only and let `prime` stay quiet |
| The residual hazard is not flat once the stores are older (a real decrease past day 15, or a rise) | rung 3's table after 60 days of store age | the triage age moves with it — it is a config value, not a theorem; nothing else changes |
| Category coverage stays low in the stores that need bands most (leras 5/64) | rung 2's `no-category` count after two weeks | bands cannot place what has no category; the fix is the skill's `add` ritual, not the tool — and the floor's `default` is doing the placing meanwhile, visibly |
| Lanes stay a single giant component even on `needs` only | rung 1 | collapse the section to the multi-member count and drop the per-lane lines |
| `needs-behind` findings are always resolved the same way (the prerequisite gets the dependent's place) | count after a month of rung 1 | flip `[bands] inherit` to default-on; the finding becomes the placement and says so |
| Sidney's order differs from the rung-2 order once bands exist (re-run rung 0) | one script run | rung 6 is built; if not, it is struck from the ladder with this table as the record |
| Rung 2's order surprises the owner on a real store (he re-seqs > 10 tasks in the first week) | observe | the `default` band or a rule is mis-set; the placements name which; adjust the rule, not the tasks |
| `ready` at 1K tasks misses 100 ms with rung 6 on | bench | rung 6 runs under `portfolio`/`--json` only, or not at all |

---

## 9. What this reverses, and why

- **graph-of-graphs §5.4 (`pack --tokens`) and the notes' step 5 (CP-SAT).** Packing is a
  knapsack only when there are capacities to pack against; there are none. A session claims a
  median of 4 tasks from a precedence-closed ready set; a sorted list plus lanes is the plan.
  And a context window is not a knapsack capacity but a *magazine*: the published model is
  tool switching, where the order is NP-hard and the eviction is a polynomial greedy given the
  order (evidence §C) — which argues for exactly a cheap ordering heuristic, not a solver.
  A solver dependency (every CP-SAT/HiGHS/CBC path drags a C++ toolchain; the only pure-Rust
  MILP is `microlp` — evidence §E) would be paid for an empty constraint set. The notes'
  "trivial for CP-SAT at n≈50–300" is also not so: PSPLIB's 120-activity set has 89 of 600
  instances proven optimal after three decades.
- **graph-of-graphs §5.2's *"this is a weekend"* on Sidney, and rung 3 of its ladder.** The
  algorithm is a weekend and the pieces are in the tree; its *inputs* are the work. With unit
  `w` and flat `p` the decomposition is vacuous, and the baseline says `p` is flat in the only
  unit the store records. The one estimate-free wiring that is not vacuous — declared place as
  weight — was run on every store and reproduced the order already there. Rung 6, behind a
  re-run once bands widen the weight's dynamic range. Its cousin, priority inheritance, earns
  rung 1 on the same experiment: it found the four cases that matter, in one line each.
- **The notes' step 4 (*"modulate `p_j` with SERPT from your empirical cycle-time
  distribution"*).** The empirical distribution exists and was fit; it says service is twelve
  minutes and queue is everything. SERPT stays a column (MW-R11) and enters the order only if
  tokens vary.
- **The notes' aging term (*"add an aging term so nothing starves"*).** The measured hazard
  points the other way under either reading (decreasing failure rate, or a will-do/never-do
  mixture); MW-R12/R13 follow the measurement and route starvation to policy (bands) and to
  decisions (triage). The notes' *"deprioritize aging tickets"* is also softened: the theorem
  is on the attained-service clock, and demoting by wall-clock age would be a heuristic wearing
  a citation — so the floor does neither; it just never boosts.
- **The refresh proposal's rejection of auto-ranking.** Kept in substance: nothing ranks
  silently, the authored layers are untouched and win, every placement names its reason. What
  changes is that the *absence* of a ruling gets a defensible, visible default instead of
  *last* — and the residual half of the backlog gets a word for what it is.

---

## 10. Rulings requested

1. **Bands as the tier mechanism.** Is *declare the tier once per category, keep `seq` for
   exceptions* the right expression of what the 08-21 tiering did by hand? Changes the skill's
   ritual; changes nothing in any store until it adds `[bands]`.
2. **Where bands live.** Per-repo `config.toml` only (proposed), or also a cross-repo table in
   the portfolio repo? `sequence.md` stays the only cross-repo override either way.
3. **Two §3 scope rulings** (§6 above): log-derived lifecycle statistics are not estimates;
   ready-set ordering is not sprint semantics.
4. **The `default` band once a store opts in** — 500 (above parked, below steered) is proposed;
   today's 999999 stays for stores that have not.
5. **Aging posture.** Confirm MW-R12/R13: past-triage-age tasks are a decision queue rolled up
   by category, never boosted, never auto-dropped; `owner`-labelled markers exempt; the triage
   age is a config value (14 d proposed) informed by the fitted table, not derived from it.
6. **Value stays unit until measured otherwise** (MW-R18) — or rule now on an authored field.
   Proposed: not now; rung 2's histogram is the evidence, and the baseline says the parked half
   of the backlog is the first thing a value field would have to explain.
7. **Surfaces**: `placed by:`, the WIP line and the triage rollup inside prime's existing
   sections (proposed as behavior, no ruling); the `lanes` section (rung 4) and `lint --stats`
   need a nod.
7b. **Inheritance as placement.** `needs-behind` as a finding is proposed as behavior; letting it
   *place* (`[bands] inherit = true`) is opt-in per store and defaults off — confirm, or rule it
   on. The baseline's own contradiction (mw-v4ej at 150 waiting on the parked mirror) wants a
   ruling of its own either way: re-seq the gate behind the park, or un-park.
8. **The token half of rung 0.** It reads transcripts, as the matrix did — outside the tool,
   inside this repo's `scripts/`. Authorize it as the gate on rungs 5–6, or strike rungs 5–6 now.
9. **Filing.** Rungs 0–3 as tasks now, or after the ruling?

---

## 11. Smallest honest slice

Rung 0 is already run once (§1) and costs a script to make repeatable. Rungs 1–3 are the
proposal's core: the floor exists, bands replace tier-by-seq, every placement is named, the
histogram starts counting, the residual half of the backlog is named as such by category, and the
digest says the WIP number first. Nothing after rung 3 is built on this proposal's say-so; each
later rung is gated on a number the earlier rungs produce.
