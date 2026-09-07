# PROPOSAL — prioritization: evidence companion

Companion to `PROPOSAL-prioritization.md`. Written 2026-09-01. Five research passes ran in
parallel against primary sources (arXiv abstract pages and PDFs through `pdftotext`, publisher
and author-hosted PDFs, raw GitHub files, crates.io and docs.rs pages, DataFusion's tagged
source tree), plus a read-only baseline over the nine registered stores. **Method rule for every
verdict below: a fetched summary is a lead, never evidence.** Each CONFIRMED / CORRECTED row
quotes the raw artifact; anything that could not be read in the raw is marked UNVERIFIED and
names what was checked. The informal notes being verified are `portfolio/task-prioritization-
notes.md` (the first snippet) — cited here as *the notes*.

Sections: **A** precedence layer · **B** uncertainty layer · **C** packing, setup cost, agent
DAGs · **D** prior art in trackers · **E** Rust building blocks · **F** the baseline (method
and tables) · **G** local sources read.

*Provenance of the research passes.* Five subagents ran against primary sources; their raw
fetches are under the session scratchpad (`research-A/` … `research-E/`). Two passes issued
corrections to their own first reports — §B's, which had written unfetched citations and then
withdrew them (noted at the end of §B(a)), and §C's, which first marked the setup-cost numbers
"not found" and then located their source. Only the re-verified versions appear here.

---

## A. The precedence-graph layer

### A(a) Verdicts

| # | claim (the notes) | verdict | citation | verbatim support |
|---|---|---|---|---|
| 1 | Smith's rule: sort by `w_j/p_j` is optimal for `1‖Σw_jC_j` | **CONFIRMED**, strengthened to *iff* | W. E. Smith, *Naval Res. Logist. Quart.* 3:59–66, 1956; statement via Lawler, Lenstra, Rinnooy Kan & Shmoys, *Elements of Scheduling*, Thm 4.1, arXiv:2001.06005 | "A sequence is optimal for 1‖∑w_jC_j **if and only if** it places the jobs in order of nonincreasing ratios w_j/p_j." |
| 2 | Lenstra & Rinnooy Kan 1978: `1|prec|Σw_jC_j` strongly NP-hard | **CORRECTED** — attribution incomplete | Lenstra & Rinnooy Kan, *Oper. Res.* 26:22–35, 1978 **and** Lawler, *Ann. Discrete Math.* 2:75–90, 1978; via *Elements* §4.4 notes | "Lawler (1978) showed that 1\|prec\|∑w_jC_j is NP-hard, **even if either all w_j = 1 or all p_j = 1**." |
| 3 | Sidney 1975 decomposition; any consistent order is 2-approx | **CONFIRMED**; the 2-approx is due to *both* Chekuri & Motwani 1999 **and** Margot, Queyranne & Wang 2003, independently | Chekuri & Motwani, *Discrete Appl. Math.* 98:29–38, 1999; MQW, *Oper. Res.* 51:981–992, 2003; via Correa & Schulz, *Math. OR* 30(4):1005–1021, 2005 (dii.uchile.cl/~jcorrea/papers/Journals/CS2005.pdf) | "every feasible schedule that is consistent with a Sidney decomposition is a 2-approximation for the scheduling problem (Chekuri and Motwani [4] and Margot et al. [17])" |
| 4a | Picard 1976 max-closure via min-cut | **CONFIRMED** | Picard, *Management Sci.* 22:1268–1272, 1976; via Hochbaum, *Oper. Res.* pseudoflow paper (hochbaum.ieor.berkeley.edu/html/pub/Hochbaum-OR.pdf) | "A(s) contains arcs from s to all nodes of positive weight with capacity equal to that weight, and A(t) contains arcs from all nodes of negative weights to t" |
| 4b | initial set = closure in the reversed graph | **CONFIRMED** | *Elements of Scheduling* §4.7 | "an initial set is a closure in the directed graph defined by **reversing all precedence constraints**" |
| 4c | parametric max-flow sweeps λ | **CORRECTED** — Gallo, Grigoriadis & Tarjan 1989, not Lawler | GGT, *SIAM J. Comput.* 18:30–55, 1989; via Chekuri & Motwani, Stanford CS-TN-97-58, Lemma 4 | "a result of Gallo et al. [6] shows that it is possible to obtain all the such values of λ in the time it takes to do one maximum flow computation using the push-relabel algorithm" |
| 4d | a simple λ search suffices | **CONFIRMED** | same | "The algorithm to compute G_λ using a maximum flow computation is present in Lawler's book [13] where a **binary search** is performed to find the optimum λ." |
| 4e | series-parallel precedence is exact in polynomial time | **CONFIRMED** | Lawler 1978; via *Elements* §4.3 notes | "Lawler (1978) presented the **O(n log n)** algorithm for series-parallel precedence constraints." |
| 5a | Correa & Schulz: every known 2-approx is Sidney-consistent | **CONFIRMED** with a wording caveat | Correa & Schulz 2005 | body: "all known 2-approximation algorithms follow Sidney's decomposition"; abstract: "**virtually** all" |
| 5b | Ambühl & Mastrolilli: vertex-cover connection | **CONFIRMED** | *Algorithmica* 53(4):488–503, 2009 | via Ambühl, Mastrolilli, Mutsanas & Svensson: "Ambühl & Mastrolilli [2] settled an open problem first raised by Chudak & Hochbaum [7] and whose answer was subsequently conjectured by Correa & Schulz [8]." |
| 5c | Bansal & Khot: no 2−ε under a UGC variant, *even with unit processing times* | **CONFIRMED** for 2−ε; **UNVERIFIED** for the unit-`p` qualifier | Bansal & Khot, FOCS 2009, pp. 453–462; via Sitters & Yang arXiv:1706.07604 | "Bansal and Khot [2] showed that no (2−ε)-approximation algorithm exists assuming a variant of the unique games conjecture is true." No source read attaches "unit processing times" to the single-machine result. |
| 6a | with release dates: Skutella √e/(√e−1) ≈ 2.542 | **CONFIRMED** (2016, not the 1997 e+ε result) | Skutella, *Oper. Res. Lett.* 44:676–679, 2016, arXiv:1603.04690 | "Recently, Skutella [18] improved the ratio to √e/(√e − 1) < 2.542 and conjectured that a (2 + ε)-approximation algorithm exists." |
| 6b | Sitters & Yang (2+ε) | **CONFIRMED** | arXiv:1706.07604 | "We give a (2 + ε)-approximation algorithm for minimizing total weighted completion time on a single machine under release time and precedence constraints." |
| 7 | HEFT upward rank formula | **CORRECTED** — terms are *averages*; base case is not zero | Topcuoglu, Hariri & Wu, *IEEE TPDS* 13(3):260–274, 2002, Eq. (8) | "c̄_{i,j} is the average communication cost of edge (i, j), and **w̄_i is the average computation cost** of task n_i"; `rank_u(n_exit) = w̄_exit`; list sorted by "**nonincreasing** order of rank_u". |
| 8 | Kolisch: LFT the most robust single rule; MTS, EFT, MTSPT strong | **CORRECTED** three ways | Kolisch, *EJOR* 90(2):320–333, 1996a (the comparison); Kolisch, *J. Oper. Manag.* 14(3):179–192, 1996b (proposes **WCS**); LFT is Davis & Patterson, *Management Sci.* 21(8):944–955, 1975 | Kolisch & Hartmann 1999: Kolisch (1996b) "developed, amongst other priority rules, the so-called **worst case slack (WCS)** rule". "MTSPT" and "EFT" appear nowhere in the 1999 or 2006 surveys. Kolisch & Hartmann 2006: priority-rule methods "are inferior to metaheuristic approaches which are capable of learning." |
| 9 | total float = LS − ES; "MinSlack" dispatch | **CONFIRMED** identity; rule name **CORRECTED** to MSLK | Kelley & Walker, *Proc. Eastern Joint Computer Conf.* 1959, p. 163; Kolisch & Hartmann 1999, Table 3 | "Total Float = t_j^(1) − t_i^(0) − y_ij"; "LST (latest start time), **MSLK (minimum slack)**, MTS (most total successors)…" |
| 10a | DRank: weighted PageRank over requirement dependencies | **CONFIRMED** — 2017, not 2014 | Shao, Peng, Lai & Wang, *J. Syst. Softw.* 126:141–156, 2017, doi:10.1016/j.jss.2016.09.043 | "an algorithm based on the **weighted PageRank** is proposed to analyze the dependencies between requirements" |
| 10b | Achimugu et al. modified PageRank | **CORRECTED — false** | Achimugu, Selamat, Ibrahim & Mahrin, *Inf. Softw. Technol.* 56(6):568–585, 2014 | A systematic literature review; full-text search: zero hits for "pagerank", "eigenvector", "random walk". |
| 11 | no tracker implements Sidney | **NOT FOUND** in named sources (see A(b)) | — | — |

### A(b) Corrections, in one place

1. Lawler 1978 proved the hardness independently, and his version is the one that matters
   here: NP-hard *even when all w_j = 1 or all p_j = 1*. There is no exact shortcut in the
   estimate-free model.
2. The 2-approximation belongs to Chekuri & Motwani (1999) *and* MQW (2003); Correa & Schulz's
   contribution is the *"virtually all known 2-approximations are Sidney-consistent"* result.
3. Capacities are not symmetric as the notes wrote them: source→j carries the weight term,
   j→sink carries the processing-time term, and the infinite arcs run **from each job to its
   prerequisites**.
4. Lawler's binary search over λ is in his 1976 book; the parametric sweep is GGT 1989; Lawler
   1978 is the series-parallel exact algorithm.
5. HEFT uses *mean* costs across processors and a non-zero base case. With no durations it
   collapses to chain depth.
6. Kolisch's rule comparison is the EJOR paper; his JOM paper introduces WCS; LFT is Davis &
   Patterson's. The best rule flips with instance size (serial + LFT on J30; parallel SGS on
   J60/J120).
7. The dispatch rule is **MSLK**, not "MinSlack".
8. DRank is Shao et al. 2017; "reversed" graph is unsupported by anything read; Achimugu 2014 is
   a review.
9. The unit-processing-time qualifier on Bansal–Khot is unverified. Woeginger's hard instances
   put each job at *either* (p=1, w=0) *or* (p=0, w=1); Li (arXiv:1707.08039) applies the bound
   to *parallel* machines.

**Claim 11 — searches run.** GitHub code search: `"Sidney decomposition"`,
`"Sidney-consistent"`, `sidney "1|prec|"` → BibTeX entries and one design document
(`holmesworcester/poc-17`), no implementation. crates.io: `pseudoflow`, `hochbaum`, `sidney` →
nothing. PyPI full index: no `sidney`, `mincut`, `rcpsp`. **What exists:** PyPI `pseudoflow`
2022.12.0 ("Pseudoflow algorithm for the parametric minimum cut problem") and `closure-problem`
2020.5.2, both from Hochbaum's group — Python/C. Trackers stop at CPM or hand-set integers:
Taskwarrior's `urgency_blocked()` / `urgency_blocking()` return literal 1.0/0.0 (no graph
propagation); TaskJuggler uses a hand-set 0–1000 priority; GanttProject and ProjectLibre
implement CPM; Jira orders by LexoRank. Not settled either way: paperswithcode is retired and
Semantic Scholar rate-limited, so code links for the four papers were neither found nor ruled out.

### A(c) Implementation sketch — ratio-maximal closure

**Problem.** Tasks N with `w_j ≥ 0`, `p_j > 0`, precedence i→j ("i before j"). Find the initial
set I (closed under predecessors) maximizing `ρ(I) = w(I)/p(I)`.

**Graph, fixed λ.** `a_j = w_j − λ·p_j`. Nodes N ∪ {s, t}: `s→j` capacity `a_j` when positive;
`j→t` capacity `−a_j` when negative; **`j→i` capacity ∞ for every precedence i→j** — the arc
runs from a task to each prerequisite, so a finite cut cannot leave a prerequisite on the sink
side while its dependent is on the source side. **meshwork's `edges` already stores `needs` as
dependent→prerequisite; the infinite arcs are the `needs` rows verbatim, no reversal pass.**

**Solve.** Max-flow; source side of the min cut minus {s} is the max-weight closed set I_λ, with
`g(λ) = Σ_{a_j>0} a_j − mincut`, convex, piecewise-linear, non-increasing, `g ≥ 0`.

**λ search — Dinkelbach / discrete Newton (recommended at this scale).**

```
λ ← w(N)/p(N)                   // N is initial: a valid lower bound
loop:
    I ← max_closure(λ)
    if I = ∅: return (λ, previous I)
    λ' ← w(I)/p(I)
    if λ' ≤ λ: return (λ, I)    // fixpoint
    λ ← λ'
```

λ strictly increases, so it terminates; one max-flow per round. Compare ratios by
cross-multiplication, not floating λ.

**Full chain.** Recursive: compute I₁, recurse on N \ I₁. Or one parametric sweep (Chekuri &
Motwani's orientation `s→j = λ·w_j`, `j→t = p_j`; GGT nested cuts, ≤ n−1 breakpoints). Coarsest
(Correa & Schulz's *reduced*) vs finest (MQW) decompositions both carry the 2× guarantee.

**At n ≤ 1000, m ≤ 5000: skip GGT.** Recursive Dinkelbach over plain Dinic
(`petgraph::algo::maximum_flow::dinics`, in the tree — §E), cut by residual BFS from s. A
pathological 1000 blocks × 10 Newton steps is 10⁴ max-flows on a 6k-arc graph — under a second.
MQW report instances up to 2,000 jobs; the portfolio is under 500 live.

### A(d) Relevance to meshwork (the report's own reading)

- With `p_j = 1`, Smith's rule is "sort by weight" — which `seq` already is. The theory adds
  nothing until precedence enters.
- With unit `w` **and** unit `p`, `ρ(I) = 1` for every initial set: the decomposition is
  vacuous and the 2× guarantee trivially true. At least one of `w`, `p` must vary.
- **The obvious estimate-free wiring is `w_j = f(seq)`, `p_j = 1`.** Then `ρ(I)` is the mean
  declared priority of a closure: *"the legally-schedulable prefix with the best average
  priority, counting the prerequisites you are forced to drag along."* A seq-10 task behind
  five seq-500 prerequisites correctly loses to a ready seq-100 task. The argmax is invariant
  under positive affine maps of the weight; only `w_j ≥ 0` is required.
- **`seq` is per-repo; cross-repo closures mix incommensurable scales** — a normalization
  ruling is required before a portfolio-wide run.
- The 2× bound leans on the fixed cost `Σ w_j p_j`; Uhan (via Ambühl et al.) gives an instance
  where any Sidney-consistent order is arbitrarily bad for the *variable* cost.
- A max-flow is not expressible in DataFusion SQL: this is a Rust pass feeding a column, never
  an `ORDER BY`.
- HEFT and MSLK need durations the store lacks; the ratio-closure route is the only one of the
  four families that survives having no `p_j`.
- Cheapest experiment: `p = 1`, `w = max_seq − seq`, single repo, diff against `ready`'s top-10.
  **Run in §F: the orders agree.**

---

## B. The uncertainty layer

### B(a) Verdicts

| # | claim (the notes) | verdict | citation | verbatim support |
|---|---|---|---|---|
| 1 | Gittins is optimal for mean response time in the M/G/1 with unknown sizes; the index depends only on age | **CONFIRMED**; citation **CORRECTED** — cite by role | Scully & Harchol-Balter, *The Gittins Policy in the M/G/1 Queue*, WiOpt 2021 (optimality proof); Scully & Harchol-Balter, *SOAP Bubbles*, Allerton 2018 (index/age); Aalto, Ayesta & Righter, *Queueing Syst.* 63:437–458, 2009 (characterization in the M/G/1); Gittins 1979 *JRSS-B* (the index, for bandits) | "A job's Gittins index depends only on its age, the amount time [sic] the job has been served so far." 2021: despite being "widely accepted in the literature", "there is no complete proof of Gittins's optimality in its full generality" before it. |
| 2 | SERPT is a 2-approximation | **CORRECTED** three ways | Scully, Harchol-Balter & Scheller-Wolf, *Simple Near-Optimal Scheduling for the M/G/1*, POMACS 4(1), 2020, doi:10.1145/3379477; Scully, Grosof & Harchol-Balter, *Perf. Eval.* 145:102150, 2021 | "We prove the mean response time ratio between M-SERPT and Gittins is at most 3 for load ρ ≤ 8/9 and at most 5 for any load." It is **M-SERPT** (monotonic), the M/G/1 factor is **5**, the **2** is the heavy-traffic M/G/k result. Vanilla SERPT: "whether SERPT has a constant-factor approximation ratio remains an open problem." |
| 3 | vanilla Gittins is not robust to age noise; shift-flat fixes it for any SOAP policy | **CONFIRMED** — 2018, Allerton | Scully & Harchol-Balter, *SOAP Bubbles: Robust Scheduling Under Adversarial Noise*, Allerton 2018 | "introducing even an infinitesimal amount of noise in job ages can cause a large jump in mean response time"; "at load ρ = 0.9 … by over 40%". Decoy to avoid: Moseley et al., *Robust Gittins*, arXiv:2504.10743, 2025 — about imperfect *predicted distributions*, not noisy ages. |
| 4 | Gittins on the empirical size distribution is provably near-optimal | **CONFIRMED** | Ramakrishna, Harlev & Scully, *Empirical Gittins for Data-Driven M/G/1 Scheduling with Arbitrary Job Size Distributions*, POMACS 10(1), 2026, doi:10.1145/3788091 | "it suffices to simply apply the Gittins construction to the empirical distribution of the job size samples… we prove an explicit high-probability bound." |
| 5 | decreasing hazard → deprioritize aging; increasing → grind | **CONFIRMED**, sharpened | Righter & Shanthikumar, *PEIS* 3:323–333, 1989; via Nuyens, *The Foreground-Background queue: a survey*, arXiv:math/0412182, Thm 3.1 | "If the service-time distribution belongs to the class DFR, then for every t ≥ 0, Q_FB(t) ≤st Q_π(t) ≤ Q_σ(t). For IFR service times the inequalities are reversed." Over the class of size-blind disciplines; on queue length, stochastically; mean response by Little. Do **not** cite the IMRL variant (Righter, Shanthikumar & Yamazaki 1990, Thm 3.14): "the proof contains an error that cannot be immediately fixed". |
| 6 | software cycle times are lognormal / Weibull(k<1) | **CORRECTED — weaker than "evidence", and rebutted** | Little, *IEEE Software* 23(3):48–54, 2006 (doi:10.1109/MS.2006.82, metadata via Crossref); Eveleens & Verhoef, *Sci. Comput. Program.* 74:934–988, 2009 (cs.vu.nl/~x/cone/cone.pdf); *The Kanban Guide* 2025.5 | Little measured *project schedule-estimation ratios*, not task cycle time, and concluded lognormality from plots: "Little compared the cumulative density plots with the one belonging to the lognormal distribution and found them to be quite similar." Eveleens & Verhoef re-ran the data through Shapiro–Wilk, Anderson–Darling, Cramér–von Mises and Lilliefors and **reject** lognormality (p = 0.000) for several phases; "Little assured us that the findings are only applicable to the ex-ante ratios." The Weibull-shape claim attributed to Vacanti is **UNVERIFIED** (not in the Kanban Guide's full text; the book was not obtained). Folklore, not an established result. |
| 6d | Kanban's aging practice conflicts with DHR | **weaker than claimed** | *The Kanban Guide* 2025.5, kanbanguides.org | "Ensuring work items do not age unnecessarily, using the SLE as a reference." Negative and SLE-relative, not oldest-first; and its Work Item Age runs from *started*, a third clock. |
| 7 | four CoD shapes: linear/urgent, fixed-date, expedite, intangible (Reinertsen); SAFe WSJF sums Fibonacci ordinals | **CORRECTED** — two taxonomies merged | Arnold & Yüce, *AGILE 2013*, pp. 101–116, doi:10.1109/AGILE.2013.16; Anderson, djaa.com/classes-of-service; SAFe, framework.scaledagile.com/wsjf | "standard / fixed date / intangible / expedite" are **Anderson's Kanban classes of service**. Arnold's urgency profiles cut on "the length of the lifecycle of benefits… and whether the peak is affected by delay or not" plus an external-deadline case. Reinertsen as the source of the shapes: **UNVERIFIED**. SAFe: "WSJF is estimated as the relative cost of delay divided by the relative job duration." "Fibonacci": **UNVERIFIED** (0 hits in the public page; details login-gated). |
| 8 | EDD exact for `1‖L_max`; Moore–Hodgson exact for `1‖ΣU_j` in O(n log n) | **CONFIRMED** | Lenstra & Shmoys (eds.), *Elements of Scheduling*, arXiv:2001.06005, Thm 3.1 and §5.4; Moore, *Management Sci.* 15(1):102–109, 1968, doi:10.1287/mnsc.15.1.102; Jackson's original is *Research Report 43, Management Science Research Project, UCLA, 1955* | "Theorem 3.1 [Jackson's EDD rule]. Any EDD schedule is optimal for the problem 1‖Lmax." / "A simple and elegant algorithm of Moore and Hodgson solves the unweighted problem 1‖∑Uj in O(n log n) time." |
| 9 | Pandora's box: reservation value from `E[max(x−z,0)] = c` | **CORRECTED** — undiscounted special case; "reservation *price*"; the pair is "Pandora's Rule" | Weitzman, *Optimal Search for the Best Alternative*, MIT-EL-78-008, May 1978 (OSTI 6795538); *Econometrica* 47(3):641–654, 1979, doi:10.2307/1910412 | "ci = βi ∫(xi−zi) dFi(xi) − (1−βi)zi (7)"; "The critical number z_i which satisfies (7) is called the reservation price of box i."; "Selection Rule: If a box is to be opened, it should be that closed box with highest reservation price." |
| 10 | M/M/1: ρ/(1−ρ); 85% vs 50% ≈ 5.7× | **CONFIRMED**, quantity specified | Modiano, *Introduction to Queueing Theory*, MIT 6.263J/16.37, web.mit.edu/modiano/www/6.263/lec5-6.pdf | "N = Σ nP(n) = Σ nρⁿ(1−ρ) = ρ/(1−ρ)"; "N = Average number of customers in the system"; 0.85/0.15 ÷ 1 = **5.667×**. In *queue* it is ρ²/(1−ρ): **9.63×**. |
| 11 | Linux CFS vruntime as anti-starvation; "lexicographic tiers with decay" | **CONFIRMED** mechanism; currency and name **CORRECTED** | kernel.org `scheduler/sched-design-CFS`, `scheduler/sched-eevdf`; Arpaci-Dusseau, *OSTEP* ch. 8 | "it always tries to run the task with the smallest p->se.vruntime value"; "transitioning to EEVDF in version 6.6… moving away from the earlier Completely Fair Scheduler (CFS)". The recognized name is **priority boost** (OSTEP Rule 5: "After some time period S, move all the jobs in the system to the topmost queue"). "Lexicographic tiers with a decay term" was not found in OSTEP ch. 8 or the kernel docs. |

*Provenance note.* The pass behind this section first returned rows 6–11 with citations it had not fetched, then withdrew that message and re-fetched every source; the table above is the re-verified version only. Anything in the withdrawn version that does not appear here (a 1993 Handbooks citation for row 8; an Epema 1995 "decay-usage" reference for row 11) is not relied on anywhere in the proposal.

### B(b) What the meshwork data implies — the confound

The setup-cost matrix's headline series (of tasks still open at 24 h / 3 d / 7 d, 20% / 12% / 6%
ever closed) **is not a hazard rate**: it is P(ever closed by data end | reached age *a* open), a
cumulative, defective survival quantity. And it is **confounded with the observation window**.
Stores were born 2026-08-05 … 08-14 and the matrix's data ends 08-17, so the 7-day stratum had
zero to five days of residual follow-up — shorter than the p90 of the distribution being
measured. The percentages *must* fall with age for mechanical reasons.

The baseline in §F re-fits it the defensible way — a discrete Kaplan–Meier hazard (fraction
closing in [a, a+Δ) among tasks observed through a+Δ), with `dropped` as a competing event. Its
shape: a fast regime (27% on day 0, 8% on day 1), then a flat residual regime (2–4% per day
through day 15). Two hypotheses fit that shape and the data cannot yet separate them:

- **Decreasing failure rate.** Then FB — serve the least-served job — is stochastically optimal
  among size-blind disciplines (Righter & Shanthikumar); *deprioritize aging work*.
- **Frailty.** A mixture of will-do and never-do tasks produces a declining *population* hazard
  even when every individual hazard is constant (Lancaster, *Econometrica* 47(4):939, 1979;
  Vaupel, Manton & Stallard, *Demography* 16(3):439–454, 1979). Then the correct action is to
  **identify and drop the dead tasks**, not to demote a live one for being old.

Further caveats the report adds: competing risks are pooled in the matrix (done vs dropped —
portfolio-wide 465 vs 26 in the baseline); abandoned-but-never-dropped tasks inflate the old
strata; the theorems are on **attained service**, the matrix measures wall-clock from
`created:`, Kanban measures from *started* — three clocks; the observed closure rates were
generated under whatever prioritization was already in force (policy endogeneity); the tail
rows are small-n with no interval reported.

**What a tracker should therefore do with aging tasks:** not silently demote them. Under
*both* hypotheses the correct move is the same and is not a ranking change — **forced triage**:
surface a task past a threshold for an explicit keep-or-drop decision. Under DFR that removes
what FB would deprioritize anyway; under frailty it removes the never-do mass generating the
apparent decrease. It is also the only reading compatible with the Kanban Guide's actual
wording. The proposal's MW-R12/R13 are this paragraph.

### B(c) Minimal fields, and what meshwork has

Needed for an empirical age-based index: attained service per task; completed sizes (samples of
attained service at completion — empirical Gittins needs only their empirical CDF, never a
per-task estimate); terminal event type; a censoring indicator and as-of stamp; arrival times;
optionally class covariates. meshwork's `log` projection `(gid, ord, date, from_status,
to_status, note)` with minute stamps and a five-value status enum already supplies event type,
censoring, arrivals, and — derivably, as Σ time in `doing` — attained service. **The limit is
coverage, not schema**: only 280 of 1,039 task files ever record a `→doing` transition and 200
close `open→done` directly (the baseline: 264 of 464 dated closes have a start). That is a
ritual gap, not a format gap; deriving service from transitions already logged does not collide
with the §3 *no time tracking* fence, while adding a timer would. Two precisions in the log
(2,403 minute-resolution bullets vs 133 date-only) also bound what can be resolved. **On the
wall-clock clock alone**, a SOAP-style rank function of age is buildable and analyzable but is
not the Gittins index; none of claims 1, 2, 5 transfer to it. The proposal says so (§9) and
does not borrow the citations for a wall-clock rule.

---

## C. Packing, setup cost, agent DAGs

Sources: arXiv API records and PDFs (`pdftotext`), Crossref / OpenAlex / OpenAIRE / Unpaywall
metadata, publisher-deposited abstracts, PSPLIB's machine-readable JSON, raw GitHub READMEs.
Where a full text was unreachable the row says so.

### C(a) Verdicts

| # | claim (the notes) | verdict | citation | verbatim support |
|---|---|---|---|---|
| 1 | Next Release Problem = 0-1 knapsack + precedence closure (Bagnall et al. 2001) | **CONFIRMED**, two nuances | Bagnall, Rayward-Smith & Whittley, *Inf. Softw. Technol.* 43(14):883–890, 2001, doi:10.1016/S0950-5849(01)00194-X (abstract only — full text not open; UEA repository: "Full text not available"); formulation via Domínguez-Ríos et al., arXiv:2402.04586; canonical name via Dose, Furini & Locatelli, arXiv:2606.22018 | "the problem of selecting an optimal next release is shown to be NP-hard." / "The Precedence Constrained Knapsack Problem (PCKP) asks for a maximum-profit subset of items, subject to a knapsack capacity constraint and precedence constraints encoded by a directed acyclic graph." Nuance: NRP's profit accrues to *stakeholders*, who count only if all their requirements ship — an AND-layer that folds into the DAG; "items carry the profit" is wrong. |
| 2 | Veerapen et al.: ILP competitive with metaheuristics, with optimality | **CONFIRMED** | Veerapen, Ochoa, Harman & Burke, *Inf. Softw. Technol.* 65:1–13, 2015, doi:10.1016/j.infsof.2015.03.008 | "We show that a modern Integer Linear Programming solver is now a viable method for this problem. Large single objective instances and small bi-objective instances can be solved exactly very quickly." |
| 3 | CP-SAT solves RCPSP at n≈50–300 with 3–5 resource types in seconds | **CORRECTED** | PSPLIB summaries (om-db.wi.tum.de/psplib, fetched 2026-09-01); MiniZinc Challenge 2024 results | j30 480/480 proven optimal · j60 382/480 · j90 375/480 · **j120 89/600** (`"num_unproven": 511`); instances carry 4 renewable resources (`j1201_1.sm`: "renewable: 4 R"). CP-SAT took Gold in every 2024 MiniZinc category — it is the strongest general solver — **and RCPSP at n ≥ 100 is still open after three decades.** The *packing* side scales far better than the resource-scheduling side: exact PCKP algorithms report > 1000 items. |
| 4 | serial/parallel SGS; regret-based biased random sampling with LFT (Kolisch & Drexl 1996); forward–backward improvement (Tormos & Lova 2001; Valls et al. 2005) | **CONFIRMED**, one attribution corrected | Kolisch, *EJOR* 90:320–333, 1996 (SGS); Kolisch & Hartmann, *EJOR* 174:23–37, 2006; Tormos & Lova, *Ann. OR* 102:65–81, 2001; Valls, Ballestín & Quintanilla, *EJOR* 165:375–386, 2005; regret-based sampling is **Drexl, *Management Sci.* 37:1590–1602, 1991** | Goncharov, arXiv:2502.18330: "we apply a local improvement procedure FBI (the forward-backward improvement procedure) to the resulting schedule." Kolisch & Drexl 1996 (*NRL* 43:23–40) is real but is "a hybrid of priority rule and random search techniques", not the origin of regret sampling. |
| 5 | `1\|s_ij\|Σw_jC_j`: B&B tops out ~40 jobs / 2 h; best-index dispatch O(n⁴); modified WSPT O(n³) | **CONFIRMED**, variant corrected | **Chou, Wang & Chang, *Int. J. Adv. Manuf. Technol.* 43(7–8):810–821, 2009, doi:10.1007/s00170-008-1762-4** (abstract via OpenAIRE's harvest of the publisher deposit) | "a best index dispatch (BID) and a modified weighted shortest processing time (MWSPT) … The time complexities of the two proposed heuristics are O(n 4) and O(n 3), respectively." / "the branch-and-bound method could solve most instances with 40 jobs under the time limit of 7,200 s." The problem is **`1\|r_j, s_ij\|Σw_jC_j` — with release times**; say "solved most 40-job instances", not "tops out", and do not generalize (Tanaka & Araki 2013 solve 85-job `1\|s_ij\|ΣT_j` exactly). Strong NP-hardness is inherited via the cumulative-TSP / minimum-latency special case (Bianco, Mingozzi & Ricciardelli 1993; Sitters 2002), never via TSP↔makespan. |
| 6 | past-sequence-dependent setups (Koulamas & Kyparisis 2008) are studied only for degenerate cost functions; prefix-dependent setup is open | **CONFIRMED**, and the "gap" is narrower than claimed | Koulamas & Kyparisis, *EJOR* 187:1045–1049, 2008; form via Zhu, Chu, Yu & Sun, *RAIRO-OR* 50:733–748, 2016; **Tang & Denardo, *Oper. Res.* 36(5):767–777, 1988**; Crama, Kolen, Oerlemans & Spieksma, *IJFMS* 6:33–54, 1994 (title via OpenAlex; abstract UNVERIFIED); Lee, Lei & Pinedo, *NRL* 59(1):58–68, 2012 (quote relayed from OpenAlex by a nested pass — UNVERIFIED by the reporting agent) | p-s-d: "Koulamas and Kyparisis [25] initiated this form of setup times which depends on all already scheduled jobs… still solvable in polynomial time" — `s[r] = ε·Σ_{l<r} p[l]`, a **scalar** aggregate that discards identity. The prefix-**set** model exists under another name — **tool switching**: "If the requisite tools are not on the machine, then one or more tool switches must occur before the job can be processed." Cost is `\|T_j \ magazine\|` against a capacity-bounded working memory; KTNS is optimal in polynomial time for a fixed sequence; the sequencing problem is NP-hard. An arbitrary function of the full prefix set was **not found** in Allahverdi et al. 2008/2015, the Koulamas–Kyparisis chain, or arXiv. |
| 7 | empirical context-switch cost for knowledge workers / agents | **CONFIRMED**; one famous number is folklore | Mark, Gonzalez & Harris, CHI 2005; Parnin & Rugaber, ICPC 2009 / *Softw. Qual. J.* 19(1):5–34, 2011; Gupta, Sheth, Raina, Gales & Fritz, EMNLP 2024, arXiv:2402.18216 | "57% of their working spheres are interrupted"; "only 10% of the programming sessions have coding activity start in less than a minute, only 7% … involve no navigation to other locations prior to editing"; LLMs: "many of the task-switches can lead to significant performance degradation." **"23 minutes to refocus" appears nowhere in Mark 2005 or Mark, Gudith & Klocke 2008** (both full texts grepped); Mark 2008's finding runs the other way ("people completed interrupted tasks in less time … at a price: … more stress"). |
| 8 | arXiv:2406.14096 is a GNN-for-JSSP survey; JobShopLib's model beat graph dispatchers on operation features alone | **CORRECTED** — two papers conflated | Smit et al., *Graph Neural Networks for Job Shop Scheduling Problems: A Survey*, arXiv:2406.14096 (accepted *Comput. Oper. Res.*); Ariño Fernández, arXiv:2506.13781; `github.com/Pabloo22/job_shop_lib` | The survey resolves as claimed. JobShopLib exists ("a Python package for creating, solving, and visualizing job shop scheduling problems"). The quoted result is from JobShopLib's own paper — "One model outperformed various graph-based dispatchers using only individual operation features" — not from the survey; the two are unrelated projects. |
| 9 | arXiv:2604.11378 "Graph Harness" exists and tabulates where the DAG analogy fails | **CONFIRMED**, title corrected | Hu Wei, *From Agent Loops to Structured Graphs: A Scheduler-Theoretic Framework for LLM Agent Execution*, arXiv:2604.11378, 2026-04-13 | "Table 3: Classical DAG scheduling vs. LLM agent scheduling: why the analogy is not trivial." — five rows (deterministic output → "Same input may yield different outputs; hallucination"; context = parameters → "Context includes reasoning history that may corrupt subsequent steps"; retry = idempotent → "non-idempotent side effects"…). "Graph Harness" (SGH) is the system name. Caveat, in the paper's own words: "This is a position paper and design proposal … not a production implementation or empirical results." |
| 10 | agent-orchestration work rediscovering list scheduling / critical path | **CONFIRMED** | LLMCompiler (Kim et al., arXiv:2312.04511: "latency speedup of up to 3.7x"); Parrot (Lin et al., OSDI '24); Teola (arXiv:2407.00326: "reversed topological sort"); Autellix (arXiv:2502.13965: "prioritizing critical LLM calls"); LLMSched (arXiv:2504.03444, ICDCS 2025); Shi, Zheng & Lou, arXiv:2601.10560 ("the critical path as the longest path in G measured by execution time"); adjacent: arXiv:2606.00953, arXiv:2608.25523; Anthropic, *How we built our multi-agent research system*, 2025-06-13: "You can't hardcode a fixed path for exploring complex topics, as the process is inherently dynamic and path-dependent." | all serve LLM *call* graphs at sub-second latency — a serving problem, not a backlog. |
| 11 | any guarantee for naive greedy max `w/p` among ready tasks under precedence | **CORRECTED — none** | Graham 1966 (*BSTJ* 45:1563–1581) for makespan; Chekuri & Khanna, *Handbook of Scheduling* ch. 11; Jäger & Warode, arXiv:2309.12031 (Sidney in O(n³)); Skutella, arXiv:1603.04690v2 | Makespan: list scheduling is 2 − 1/m. Weighted completion on parallel machines: "as much as an Ω(m) factor away from the optimum". One machine with precedence: every 2-approximation is Sidney-consistent; naive greedy is not, and **no published ratio was found**. Bad example (the reporting agent's own construction, not a citation): chain `a₁(w=0,p=1) → a₂(w=W,p=ε)` plus independent `b(w=1,p=P)` — greedy takes `b` first, cost ≈ W·P vs optimal ≈ W + P, unbounded. |

### C(b) Relevance to meshwork (the report's own reading)

- **Collapses to nothing** with no `p_j` and no resources: all of claim 4 (an SGS degenerates
  to a topological sort; FBI has nothing to squeeze); claim 3's solver question (10–300 tasks
  with no resources is nowhere near a hard instance); claim 5's exact algorithms; claim 11's
  ratios except as the warning below.
- **Survives:** precedence closure — the ready set, already computed; claim 2 if a cost or
  value field ever appears (an exact solver beats hand-rolled search at these sizes, with a
  proof); claim 7 as the measured warrant that switching costs are real for LLM agents.
- **Packing under a context window is not knapsack; it is paging.** Knapsack needs additive
  weights against a scalar capacity; context is neither additive (shared files amortize) nor
  known in advance. The tool-switching literature has the model:

  | tool switching | meshwork |
  |---|---|
  | tool magazine, capacity C | the session's context window |
  | tool set `T_j` required by job j | the `docs:` refs / files a task touches |
  | switching cost `\|T_j \ magazine\|` | context that must be re-read to start this task |
  | KTNS — optimal, polynomial, for a fixed sequence | what to keep in context once an order is fixed |
  | sequencing — NP-hard | which order to work tasks in |

  The hard half is the order; the easy half has an optimal greedy. That is an argument for a
  simple, cheap ordering heuristic: exactness is unattainable on the hard half and unnecessary
  on the easy one.
- **Cheapest useful thing:** make the ready-set ordering *cohesion-aware* — among ready tasks,
  prefer those sharing a `category` prefix, `docs:` paths, or `parent` with what the session
  has already touched. O(n), no new fields. Two guardrails: never add a bare max-value/effort
  greedy even if estimates arrive (no guarantee under precedence, unbounded bad example — the
  proposal routes any cost ordering *after* inheritance); never build a static plan DAG
  (SGH's Table 3 and Anthropic's "inherently dynamic and path-dependent" both argue against it
  — `prime` re-derives the ready set each run, which is the right posture).

---

## D. Prior art in trackers

Sources: raw docs pages (`curl`), raw GitHub READMEs and source (`raw.githubusercontent.com`),
browser-rendered pages where curl was refused; absence claims name the pages checked.

| tool | dep-aware readiness | graph-derived priority | value ÷ effort | aging | packing | ordering, quoted |
|---|---|---|---|---|---|---|
| **Taskwarrior** | yes — `depends:`, `blocked`/`blocking` virtual tags | no — booleans, no blocker count | no | yes — linear ramp capped at `age.max` (365 d) | no | weighted sum `U = A·tₐ + B·t_b + …` (taskwarrior.org/docs/urgency, `src/Task.cpp`) |
| **Beads** (`bd`) | yes — `bd ready`; `blocks`/`parent-child`/`conditional-blocks`/`waits-for` gate, `related`/`discovered-from` do not | no (native) | no | inverted — `hybrid` buries anything > 48 h old beneath fresh work | no | `ORDER BY priority ASC, created ASC, id ASC` (`internal/storage/sqlbuild/ready.go`) |
| **Beads Viewer** (`bv`, community sidecar for agents) | yes | **yes** — PageRank, betweenness, blocker ratio, critical path | closest found | yes — explicit staleness term | no | `Impact = 0.30·PageRank + 0.30·Betweenness + 0.20·BlockerRatio + 0.10·Staleness + 0.10·PriorityBoost`; flags where human priority disagrees with computed impact |
| **Linear** | no — "blocked" is a sidebar flag | no — sort by Status/Manual/Priority/Date/Link count | no | no — SLA is a countdown, not a boost | partial — Cycle shows capacity | "drag & drop… position will be saved globally" |
| **Jira Advanced Roadmaps** | yes — auto-scheduler "considers… dependencies" | partial — aggregate, no formula | marketplace apps only (four competing WSJF plugins) | no | yes — by capacity/velocity | "balances… dates… estimates, a team's capacity and velocity, and any dependencies… in aggregate" |
| **Asana** | partial — notifies, does not block completion | yes — Timeline critical path, visualization only | no | no | no | "critical path is the longest chain of dependent tasks… highlighted in yellow" |
| **MS Project** | yes | yes — native CPM | no | no | yes — leveling, priority 1–1000 | "critical if… 0 days of Total Slack" |
| **OpenProject** | yes — automatic scheduling | no — "we don't have the critical path feature yet… feature request" | no | no | no | (gantt FAQ) |
| **Shortcut** | no dependency concept found | no | no | shipped then deprecated — "Card Aging [Deprecated]" in the live nav | no | five fixed priority values |
| **GitHub Projects** | no — "blocked by" is an icon | no | no | no | no | priority is a custom field |
| **org-mode** | no — `org-enforce-todo-dependencies` blocks *state changes* | no | no | via deadline proximity | no | `org-agenda-sorting-strategy` (ordered predicates, not a score) |
| **Todoist / Things** | no (Todoist's paid dependency feature UNVERIFIED — no primary page reached) | no | no | no | no | "Priority → Date and time → Deadline → Manual ordering → creation time"; Things has no priority field |
| **MCP Agent Mail / claude-flow (ruflo)** | delegates to Beads / topological | no | no | no | no | "Beads owns task prioritization"; "planner schedules them in parallel where the dependency graph allows" |
| **Height** | UNVERIFIED — site unreachable | | | | | |

**Taskwarrior in full.** Coefficients: `next` tag 15.0 · due 12.0 · **blocking 8.0** · priority
H/M/L 6.0/3.9/1.8 · scheduled 5.0 · active 4.0 · **age 2.0** · annotations 1.0 · tags 1.0 ·
project 1.0 · waiting −3.0 · **blocked −5.0**. `urgency_age()` = `age_days / urgency.age.max`,
capped at 1.0; `blocking`/`blocked` are flat 1.0/0.0 — not weighted by how many tasks are
blocked or how deep the chain runs. **`urgency.inherit`** (off by default) propagates a blocking
task's urgency up to the highest urgency of what it blocks plus 0.01 — the only graph
propagation present, and the proposal's MW-R6c in a different tool; issues #2743 ("the urgency
score doesn't help me prioritise which tasks should be completed first") and #3111 (a worked
example where a dependency still scores *below* what it blocks) are the field record.

**Beads in full.** No computed component anywhere; three sort policies, `hybrid` = two buckets
at 48 h with the older bucket's priority forced to 999 — *not* anti-starvation, the reverse (an
accidental decreasing-hazard policy). `--sort oldest` is the explicit anti-starvation mode and
is manual.

**What nobody does** (positively checked, pages named above): native WSJF or cost-of-delay ÷
duration in any tracker's own product; Sidney decomposition as a named implementation (`gh
search code "Sidney decomposition"` → one unrelated Bitcoin-fee article; `"sidney's algorithm"`
→ 0; crates.io/npm/PyPI → nothing); CD3-with-dependencies as a library (nearest: npm
`@loom-loyalty/meridian-priority-reference`, WSJF-shaped with aging, no graph input);
critical-path length *feeding a score* rather than a Gantt highlight (only `bv`); Anthropic's
multi-agent guidance mentioning dependency-graph ordering (building-effective-agents, the
multi-agent research post, and the Claude Code best-practices doc read in full — decomposition
patterns only; the research post flags dependency-heavy work as a poor fit for decomposition).

**Design lessons the survey supports.** *Ready* is binary everywhere but `bv`. Aging, where it
exists, is a soft capped nudge (Taskwarrior's 2.0 vs 12.0 for due) or inverted (Beads). Manual
override always wins over any computed score (Linear's global drag position; org-mode's
predicate list). Capacity packing exists only in heavyweight PM tools. WSJF is documented as
gameable and ships only as plugins. Features get walked back (Shortcut's card aging). The
richest graph scoring found is not a PM feature but an agent-oriented sidecar built to catch
human-priority-vs-structure disagreement — the same disagreement the proposal's `needs-behind`
finding names, and the same one Taskwarrior's `urgency.inherit` was added for.

---

## E. Rust building blocks (feasibility)

Sources: `Cargo.toml` / `Cargo.lock` at HEAD; DataFusion's **51.0.0-tagged** source tree on
raw.githubusercontent.com (the live docs site serves the newest release, so version claims were
not taken from it); crates.io per-version API; docs.rs "all items" pages; raw READMEs and
changelogs.

1. **DataFusion 51.0.0** (pinned `"51"`; resolved 2025-11-19; crates.io latest 55.0.0).
   Recursive CTEs are on by default (`datafusion.execution.enable_recursive_ctes | true`); the
   physical operator's doc comment warns "there won't be any limit or checks applied to detect
   an infinite recursion, so it is up to the planner to ensure that it won't happen" — a bounding
   predicate is mandatory. The v51 `cte.slt` tests confirm aggregation inside a recursive CTE and
   `GROUP BY` in the recursive term work; distinct `UNION` errors ("only `UNION ALL`"), nested
   recursive CTEs error, and a second recursive self-reference errors. Window functions,
   `array_agg`, `unnest` documented at v51. Issue #19427 (2025-12-20, on main): a self-join of
   the same recursive CTE under `LIMIT` can hang — UNVERIFIED for 51.0.0, not meshwork's pattern.
   **Recommendation:** transitive closure over `edges` as `WITH RECURSIVE … UNION ALL` with a
   depth bound, surfaced as a column/view, never a change to `READY_SQL`.
2. **Max-flow / min-cut.** `petgraph` 0.8.3 is **already in the tree** as a transitive
   dependency of `datafusion-physical-expr` (`Cargo.lock:2262–2271`), never imported by
   meshwork's code (`grep -rn petgraph src/` → none). Its `algo::maximum_flow` module: "`dinics`
   … Implements Dinic's algorithm … `ford_fulkerson` … Ford-Fulkerson algorithm in the
   Edmonds-Karp variation"; Dinic's landed in 0.8.3 (2025-09-30). `pathfinding` 4.15.0
   (Apache/MIT, updated 2026-03-10) offers `edmonds_karp` returning "the maximum flow **and the
   minimal cut**", pure Rust. `rs-graph` 0.21.0 is **GPL-3.0+** and stale since 2023 — not
   recommended. No parametric max-flow crate found (crates.io search: zero results).
   **Recommendation:** add `petgraph = "0.8"` directly at zero marginal cost and call `dinics`;
   derive the cut by residual BFS.
3. **Integer/constraint programming.** `good_lp` 1.15.3's own backend table: `coin_cbc` needs
   the cbc C library at build and run time; `highs` needs a C++ compiler; `microlp` (pure Rust)
   is the only backend with "no C compiler, no additional libs", flagged "not fast" — irrelevant
   at n ≤ 300. `russcip` vendors a prebuilt SCIP blob. Rust CP-SAT bindings (`cp_sat` 0.4.1,
   `cpsat-rs`, `ortools-scip`) all link OR-tools' C++. **Recommendation:** a hand-rolled DP /
   branch-and-bound (< 200 lines, zero deps) if packing is ever needed; `good_lp + microlp` as
   the only toolchain-free MILP. The proposal needs neither (MW-R19).
4. **meshwork's own graph code.** Cycle detection is a hand-rolled DFS (`src/lint.rs:282–344`);
   the `ready` order is `READY_SQL` (`src/cli/query.rs:33–45`) — `seq` then `created`; the
   portfolio total order is `sort_total` (`src/cli/portfolio.rs:85–115`).
5. **Distribution fitting.** `statrs` 0.19.1 and `rand_distr` 0.6.0 index pages contain no
   `fit` / `mle` / `estimat*` items (absence confirmed by direct inspection of both "all items"
   pages, not a summary); `survival` 1.3.0 drags `ndarray` + `rayon` and a pyo3 shim.
   **Recommendation:** hand-rolled Kaplan–Meier / empirical hazard table, < 200 lines, no dep.
6. **DAG metrics in petgraph 0.8.3.** Provided: `algo::toposort`, `algo::articulation_points`
   (Tarjan; landed 0.8.0), `algo::has_path_connecting`, `algo::condensation`. Not provided
   (404 on docs.rs): betweenness centrality; transitive-successor *count*. **Recommendation:**
   `toposort` plus a ~30-line topological DP for longest path and reachable-successor count.

| option | crate/version | pure Rust | C toolchain | binary delta | maintenance |
|---|---|---|---|---|---|
| recursive CTE | datafusion 51.0.0 (pinned) | yes | no | 0 | active; 55.0.0 latest |
| max-flow (Dinic's) | petgraph 0.8.3 (transitive already) | yes | no | ~0 | active; Dinic's 2025-09-30 |
| max-flow + cut | pathfinding 4.15.0 | yes | no | small | active, 2026-03-10 |
| max-flow | rs-graph 0.21.0 | yes | no | — | GPL-3.0+, stale — no |
| MILP | good_lp 1.15.3 + microlp 0.6.0 | yes | no | small–moderate | active, 2026-08-06 |
| MILP | good_lp + highs 2.4.0 | no (C++) | yes | MB-scale | fails the constraint |
| CP-SAT | cp_sat 0.4.1 etc. | no | yes | very large | wraps OR-tools regardless |
| DP / B&B | none | yes | no | 0 | house code |
| distribution fit | statrs 0.19.1 | yes | no | small | no fit API |
| survival | survival 1.3.0 | mostly | no | moderate–large | Python-shaped |
| Kaplan–Meier | none | yes | no | 0 | house code — recommended |
| DAG metrics | petgraph 0.8.3 | yes | no | ~0 | active |

---

## F. The baseline (2026-09-01) — method and tables

**Method.** Per-repo `meshwork q --json` over `tasks`, `edges`, `log` for the nine registered
stores (meshwork, sazed, leras, marasi, tensoon, oreseur, wyndam, marasi-applied-r-and-d,
portfolio), run from each repo's own checkout with the meshwork debug binary at HEAD — **never a
`portfolio` verb**, which autoprunes `sequence.md` as a side effect of reading (atlas
`PLAYBOOK.md`: *"Reading state can mutate state"*). "Now" = 2026-09-01T18:00Z. Stamps parsed
as FORMAT.md's two conforming forms; nonconforming stamps skipped. Live = open/doing/blocked.
Cycle = `created→done`; queue = `created→first doing`; service = `first doing→last done`,
requiring `created ≤ doing ≤ done`. Reproducible with `scripts/mine_rank.py` (untracked, landed
beside this file; `--check` exits 0 when every denominator is non-zero). The stores are live —
sibling sessions close and file tasks while this runs — so a re-run drifts by a few units: the
script's own 19:04Z pass over the same nine stores read live 455 / seq'd 396 / categorized 300 /
dated closes 469 with start 265, against the 18:00Z figures tabulated below, and its day-0
hazard row was 852 at risk, 232 done (0.272) against 834 / 228 (0.273). Every table below is
the 18:00Z pass; the shape of every finding survives the drift.

**F1. Placement today (live tasks).**

| repo | live | seq'd | unseq'd | categorized | distinct categories |
|---|---|---|---|---|---|
| sazed | 239 | 229 | 10 | 194 | 29 |
| leras | 64 | 53 | 11 | 5 | 5 |
| marasi | 43 | 38 | 5 | 36 | 8 |
| tensoon | 38 | 29 | 9 | 17 | 3 |
| marasi-applied-r-and-d | 24 | 3 | 21 | 24 | 9 |
| wyndam | 15 | 15 | 0 | 0 | 0 |
| portfolio | 14 | 14 | 0 | 14 | 5 |
| meshwork | 11 | 10 | 1 | 11 | 7 |
| oreseur | 9 | 8 | 1 | 0 | 0 |
| **total** | **457** | **399 (87%)** | 58 | **301 (66%)** | |

**F2. Cycle, queue, service.** Dated closes 464; with a `→doing` line 264 (57%; sazed 42 of 167).

| quantity | median | p90 | n |
|---|---|---|---|
| cycle (`created→done`) | 17.3 h | 268 h | 464 |
| queue (`created→doing`) | 18.1 h | 285 h | 264 |
| service (`doing→done`) | **0.20 h** | 2.8 h | 264 |
| per-task queue : service | **38×** | | 264 |
| share of lifetime waiting | 0.98 | | 264 |
| services ≤ 1 h | 80% | | 264 |

Category service medians, categories with n ≥ 7 (portfolio-wide): skill 0.03 h (13) · core/format
0.05 (19) · core/lifecycle 0.07 (12) · core/verify 0.12 (15) · meta/distribution 0.17 (7) ·
plan/m2 0.17 (8) · kwaan 0.38 (11) · (none) 0.41 (86) · method 1.27 (10). Category *cycle*
medians span 0.32 h (meta/distribution) to 211 h (core/verify) — that spread is queue position,
not effort.

**F3. Discrete daily close hazard** (all stores pooled; among tasks open at age *a* and
observable through *a*+1 d — created at least *a*+1 d before now — the share closed `done`
inside the day; `dropped` separately). n = 947 (465 done, 26 dropped, 456 open).

| age (d) | at risk | done | dropped | h_done | h_drop |
|---|---|---|---|---|---|
| 0 | 834 | 228 | 13 | **0.273** | 0.016 |
| 1 | 583 | 46 | 1 | **0.079** | 0.002 |
| 2 | 517 | 18 | 3 | 0.035 | 0.006 |
| 3 | 489 | 19 | 2 | 0.039 | 0.004 |
| 4 | 464 | 8 | 0 | 0.017 | 0 |
| 5 | 451 | 10 | 0 | 0.022 | 0 |
| 6 | 419 | 8 | 3 | 0.019 | 0.007 |
| 7 | 402 | 5 | 1 | 0.012 | 0.002 |
| 8 | 363 | 15 | 0 | 0.041 | 0 |
| 9 | 348 | 8 | 0 | 0.023 | 0 |
| 10 | 319 | 11 | 0 | 0.034 | 0 |
| 12 | 270 | 7 | 0 | 0.026 | 0 |
| 14 | 233 | 7 | 0 | 0.030 | 0 |
| 16 | 209 | 1 | 0 | 0.005 | 0 |
| 18 | 183 | 1 | 0 | 0.005 | 0 |
| 20–24 | 169 → 107 | 0–1 | 0 | ≤ 0.009 | 0 |

First day in 4-hour bins: [0,4) h **0.174** (at risk 914) · [4,8) 0.023 · [8,12) 0.023 ·
[12,16) 0.029 · [16,20) 0.026 · [20,24) 0.020. Rows from day 16 on have short residual
follow-up (oldest store 27 d) and wide intervals; the cumulative "ever closed" series the matrix
printed (and this baseline's first pass reproduced: 33% / 26% / 21% / 7.6% at 1/3/7/14 d) is
shown in §B(b) to be confounded and is not used.

**F4. Live tasks past 14 d** (the proposed triage age): sazed 146 of 239 (139 seq'd; top
categories (none) 28 · method 15 · housekeeping 14 · engine/memory 13 · seam/facts 9), leras 48
of 64 (all uncategorized), meshwork 8 of 11 (plan/m3 5), tensoon 8, marasi 5, oreseur 4 —
**219 of 457, 210 with a `seq`**.

**F5. Structure.** `needs` edges with both ends live: 47 (meshwork 5, sazed 15, leras 1,
marasi 14, tensoon 6, oreseur 1, wyndam 0, lab 1, portfolio 4). Live tasks with any live
transitive dependent: 35 (sazed 12, marasi 9, meshwork 5, tensoon 3, portfolio 3, leras 1,
oreseur 1, lab 1, wyndam 0). Multi-member lanes over `needs` only: sazed 9 (sizes 4,4,4,2,2,2…),
marasi 4 (8,4,2,2), portfolio 3 (3,2,2), tensoon 2 (5,3), meshwork 1 (6), leras 1 (2), oreseur 1
(2), lab 1 (2) — 67 live tasks in a multi-member lane; adding `parent` edges: 106, and leras
gains one 26-member component (a `parent` umbrella).

**F6. Sidney and inheritance on the real stores** (ready set; `w = max(1, 1001 − min(place,
1000))`, `p ≡ 1`, place = `seq` or 999999; recursive Dinkelbach over Edmonds–Karp; infinite arcs
= `needs` rows as stored).

| repo | ready | live needs | Sidney blocks | ready positions moved under Sidney | top-8 changed | inheritance findings |
|---|---|---|---|---|---|---|
| meshwork | 6 | 5 | 6 | 0 | no | mw-cvw8: place 900, inherits **150** (the v1 gate mw-v4ej needs the mirror block) |
| sazed | 222 | 15 | 129 | 75 | no | sa-399rbej: unranked → 72; sa-cytn77t: 190 → 30; sa-f6yy7w9: 710 → 570 |
| marasi | 23 | 14 | 35 | 0 | no | none |
| tensoon | 30 | 6 | 26 | 0 | no | none |
| leras | 53 | 1 | 47 | 0 | no | none |
| portfolio | 9 | 4 | 11 | 0 | no | none |

**Caveats.** Nine stores born 2026-08-05 … 08-21, so no age beyond 27 d is observable; date-only
stamps (133 bullets) round to midnight; sessions that never `start` leave no service interval
(43% of closes), and that absence almost certainly correlates with task size; the meshwork store
dogfoods against fixture stores in tests, which does not touch these numbers (they read the real
`docs/meshwork/`); `dropped` is 5% of terminal events and is carried as a competing risk, never
pooled with `done`.

---

## G. Local sources read (2026-09-01)

`docs/REQUIREMENTS-meshwork.md` (§2 A–J, §3 with its three scope rulings) · `docs/DESIGN-
meshwork.md` (§5 `ready` SQL, §6 surface, §7/7b prime, §9 portfolio + sequence hygiene, §15
decisions) · `FORMAT.md` (task file schema, stamps, addressed tasks) · `docs/PLAN-meshwork-
build.md` (Position line) · `docs/setup-cost-matrix.md` + `scripts/mine_setup_cost.py` ·
`docs/PROPOSAL-spec-traceability.md` (format precedent) · `src/cli/query.rs`, `src/cli/
portfolio.rs`, `src/cli/prime.rs`, `src/lint.rs` (finding kinds) · store tasks mw-0vw7nj0,
mw-908n9k2, mw-06j1wqe, mw-4jgrjar, mw-2nmsys2, mw-jqj9qa9, mw-59f0t1q, mw-r6g9bhe ·
`portfolio/task-prioritization-notes.md` (both halves) · `portfolio/DESIGN-graph-of-graphs.md`
§5, §7 ladder, §11 falsifiers, caveat 3 · `portfolio/PROPOSAL-meshwork-for-refresh-sessions.md`
§2.1, §2.5, §5 · `portfolio/sequence.md` (08-31 cut; 08-21 ladder incl. Rung M) ·
`portfolio/STATUS.md` §9 (the WHY rule) · `atlas/README.md`, `PLAYBOOK.md` (read-can-mutate),
`QUERIES.md` Q7–Q9, Q12, `decisions/open-calls.md` (answered/assigned/struck),
`clusters/dev-instrumentation.md` §5, `clusters/devtools.md` (backlog burn), `clusters/
governance-canon.md` (value-per-token ruling) · `sahjhan/docs/superpowers/specs/2026-03-29-gate-
composition-branching-design.md` (first-match transition candidates — the second snippet's
precedent).
