# PROPOSAL — analytics: the derivatives an agent will not take

**Status: proposal, unruled.** Written 2026-09-06. Method: every metric below was written as SQL
over the six-table projection, executed read-only against all nine registered stores with the
v0.4.0 release binary (per-repo `q --json`, never a `portfolio` verb — those autoprune
`sequence.md`), and timed. Every number carries its denominator and the time it was taken
(2026-09-06T16:11Z). The complete view layer is Appendix A, byte-identical to what ran; the
validation record is Appendix B. Companions: `PROPOSAL-prioritization.md` (the ordering this
proposal feeds) and `portfolio/PROPOSAL-meshwork-ui.md` (the reader this proposal feeds). The
inventory of hand-computed numbers in §2 was mined from the portfolio's own BURNDOWN, STATUS,
ARC and review documents.

**The thesis in one paragraph.** `prime` shows an agent *state*: counts, a ready list, a next task,
a handoff. An agent reads state well and never takes the derivative — it will not notice that the
backlog widened on 26 of the last 33 days, that half the live work is past two weeks old and parked
under a `seq`, that the task it is about to lead with has a handoff citing eight closed tasks, that
the same ticket has been touched by three sessions since it last moved, or that one task filed
two days ago has already spawned twenty descendants seven levels deep. The store holds every
input for those sentences and has since the pilot; the setup-cost matrix, the prioritization
baseline, and the UI proposal each had to leave the tool and write a script to get them. The
proposal is a **view layer**: thirteen named SQL views over the six tables plus a one-row
`clock`, registered by the binary, published verbatim in FORMAT.md, queryable by `q` and
`portfolio q` the day they land, consumed by a five-line **pulse** block in `prime`, by a `stats`
verb if the surface is ruled open, and by meshwork-ui as its analytics feed. Nothing is stored,
nothing is authored, nothing is forecast, nothing runs in the background. The derivative is
taken by the tool, once, in SQL, where the definition is the documentation.

---

## 1. What the views say — the 2026-09-06 baseline

Nine stores, single-repo loads summed (so cross-repo joins are what a *repo* sees, not what the
union sees — §6.2 says why that matters for asks). Live = open, doing, blocked.

| measure | value | denominator / note |
|---|---|---|
| tasks | **1,225** | 437 open · 5 doing · 12 blocked · 741 done · 30 dropped |
| **filed / closed, last 7 days** | **380 filed · 305 done · 6 dropped** | backlog **+69** in a week; 131 of the 380 (34%) carry `discovered-from` — filed *from* other work |
| **days the pooled backlog widened** | **26 of 33** | narrowed 4, flat 3, since the first store (2026-08-05); the last seven: +46 +1 +15 −4 +7 −1 +4 |
| ready | **371 of 437 open (85%)** | the queue's primary verb still does not discriminate (UI proposal §1, reading 1) |
| doing | 5, **4 stale** (idle ≥ 72 h) | STALE_DOING_DAYS = 3 |
| live past 14 days | **260 of 454 (57%)** | 219 of them carry a `seq` — *parked in place* |
| open age, median | 5.6 d (applied-lab) … **32.7 d (meshwork)** | per repo; sazed 21.9 d over 259 open |
| daily close hazard, pooled | day 0 **0.357** (n 1,171) · day 1 0.081 (718) · days 2–3 ≈ 0.047 · **days 4–15 mean 0.025** · day 21 0.009 (234) · day 28 0.000 (129) | the 09-01 shape, five days on: a fast population, then a flat residual |
| live tasks that unlock something | 37 | multi-member `needs` lanes: 25 · `needs-behind` contradictions: 9 · live tasks blocked on an unresolved edge: 14 |
| unanswered outbound asks | **32** | as seen from the *senders'* stores; every store's inbound count is **0** in a single-repo load — the join needs the union (§6.2) |
| friction, last 7 days | **29 failed close attempts** · 6 reopens · 2 blocks | all-time waived closes 6; live with no `verify:` 22; no `seq` 50; **no category 138 of 454** |
| attention, last 7 days | 269 comments by 29 distinct identities | 13 live tasks touched by ≥ 2 identities since they last changed status |
| **handoffs citing closed tasks** | **85 live tasks** | sazed 69: `sa-72rrcck`'s handoff names 8 terminal ids, `sa-0bbnpw7`'s names 6 and has been idle 15 d |
| **mentions with no edge behind them** | 928 pairs; **107 live→live** | ids named in a body, handoff, comment or log note of another task, resolved, and not backed by any declared edge |
| **spawn** | 66 origins spawned a descendant this week; deepest chain **7** | applied-lab `ar-pwtdbx5` ("Profile the dock rig") → 20 descendants, 4 live; sazed `sa-72rrcck` → 25 descendants in 7 d, depth 4, 8 direct — and it is the task with the 8-closed-id handoff |
| umbrellas | leras `le-k8708jr`: 29 children, 20 live, mentioned by 30 tasks | sazed `sa-xtkbkm9`: 11 descendants, depth 2, all 11 filed this week |
| `seq` as a discriminator (sazed) | 253 of 263 live carry one; **140 distinct values** | 113 live tasks share a `seq` with another; 24 sit at ≥ 900 |
| state intervals (sazed) | doing: median **1.2 h**, p90 34.8 h (60 spans) · blocked: median 18 h, p90 **214 h** (6) | from the log alone; nothing authored |
| cost, release binary, sazed (606 tasks) | events 44 ms · graph 79 ms · mentions 90 ms · lineage 111 ms · hazard 163 ms · flow 267 ms · full pulse 0.7–0.9 s | wall time per `q` process including load; baseline `SELECT count(*)` is 12 ms |

Four readings, and they decide the shape of everything below:

1. **The first derivative is the finding.** Filed 380, closed 305, and the backlog widened on
   26 of 33 days. No state view says that. The portfolio's own BURNDOWN documents compute it by
   hand every refresh (*"40 closed, the store went 35 open → 13"* — subtracted against a number
   written in prose the week before), and the refresh proposal's `since` mock asks for exactly
   *"transitions: 3 open→doing, 14 doing→done, 4 →dropped … filed (18) closed (14)."* The
   `events` view is that mock as a table.
2. **The handoff is the loudest thing in the digest and the least checked.** 85 live tasks carry
   a handoff naming a closed task. The 09-05 review's F2/F3 — a sixty-line handoff promoted for
   days after both blockers it named had closed — is not one incident; it is the modal state of
   a handoff older than a week. `mentions` makes it a column.
3. **Work spawns work, and nothing counts the spawn.** A third of everything filed this week was
   discovered from something else; one task in the applied lab has a seven-deep chain of
   descendants; one task in sazed grew 25 descendants in a week while itself staying open and
   ready. The 09-05 review's mechanism — *finishable adjacent work filed instead of the work* —
   is a lineage shape, and `lineage` renders it as `spawned_w`, `spawn_depth`, `spawned_live`.
4. **Ordering has less to order than it looks.** In sazed, 253 of 263 live tasks carry a `seq`
   and only 140 values are distinct; `ready` returns 239 of 259 open. The prioritization proposal
   diagnosed *"every ticket carries policy"*; the placement view adds that the policy is not even
   unique per ticket. The metrics that will decide that proposal's rungs — the placed-by
   histogram, the hazard table, the triage mass, the `needs-behind` list — are all columns here.

---

## 2. The questions people answer by hand

Mined from `portfolio/BURNDOWN-2026-08-{21,25,31}.md`, `STATUS.md`, `ARC-2026-08.md`,
`PROPOSAL-meshwork-for-refresh-sessions.md`, `task-prioritization-notes.md`,
`DESIGN-graph-of-graphs.md`, `REVIEW-meshwork-continuity-failure-2026-09-05.md`, and
`docs/setup-cost-matrix.md`. Forty distinct hand-computed numbers were found; the ones the
store can answer alone are grouped here by the view that answers them. (Numbers needing git —
commits per period, unpushed counts, "zero commits while `doing`" — are a later rung, §9; numbers
needing transcripts — ramp minutes, tokens, live sessions — stay with meshwork-ui.)

| what was asked, in the author's words | where | answered by |
|---|---|---|
| *"transitions: 3 open→doing, 14 doing→done, 4 →dropped … comments: 11 (2 by Jon Rubin, 9 by claude) … filed (18) closed (14)"* | refresh-sessions §2.3 (`since` mock; today 11 tool calls) | `events` filtered by `at >= …` |
| *"40 closed, the store went 35 open → 13"* · *"+265 tasks and +209 closures in four days"* | BURNDOWN-08-25, STATUS §4 (hand-subtracted) | `flow`, `pulse.filed_w / done_w / backlog_delta_w` |
| *"162o/10d/110✓"* per repo, in one call | refresh-sessions §2.2 (`portfolio prime` mock; 8 `cd`+`prime` today) | `pulse`, one row per repo |
| *"`doing` with zero commits since date X"* · *"U1: a ninth period at zero"* · *"stale doing tasks accumulate silently"* | refresh-sessions §2.3, BURNDOWN-08-31 | `facts.idle_h`, `pulse.doing_stale`; the commit half is §9 |
| *"Option C — 24 days untouched since filing"* | BURNDOWN-08-25 (file mtime + memory) | `facts.idle_h`, `facts.age_h` |
| *"sazed has ten `doing` tasks, and they are not ten steps of one thing — they are lanes"* / *"you can't just look at next tasks btw there are multiple threads"* | refresh-sessions §2.1 (grep over 200 files) | `graph.lane`, `lane_size`; lane heads by min `place` |
| *"which lane is externally blocked"* | refresh-sessions §2.1 | `graph.needs_open_foreign`, `needs_unresolved` |
| *"45 asks filed, 37 answered (marasi 14/14 · tensoon 12/13 · sazed 11/17 …)"* | BURNDOWN-08-31 (hand tally) | `asks` grouped by `to_repo` |
| *"N asks unanswered, oldest D days"* · *"open 13 days … listed, unread, in every session digest"* | mw-r6g9bhe; review F4 | `asks.unanswered`, `age_h`; `pulse.asks_in_open / asks_in_oldest_d` |
| *"6 sessions have led with this and closed nothing"* | review F7 (impossible today) | `attention.touched_since_move` — the store-side proxy (§4.9); the transcript half stays with the UI |
| *"Cycle time … median 15.1 h, p90 123.9 h"* · *"70% of all task touches land in the task's first 24 h"* · closure hazard by age | setup-cost matrix (one-off script) | `facts.cycle_h`, `events`, `hazard` |
| *"effort as p10/p50/p90 — derived: the `## log` table has every open→doing→done transition"* | graph-of-graphs §5.1 | `spans`, `facts.service_h` |
| *"WIP headline — ρ/(1−ρ) dominates all sequencing"* | graph-of-graphs §5.2 | `pulse.doing_n` vs `[wip] cap` (MW-R14) |
| *"Cross-repo `needs` edges 8 → 14"* | BURNDOWN-08-21 (counted by hand) | `edges` where the repo segments differ; `graph.needed_by_foreign` |
| *"`dependents` (in-degree) ranked over unbuilt nodes is the real backlog"* · articulation points | atlas Q13, Q7; notes layer 1 | `graph.unlock`, `graph.depth`; `lineage.gravity` |
| *"a fix loop filing 2.43 new findings per one closed"* | ARC-2026-08 §6 | `pulse.filed_from_w` ÷ `done_w` |
| *"which ideas has the portfolio never once touched from any other plane"* | graph-of-graphs W5 / atlas Q5 | `lineage` rows with `gravity = 0` |
| *"39 of 48 open-task verifies silently green"* | ARC-2026-08 §6 | `facts.close_attempts`, and rung 5 (§9) |
| *"`prime` says '4 uncommitted task edits' — an accurate and useless sentence"* | refresh-sessions §2.4 | out of scope: git, not the store |

Three constraints found in the same documents bound the catalog, and the design keeps them:

- **"Counted, never as throughput"** (BURNDOWN-08-21, repeated 08-25: *"memory, never
  throughput"*). Flow counts are period deltas with denominators, never a velocity, never a
  target, never a trend line extrapolated. *"A burndown predicts sessions, not findings."*
- **"Mine what enforcement already writes; never ask an agent to log effort"**
  (refresh-sessions §5). Every column here derives from stamps the lifecycle already mints.
- **"Give me better inputs, not a draft"** (refresh-sessions §5). The output is numbers with
  denominators, never generated prose. And REQUIREMENTS §3 stands verbatim: no burndown chart
  in the tool, no estimates, no sprint semantics — the two scope rulings the prioritization
  proposal requests (§6 there) cover this proposal too, and §12 requests them jointly.

---

## 3. Why `prime` is weather-blind — the derivative gap

Everything in the digest today is **zeroth-order**: a count, a list, a task, a text. The
sections are correct and an agent reads them correctly. What an agent does not do, reliably, is
the arithmetic that turns state into weather:

| order | question | example today | who computes it now |
|---|---|---|---|
| 0 | what is there? | 259 open, 239 ready | `prime` |
| 1 | how is it changing? | +206 filed, −155 closed, backlog +47 this week; 26 of 33 days widened | BURNDOWN by hand |
| 1 | how long has it been like this? | median open age 22 d; 173 past 14 d; doing idle 4 d | nobody |
| 1 | what does this one release? | `sa-cytn77t` unlocks 3, depth 2 | `why` (one task at a time) |
| 2 | is the change changing? | filed last week 200 vs 59 the week before; closes 149 vs 39 | nobody |
| 2 | is attention converging or thrashing? | `sa-dnqvghr` touched by 3 sessions since it last moved, 12 d ago | nobody (F7) |
| 2 | is the thing I am about to trust still true? | handoff cites 8 ids, all closed | nobody (F2/F3) |
| 2 | is this task growing? | 25 descendants in 7 d, depth 4 | nobody |
| dist. | what is the residual close rate? | 2.5%/day past day 4 | mine_rank.py |

The prioritization proposal's baseline (§1 there) shows the consequence at the ordering layer:
half the live work is in the residual hazard regime and 96% of it is placed — *parked in
place*. This proposal's claim is narrower and prior: the digest cannot show a rate, an age, a
growth, or a staleness, so a session cannot see weather at all, and the six-session failure was
a session reading a clear sky off a state view. The fix is not more state. It is the four rows
of first- and second-order quantities that a human refresh computes by hand, computed once in
SQL and rendered in the space the digest already has.

---

## 4. The catalog — methodical, then pruned

### 4.1 The space

Every candidate metric is a point in a four-dimensional space, and the catalog was built by
walking it rather than by listing favourites:

- **entity** — task · lane · category · repo · portfolio · identity (session/author) · ask ·
  edge · mention
- **derivative order** — state (0) · rate, duration, age (1) · change of rate, growth,
  staleness, convergence (2) · distribution (median/p90/hazard) · structure (closure, components)
- **window** — now · since a stamp · last 7 d · last 28 d · all history
- **scope** — one repo (what a session sees) · the union (what the portfolio sees)

The pruning rules, applied to every candidate: (a) derivable from the six tables alone, from
stamps enforcement already mints; (b) not on REQUIREMENTS §3's list and not a throughput
target, a forecast, or a generated sentence; (c) actionable by an agent at session start, or by
the owner at a refresh, or by the prioritization ladder, or by a UI canvas — a metric with no
consumer is struck; (d) reproducible by a third-party reader from FORMAT.md; (e) cheap enough
for its consumer's budget (§7). Candidates that failed are listed in §4.12 with the reason.

### 4.2 The layering

Thirteen public views in three layers. Layer 1 normalizes; layer 2 is one row per entity;
layer 3 rolls up. Helper views (prefixed `f_`, `g_`, `m_`, `li_`, `fl_`, `hz_`, `p_`) exist to
keep each public definition readable and are registered but not contract.

```
clock ─┐
tasks ─┼─▶ events ─▶ spans ─┐
log ───┤                     ├─▶ facts ──┬─▶ asks       ─┐
comments ┘                   │           ├─▶ mentions   ─┼─▶ lineage
edges ──────────▶ graph ◀────┘           ├─▶ attention   │
                                         ├─▶ flow        ├─▶ pulse
                                         ├─▶ hazard      │
                                         └─▶ sessions   ─┘
```

Every column's derivation is the SQL in Appendix A. The sections below give, per view, the
question, the columns that matter, one consumer query that ran, and the reading.

### 4.3 `clock` and `events` — the stream every derivative is taken over

**`clock`** is one row: `now`, `today`, `source`. The binary registers it as a MemTable honouring
`MESHWORK_TODAY` (the determinism hook every golden already depends on, DESIGN §15.6), so a
view that says "age" is byte-stable under test. A third-party reader supplies its own row.
Without it every age-bearing view would embed `now()` and no golden could pin it.

**`events`** is the union of everything with a stamp: one `created` row per task, one row per
`## log` entry (`transition`, `close-attempt`, or `note`), one per comment. Columns: `gid`,
`repo`, `kind`, `stamp` (as written), `at` (parsed; NULL when nonconforming — FORMAT.md's two
forms only), `actor` (the comment author, or the `claimed by <author>` note on a `→doing`
line — the only two places the store records who), `from_status`, `to_status`, `note`, `ord`.

This is the refresh proposal's `since` verb as a table, and the answer to "what changed":

```sql
SELECT kind, coalesce(from_status || '→' || to_status, '') AS transition, count(*) AS n,
       count(DISTINCT actor) AS identities
FROM events WHERE at >= TIMESTAMP '2026-08-30' GROUP BY 1, 2 ORDER BY n DESC
```

The stamp guard is the one idiom every view repeats and FORMAT.md must publish: DataFusion's
`to_timestamp` *errors* on a non-date token (the log grammar guarantees `date='fixed'` will
occur), so parsing is always `CASE WHEN regexp_like(x, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$')
THEN to_timestamp(x, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END`. Nonconforming stamps project `NULL
at` and are invisible to every duration — the cost FORMAT.md already assigns to minting one.

### 4.4 `spans` and `facts` — one row per task, every lifecycle scalar

**`spans`**: each transition opens an interval in `to_status` that the next transition closes;
open intervals run to `clock.now`. `hours` per span. This is where "blocked for 214 h" and
"doing for 1.2 h" come from, and it is the M-SERPT attained-service clock the prioritization
proposal's MW-R11 names.

**`facts`**: the per-task vital signs. Times: `created_at`, `first_doing`, `last_doing`,
`done_at`, `terminal_at`, `last_transition`, `last_activity`. Durations, all in hours:
`age_h` (to now for live tasks, to terminal for closed — so it is lifetime for the hazard),
`queue_h` (created → first doing), `service_h` (first doing → done, only when ordered),
`cycle_h` (created → done), `blocked_h`, `doing_h` (summed spans), `idle_h` (since last
activity). Counts: `touches`, `comments`, `actors`, `claims`, `close_attempts`, `reopens`,
`blocks`. Flags: `live`, `has_verify`, `has_category`, `has_seq`, `waived`, `has_handoff`,
`is_ask`. Scoped to loaded stores via `repos`, so the thin foreign rows a single-repo session
injects for dependency resolution never count as tasks.

```sql
SELECT state, count(*) AS n, round(median(hours), 2) AS med_h,
       round(approx_percentile_cont(hours, 0.9), 1) AS p90_h
FROM spans WHERE state IN ('doing', 'blocked') GROUP BY state
-- sazed: doing 60 spans, median 1.18 h, p90 34.8 h · blocked 6, median 18.2 h, p90 214.5 h
```

`close_attempts` deserves its own sentence: a `close attempt — verify exit N` line is minted by
every failed close, so "how many times did a session try to close this and fail" is already in
the log — 29 such attempts portfolio-wide this week, sazed's `sa-b9k121w` and three others at
two each. It is the cheapest honest signal of a verify that does not match the work, and no
surface shows it.

### 4.5 `graph` — structure over live `needs`

One row per task: `unlock` (live tasks that transitively need it), `depth` (longest live
chain below it), `place` (`seq` or 999999), `inherit` (best place among same-repo live
dependents), `needs_behind` (`inherit < place`), `lane` and `lane_size` (connected component
over live `needs` only — `parent` umbrellas join everything, as the prioritization baseline
showed), `needs_open`, `needs_unresolved`, `needs_open_foreign`, `needed_by_open`,
`needed_by_foreign`, `live_children`, `discovered`, and `ready` (the §5 predicate as a
boolean, so `SELECT count(*) FILTER (WHERE ready)` is one line).

These are MW-R6 and MW-R7 of the prioritization proposal, as a view. On this repo it
reproduces that proposal's headline finding in one row — `mw-cvw8` at place 900 inherits 150,
unlocks 5, depth 5, in the one 6-member lane — and adds the contradiction's whole chain:

```sql
SELECT id, status, unlock, depth, place, inherit, needs_behind, lane_size, ready
FROM graph WHERE unlock > 0 ORDER BY unlock DESC, id LIMIT 4
-- mw-cvw8 900→150 unlocks 5 · mw-wm9w 910→150 unlocks 4 · mw-vmzg 920→150 · mw-ws31 930→150
```

Two implementation facts that shaped the SQL and belong in FORMAT.md's derivation notes. The
transitive closure is a recursive CTE with a depth bound (64) — DataFusion 51 applies no
cycle guard, `needs` is cycle-linted, and a diamond multiplies paths but `count(DISTINCT)`
absorbs it. The connected components are **synchronous min-label propagation** with a
convergence stop: an undirected recursive walk explodes (every edge is a 2-cycle; the first
attempt ran two minutes on sazed and was killed), and a fixed 64-iteration propagation costs
~5 s because every iteration pays planning. The recursive term therefore emits rows only while
some label still changed in the previous round (`sum(changed) OVER () > 0`), converging in
diameter+1 rounds — 79 ms on sazed for the whole `graph` view.

### 4.6 `asks` — obligations across the boundary

One row per task carrying `to:`: `from_repo`, `to_repo`, `to_ref`, the ask's own `age_h` and
`idle_h`, and the answer if any (`answer_gid`, `answer_status`, `answered_at` = the answering
task's creation, `answer_done_at`, `response_h`), plus `unanswered` = no non-dropped answer and
the ask itself non-terminal. This is `addressed::inbox` as SQL, with age — mw-r6g9bhe's
deliverable is one `pulse` column over it — and with the *outbound* direction the inbox never
shows: what this repo owes others.

```sql
SELECT gid, to_repo, round(age_h / 24.0, 1) AS age_d FROM asks
WHERE unanswered ORDER BY age_h DESC LIMIT 4
-- sazed owes: sa-b9y0pxe → meshwork 7.0 d · sa-h4mct2a → applied-lab 4.5 d · sa-s7c9186 → marasi 3.8 d …
```

The baseline's sharpest structural fact: every store's `asks_in_open` is **0** in a single-repo
load, because an inbound ask lives in the sender's store. `sa-b9y0pxe` is 7 days old and
addressed to this repo; this repo's own tables cannot see it. The inbox already reads the union
for exactly this reason (the one sanctioned full-union read inside a single-repo verb, owner
lane 2026-08-17); §6.2 keeps that arrangement and extends it to the pulse's asks line only.

### 4.7 `mentions` — every id named anywhere, resolved

The user's second interest, generalized. Tokenize `handoff`, `body`, every comment, and every
log note on the identifier alphabet; keep tokens that resolve to a task (`repo#id`, or a bare
`id` qualified with the mentioning task's repo); one row per (source task, field, referenced
task): `times`, `first_at` (for comments and log notes, which carry stamps), the referenced
task's **current** `ref_status` and `ref_terminal_at`, the source's `last_activity`,
`moved_since_activity` (the reference went terminal *after* the source last moved — the F3
detector), and `edge_backed` (a declared edge of any kind exists between the two, either
direction).

```sql
SELECT src_gid, count(*) AS terminal_refs, round(f.idle_h / 24.0, 1) AS idle_d
FROM mentions m JOIN facts f ON f.gid = m.src_gid
WHERE m.field = 'handoff' AND m.ref_status IN ('done', 'dropped') AND f.live
GROUP BY src_gid, f.idle_h ORDER BY terminal_refs DESC LIMIT 2
-- sazed: sa-72rrcck 8 closed ids, idle 4 d · sa-0bbnpw7 6 closed ids, idle 15 d
```

Two readings of the numbers. First, **handoffs rot at the rate the store closes tasks**: 155
handoff mentions in sazed, 109 point at terminal tasks. That is not a flaw in any one handoff;
it is what a free-text field does when nothing re-reads it, and the fix is display (the UI's D3
chip; `prime`'s "cites N closed" tail in §5) plus one write-path change — `set --handoff` should
mint a log line (`- <stamp> handoff`), because today the field carries no timestamp at all and
`moved_since_activity` has to use the task's last activity as an upper bound. Second,
**mentions are the undeclared graph**: 928 resolved mentions have no edge behind them, 107 of
them live→live. Those are candidate `relates`/`needs` edges a lint can offer (`implicit-edge`,
report only) and a UI can draw dashed. The tokenizer is deliberately naive — one regex, no
pattern language — and resolves only what exists; a false positive requires a word that
happens to equal a minted id.

DataFusion notes for the implementer: `unnest` works in a projection but not as a lateral
join, `regexp_match` has no global flag, and the resolution join must be an equality on a
normalized `repo#id` — an `OR` of two conditions is a nested loop and cost 1.4 s on sazed
before it was rewritten; 90 ms after.

### 4.8 `lineage` — spawn trees, umbrellas, gravity

The user's first interest. One row per task:

- **spawn** (transitive `discovered-from`, child→origin): `spawned_direct`, `spawned_total`,
  `spawned_live`, `spawned_done`, `spawn_depth`, `last_spawn_at`, `spawned_w` (descendants
  created in the window — the *growth rate*).
- **umbrella** (transitive `parent`): `children_direct`, `descendants_total`,
  `descendants_live`, `descendants_done`, `umbrella_depth`, `last_child_at`, `children_w`.
- **inbound edges**: `needs_in`, `relates_in`, `answers_in`, `referrers_edges` (distinct
  sources).
- **mentions**: `mentioned_by`, `mentioned_by_unlinked`, `mentioned_in_talk` (comments/log).
- **`gravity`** = spawned + descendants + edge referrers + mentioners: how much of the store
  points at this task. A display quantity for the UI's radius and a triage cue; never a rank
  input (MW-R8's fence applies).

```sql
SELECT l.id, l.status, l.spawned_direct, l.spawned_total, l.spawned_live, l.spawn_depth, l.spawned_w
FROM lineage l WHERE l.spawned_total > 0 ORDER BY l.spawned_w DESC, l.spawned_total DESC LIMIT 2
-- sazed: sa-72rrcck open · 8 direct · 25 total · 1 live · depth 4 · 25 this week
-- applied-lab: ar-pwtdbx5 done · 1 direct · 20 total · 4 live · depth 7 · 20 this week
```

What the shapes mean, and why an agent should see them. A **growing spawner** (`spawned_w`
high, itself live) is the F7 pattern from the inside: work being generated *around* a task
instead of into it — `sa-72rrcck` grew 25 descendants in a week, is still open and ready, and
carries the handoff with eight closed citations. A **deep chain** (`spawn_depth` 7 from a single
direct child) is a rabbit hole with a shape: each level was discovered from the last, and the
origin — the lab's profiling task — is what the chain is actually about. An **umbrella with
live descendants** (`le-k8708jr`, 29 children, 20 live) is the parent-container the `ready`
predicate correctly hides, and its `descendants_done / descendants_total` is the progress bar
the digest cannot draw. And the ratio `filed_from_w / done_w` per repo — applied-lab filed 41
descendants against 49 closes this week, 76% of its filing — is ARC-2026-08's *"2.43 findings
per one closed"* as a standing column instead of a one-off query.

Recursion here is over `discovered-from` and `parent`; `parent` is cycle-linted, `discovered-from`
is not — the 64 bound is the guard, and a cycle would surface as `spawn_depth = 64`, which lint
should report (`discovered-cycle`).

### 4.9 `attention` and `sessions` — who touched what, and did it move

**`attention`** (live tasks only): `touched_since_move` = distinct identities with a comment or
claim on the task *after* its last transition, and when the first of those was. It is the
store-side half of the review's F7 (*"6 sessions have led with this and closed nothing"*): the
store cannot see a `show`, but it can see that three different session identities wrote on a
task that has not changed status in twelve days.

```sql
SELECT a.id, a.touched_since_move, round(a.idle_h / 24.0, 1) AS idle_d
FROM attention a ORDER BY touched_since_move DESC, idle_h DESC LIMIT 2
-- sazed: sa-dnqvghr 3 identities, idle 11.9 d · sa-kc30eh5 3, idle 6.6 d
```

**`sessions`**: per self-professed identity per repo — `first_seen`, `last_seen`,
`tasks_touched`, `comments`, `claims`. Identities are what the shim mints (`claude
(session_…)`), so this joins to transcripts on the suffix exactly as the UI proposal's §5.3
describes. Closes are not attributed (the `→done` note carries a sha, not an author), so
"closes per session" is a git-joined metric (§9), and the view says so by not having the column.

### 4.10 `flow` and `hazard` — the two distributions worth having

**`flow`**: one row per repo per calendar day since the store's first task: `filed`, `done`,
`dropped`, their cumulatives, `backlog`, `backlog_delta`, `doing_eod` (doing spans open at end
of day). It is the UI's burn-up and the BURNDOWN's "widened on N of M days", computed in O(n)
by bucketing events per day and running window sums (a first version joined days × tasks and
cost 463 ms on sazed; 267 ms now, most of it the calendar generation).

```sql
SELECT date_trunc('week', day) AS wk, sum(filed) AS filed, sum(done) AS done, max(backlog) AS backlog_eow
FROM flow GROUP BY 1 ORDER BY 1 DESC LIMIT 2
-- sazed: week of 08-31 filed 200 · done 149 · backlog 279 — the week before 59 · 39 · 231
```

Reported as counts with the window named — never as a rate per session, never extended past
today. The second derivative (this week vs last) is one `lag()` away and is shown in `stats`,
not in `prime`.

**`hazard`**: per repo and age-day bin, `at_risk` (tasks open at that age and observable
through the end of the bin), `done_in`, `dropped_in`. Pooled by summing numerators and
denominators across repos — the honest way to pool — it is the prioritization proposal's MW-R10
table, competing risks and equalized follow-up included, as a view instead of a script. The
survival curve is `exp(sum(ln(1 - h)) OVER (ORDER BY age_d))` in the consumer.

```sql
SELECT age_d, sum(at_risk) AS at_risk, sum(done_in) AS done_in,
       round(sum(done_in) * 1.0 / sum(at_risk), 3) AS h_done
FROM hazard GROUP BY age_d ORDER BY age_d
-- pooled: 0 → 0.357 (1,171) · 1 → 0.081 (718) · 4–15 → mean 0.025 · 21 → 0.009 (234) · 28 → 0.000 (129)
```

### 4.11 `pulse` — one row per repo, the weather

Everything above reduced to the numbers a session or a refresh reads first. Columns, all with
`_w` meaning "in the configured window" (7 days by default):

| group | columns |
|---|---|
| state | `open_n doing_n blocked_n done_n dropped_n` |
| flow | `filed_w done_w dropped_w started_w backlog_delta_w filed_from_w` |
| age | `doing_stale open_age_med_d past_triage parked_in_place` |
| structure | `ready_n unlockers lanes_multi needs_behind_n blocked_foreign blocked_unresolved owed_foreign` |
| obligations | `asks_in_open asks_in_oldest_d asks_out_open asks_out_oldest_d` |
| friction | `close_attempts_w reopens_w blocks_w waived_n no_verify no_seq no_category` |
| attention | `comments_w actors_w thrash_n` |
| text & lineage | `handoff_stale_n unlinked_mentions implicit_live_n spawners_w spawned_w_max umbrellas_w` |

`SELECT * FROM pulse` at portfolio scope is the refresh proposal's `portfolio prime` mock —
one line per repo — as a table, and §5 renders the repo's own row into `prime`.

### 4.12 Considered and struck

| candidate | why not |
|---|---|
| velocity, throughput per session, burndown projection, forecast dates | "counted, never as throughput"; REQUIREMENTS §3; *"a burndown predicts sessions, not findings"* |
| aging boost, anti-starvation term | MW-R12: age is a triage signal, never a rank input; the hazard says so |
| flow efficiency (`service / (queue + service)`) | measured 0.02 with no spread (prioritization §1); a constant is not a metric — `queue_h` and `service_h` stay as columns |
| betweenness, articulation points | MW-R8: MAY, never a rank input; on a graph this sparse every chain interior is a cut vertex; `unlock` and `depth` carry the same signal |
| tokens per task, ramp minutes, context at first act | live in transcripts; the matrix mines them out of band; rung 5 of the prioritization ladder decides |
| commits per task / per period, unpushed, dirty | git, not the store; a later rung (§9) via `git log --grep=<id>`, the `commits:` tail's own method |
| category-level cost estimates (`p̂`) | authored or token-derived; MW-R21 keeps the column out until rung 0's token half reports |
| "touched, not advanced" from reads | the store cannot see `show`; `attention` is the write-side proxy and says so; the read side is the UI's M5 |
| generated summary sentences | *"give me better inputs, not a draft"* |
| a `since <ref>` diff of two store commits | FORMAT.md already says git history is the change stream; `events` gives the *dated* stream; a commit-range diff is a wrapper over `git diff` and needs a §6 ruling nobody has asked for |
| per-actor closes | unattributed in the log; would need a minted author on `→done`, a format change this proposal does not request |

---

## 5. Surfaces

### 5.1 `q` and `portfolio q` — the views land as tables (rung 1, no surface change)

`tables::session_for` registers the thirteen public views and their helpers after the six
tables and the `category_matches` UDF, by executing the Appendix A `CREATE VIEW` text in the
same runtime `run_query` already owns. The `q` error path's *queryable tables* line gains them.
Nothing else changes: MW-C1 holds (SQL is the only query surface), MW-C3's envelope is
untouched, and every metric in this document is available to every session the day this lands.

### 5.2 `prime` — the pulse block (rung 2, behaviour inside an existing section)

Five lines at the top of **weather**, each clamped to the 160-byte line cap, the block capped at
800 bytes, every number carrying its denominator in the line. The repo's `pulse` row rendered:

```
weather:
- flow 7d: filed 206 (49 from other tasks) · done 155 · dropped 4 · backlog 279 (+47) · widened 6 of 7 days
- queue: ready 239 of 259 open · doing 2 (2 stale) · open age med 22d · 173 past 14d: engine 60 · (none) 32 · method 25 · +7
- graph: 10 lanes · unlocks most sa-cytn77t (3) · needs-behind 4 · blocked on foreign 0 · owed to others 0
- asks: owed to me 6 (oldest 14d) · owed by me 10 (oldest 7d)
- friction 7d: close attempts 15 · reopens 3 · thrash 3 (sa-dnqvghr: 3 sessions, 12d) · handoffs citing closed tasks 69
- doing sa-… [claimed: …] [stale: 4d] — …            ← today's weather lines follow, unchanged
```

Every figure is sazed's measured `pulse` row except the inbound half of the asks line, which a
single-repo load cannot compute (§6.2): six unanswered asks and a 13-day-old oldest are the
09-05 review's F1/F4 figures for sazed, a day older here.

Rules. (1) A line whose every count is zero is omitted — a quiet store gets a quiet block.
(2) The asks line is the union read the inbox already performs, and it **is** mw-r6g9bhe's
headline; the addressed section below it keeps the ids. (3) The queue line's triage rollup is
MW-R13's line — this proposal renders it, that proposal owns its semantics (`[decay]
triage_days`, exempt labels). (4) When `[wip] cap` is set (MW-R14) the queue line reads
`doing 10 (cap 3 — OVER)`. (5) The 6,144-byte budget is unchanged and gated; if the block would
push the digest over, `also ready` drops rows first (to a floor of 3), then the pulse drops its
friction line, then its graph line, each cut loud with the existing `… truncated` tail. On the
two largest stores today the block fits with room: sazed's digest is 4,547 bytes, this repo's
2,715.

The next-task block gains one tail line when `mentions` says so: `  cites 8 closed tasks
(sa-ct27nwa, sa-jzdvm2e, +6) — handoff may be stale`. That is F2/F3 in one line, at the one
place the session is about to trust a handoff.

**Where the numbers come from at runtime.** `prime` is gated at 100 ms cold at 1K tasks
(DESIGN §14 row 7). A DataFusion query costs 10–20 ms to plan on any store and the pulse's
inputs cost ~120 ms on sazed even without mentions and lineage; through SQL the block would
break the gate on real stores. The pulse block is therefore computed in Rust over the tasks
`prime` has already parsed — the same way today's weather lines are — and **the SQL views are
the specification**: a differential test runs `SELECT * FROM pulse` against the fixture corpus
and asserts equality with the Rust pass, column by column, under `MESHWORK_TODAY`. Two
implementations of one definition, pinned by the gate; the SQL is the one FORMAT.md publishes
and the one a third-party reader implements. (The alternative — a `.cache/pulse.json` keyed on
the content hash FORMAT.md already specifies — stays open as rung 6 if the Rust pass ever
outgrows the budget.)

### 5.3 `stats` — the verb (rung 3, §6 ruling required)

```
stats [--window 7d|28d] [--json]          # this repo: the pulse row, then the tables below
portfolio stats [--window …] [--json]     # one pulse line per repo + totals, then the tables
```

Text mode prints the pulse block, then: the weekly flow table (last 6 weeks, with
week-over-week deltas), the pooled hazard table (bins 0–14, 21, 28, with the survival column),
state-interval distributions (`spans`), the lanes table (head, size, ready members, externally
blocked), the top-10 by `gravity` / `spawned_w` / `touched_since_move`, the mention health
line (handoffs citing closed tasks; live→live unlinked mentions), and the placement histogram
(seq'd / distinct / collisions / parked; `placed_by` once the prioritization proposal's rung 2
lands). Every table is a canned `SELECT` over a view — the `ready`/`blocked`/`search` pattern,
not a language — and `--json` emits the view rows in the MW-C3 envelope. The prioritization
proposal asked for the same numbers as `lint --stats`; this proposal recommends `stats` because
lint reports *findings* and these are *measurements*, and asks the two be ruled together.

### 5.4 `lint` — findings the views make cheap (rung 2/3)

Warnings, never errors, each one query: `handoff-cites-closed` (a live task's handoff names a
terminal id — 85 today), `implicit-edge` (live→live mention with no edge, report only — 107),
`discovered-cycle` (`spawn_depth` hit the bound), `seq-collision` (two live tasks, one `seq`,
one repo — sazed 113), `close-attempts` (≥ 2 failed closes on a live task). The prioritization
proposal's `needs-behind`, `open-decayed`, `wip-over-cap` are the same shape over the same
views.

### 5.5 `show`

Two derived lines after `commits:`: `lineage: spawned 25 (1 live, depth 4) · children 0 ·
mentioned by 7` and `cites: 8 closed (…)`. Both from views, both capped.

---

## 6. Portfolio scope

### 6.1 One code path

Every view carries `repo` and every rollup groups by it, so the union session (`portfolio q`)
computes the identical views over nine stores with no second definition — the MW-G3 discipline
the six tables already follow. `pulse` at portfolio scope is nine rows; `hazard` pools by
summing; `flow` partitions by repo and a portfolio burn-up is `sum() OVER (ORDER BY day)` of
the per-repo rows.

### 6.2 What only the union can see

Three families are wrong-by-construction in a single-repo load, and the views say so rather
than guess: **asks** (inbound = 0 per repo; the union shows 32 open obligations), **cross-repo
lanes** (`needed_by_foreign` is 0 everywhere because the foreign dependent is not loaded; the
union shows the 39 cross-repo `needs` the UI proposal counted), and **cross-repo mentions** (a
sazed handoff naming `marasi-applied-r-and-d#ar-hcvyxy5` resolves only when marasi's store is
present). `prime` already performs one sanctioned union read for the inbox; the pulse's asks
line rides that same read and nothing else in `prime` widens scope.

### 6.3 Reads must not prune

`portfolio q` autoprunes `sequence.md` (owner-ruled, mw-chcqk6g). Every analysis this proposal
rests on avoided the verb for that reason — mine_rank.py per-repo, the UI proposal's one
`portfolio ready` pruned an entry while measuring. An analytics surface that mutates the
overlay on read is the atlas's *"reading state can mutate state"* hazard verbatim. §12 requests
a narrowing: `next` and `ready` prune (they consume the overlay); `q` and `stats` never do.

### 6.4 The `[stats]` table in `config.toml`

```toml
[stats]
window_days = 7        # the _w window; the binary substitutes it into p_win at registration
```

Plus the tables the prioritization proposal defines and this one reads: `[decay] triage_days`,
`[wip] cap`. Data, never executed; unknown keys warn as today.

---

## 7. Budgets, determinism, contract

- **Cost.** Release binary, wall time per `q` process, sazed (606 tasks, 1,967 events): the
  events layer 44 ms; facts 48; graph 79; asks 60; mentions 90; lineage 111; attention 105;
  hazard 163; flow 267; the full pulse 0.7–0.9 s; the pulse without mentions/lineage/flow
  120 ms. Eight stores under 0.45 s for any single probe. `q`/`stats`/UI consumers have no
  latency gate; `prime` does, hence §5.2's Rust pass. `check-perf.sh` gains one row: `stats`
  cold ≤ 1 s at 1K tasks, the MW-C4 portfolio number.
- **Determinism.** `clock` honours `MESHWORK_TODAY`; view output is a pure function of store
  content plus that stamp, so goldens pin it. The conformance corpus gains
  `expected/views/<view>.json` for the golden store, self-checkable by any reader.
- **Contract.** FORMAT.md gains a §Views section: the thirteen names, their columns, the stamp
  guard, the recursion bounds (64) and the label-propagation stop, and the Appendix A SQL
  verbatim. Additive under the minting-rule idiom — no on-disk byte changes, no `format` bump,
  no `schema` bump; a reader that ignores views loses nothing it had. Views are versioned by
  the binary version in the envelope like every other output.
- **Trust.** Views read text; they never execute it. `verify:` is a string in `facts.has_verify`
  and nowhere else. Every rendered line passes `cli::sanitize` like every other text seam.
- **Zero network.** Nothing here leaves the loaded stores. The union is local checkouts.
- **DataFusion version.** The views run on the pinned 51.0.0 and need no upgrade. Checked
  against the raw changelogs, source and issue tracker through the newest release, 55.0.0
  (crates.io 2026-08-18; MSRV `rust-version = "1.94.0"`, up from 1.88.0; arrow 57 → 59.2):
  of the six limitations the SQL works around, one is fixed — the literal-anchored recursive
  CTE internal error, issue #22249, closed by PR #22476 in 55.0.0 (`dev/changelog/55.0.0.md`
  line 360); the views never hit it because every recursion anchors on a table. The other five
  stand at 55.0.0: outer references in a LATERAL/`unnest` projection (`lateral_join.slt` §6
  still asserts the `OuterReferenceColumn` error), `EXISTS`/`IN` in a projection (issue #23022,
  open), no global `regexp_match` and no `regexp_split_to_array`, `HH:MM` stamps rejected by
  arrow-cast's parser and `to_timestamp` erroring rather than nulling on a bad token, and the
  unique-name rule for `UNION ALL` branches (a fix, #21126, was reverted in 54.0.0). The
  embedder's seven call sites (`SessionContext::new`, `register_batch`, `register_udf`,
  `create_udf`, `values_to_arrays`, `sql`, `array_value_to_string`) are unchanged in signature
  at 55.0.0, so an upgrade is a toolchain and compile-time decision, not an analytics one.

---

## 8. What this does for the prioritization proposal

That proposal's floor needs inputs the store does not compute yet; every one of them is a column
here, so its rungs 1 and 3 shrink to "consume the view" and its rung 0 script retires:

| prioritization requirement | provided by | note |
|---|---|---|
| MW-R6 `unlock`, `depth`, `inherit` | `graph` | same-repo inheritance as specified; cross-repo waits on the scale ruling |
| MW-R6b `needs-behind` lint | `graph.needs_behind` | 9 today; this repo's four-task mirror chain is one lane |
| MW-R7 `lane`, lane head | `graph.lane`, `lane_size`; head = `min(place)` per lane | 25 multi-member lanes portfolio-wide |
| MW-R9 `service_h`, `queue_h`, `age_h` | `facts` | plus `cycle_h`, `blocked_h`, `doing_h`, `idle_h` |
| MW-R10 the hazard table | `hazard` | competing risks, equalized follow-up; pooled by summing |
| MW-R11 attained service for `doing` | `spans`, `facts.doing_h` | the M-SERPT clock |
| MW-R13 triage rollup by category | `facts` where `live AND age_h ≥ triage_days·24`, grouped | rendered in the pulse's queue line |
| MW-R14 WIP line | `pulse.doing_n`, `doing_stale` vs `[wip] cap` | |
| MW-R3 placed-by histogram | `SELECT placed_by, count(*) … ` once rung 2 adds the column | `stats` prints it beside `seq` collisions |
| MW-R17 auditability | the views *are* the projection's derived columns, documented in FORMAT.md | see the reconciliation below |
| rung 0 `mine_rank.py` | `stats --json` | the baseline becomes a verb output; the Sidney experiment stays a script |

Two reconciliations that proposal should adopt. First, MW-R17 says every derived quantity is a
*column in the §4 projection*; a view joined 1:1 on `gid` satisfies the intent (auditable with
`q`, reproducible from FORMAT.md) without growing `tasks` from 19 columns to 40, and keeps the
raw projection raw — recommend the wording "a column of `tasks` or of a FORMAT.md view".
Second, this proposal adds a placement metric that one did not have: **`seq` collisions**. In
sazed 253 live tasks carry a `seq` and only 140 values are distinct, so the per-task override
is not even a total order among the tasks it places; the tie-break falls to `created`. That is
evidence for bands (declare the tier once) of a different kind than the histogram — not that
`seq` carries policy, but that it has stopped carrying *order*.

---

## 9. What this does for meshwork-ui

The UI proposal's §5.4 posture — *"every derived signal recomputed per load"* — assumes the UI
computes unlock mass, depth, readiness, ask ages and the burn-up itself, as a second
implementation. With the views published in FORMAT.md, the UI has a choice it did not have:

- **Feed, not reimplementation.** `portfolio q --json "SELECT * FROM facts"` (and `graph`,
  `asks`, `mentions`, `lineage`, `flow`, `sessions`) is the whole analytics layer as typed JSON,
  from the pinned binary the UI already shells for writes. M0/M1 (the reader and the registry
  walk) keep their conformance value; M2/M5's *derived* quantities need not be re-derived.
- **Conformance widens.** If the UI does implement the views itself, the corpus's
  `expected/views/*.json` is the test, and any divergence is a meshwork bug by FORMAT.md's
  precedence rule — the proposal's own argument for M0, extended to the analytics.
- **Open call 2 is answered.** The ask-ageing rule lives in `asks`/`pulse`; the UI's ledger
  and `prime`'s asks line read the same column.

Per canvas, what the views add:

| canvas | today's derivation | with the views |
|---|---|---|
| Mesh (radius, dot area) | transitive dependents, depth | `graph.unlock`, `depth`; **`lineage.gravity`** as an alternative radius — what the store *talks about*, not only what it *needs* |
| Mesh (edges) | the five edge kinds | plus `mentions` where `NOT edge_backed`, drawn dashed — the undeclared 107 live→live strands |
| Lanes | components over `needs` | `graph.lane`, `lane_size`, `needs_open_foreign` for the `EXTERNALLY BLOCKED` tag |
| Inspector (D3 chips) | resolve `repo#id` in text | `mentions` rows for the task: status chip, `moved_since_activity`, `edge_backed` |
| Inspector (new) | — | the lineage strip: spawned / live / depth / this week; umbrella progress `descendants_done / total`; `close_attempts` |
| Ledger (asks) | join over `to:`/`answers:` | `asks` with `response_h` — how long the portfolio takes to answer |
| Ledger (thrash) | transcripts + `git log --grep` | `attention.touched_since_move` as the store-side count beside the transcript-side one |
| Ledger (burn-up) | `created` + `→done` | `flow` per repo, and `doing_eod` as a third line |
| Rail | transcripts | `sessions` gives the store-side `tasks_touched`, `claims`, `first_seen`/`last_seen` per identity |
| a new canvas: **Lineage** | — | spawn forests from `lineage` + the `discovered-from` edges: each origin a root, descendants by depth, live ones lit — the seven-deep lab chain drawn |

---

## 10. Requirements (proposed; `MW-S*`, RFC 2119)

### A. The view layer
- **MW-S1 (MUST)** The binary registers, in every SQL session, a one-row `clock` table honouring
  `MESHWORK_TODAY` and the thirteen public views `events`, `spans`, `facts`, `graph`, `asks`,
  `mentions`, `lineage`, `attention`, `sessions`, `flow`, `hazard`, `pulse` (and `clock`), defined
  by the SQL in FORMAT.md §Views verbatim. Helper views MAY be registered; only the thirteen are
  contract.
- **MW-S2 (MUST)** Every view is a pure function of the six tables and `clock`. Nothing is
  persisted; nothing is authored; no view executes text.
- **MW-S3 (MUST)** Stamps parse only in FORMAT.md's two conforming forms; nonconforming stamps
  project `NULL` and are invisible to every duration. Views scope task rows to loaded stores via
  `repos`.
- **MW-S4 (MUST)** Recursive derivations carry a depth bound of 64; component labelling stops on
  convergence. A bound reached is a lint finding (`discovered-cycle`), never a silent cap.
- **MW-S5 (MUST)** `[stats] window_days` (default 7) is the only parameter; the binary substitutes
  it into the published SQL at registration and `stats --window` overrides per call.

### B. Surfaces
- **MW-S6 (MUST)** `prime` renders the repo's `pulse` row as ≤ 5 weather lines, ≤ 800 bytes,
  zero-lines omitted, every count with its denominator, cut loud under the existing budget; and a
  `cites N closed tasks` tail on the next block when `mentions` reports one.
- **MW-S7 (MUST)** The pulse rendered by `prime` equals `SELECT * FROM pulse` for the same store
  and `MESHWORK_TODAY`, enforced by a differential test in the gate.
- **MW-S8 (SHOULD)** A `stats [--window] [--json]` verb and `portfolio stats` print the tables in
  §5.3 as canned `SELECT`s over the views. **§6 ruling required.**
- **MW-S9 (SHOULD)** `lint` gains `handoff-cites-closed`, `implicit-edge`, `discovered-cycle`,
  `seq-collision`, `close-attempts` — warnings.
- **MW-S10 (SHOULD)** `set --handoff` mints a log line (`- <stamp> handoff`) so a handoff has an
  age; `show` gains the two derived lines of §5.5.

### C. Fences
- **MW-S11 (MUST)** No metric is a velocity, a target, or a forecast. Flow is reported as counts
  in a named window; no line is extrapolated past `clock.today`.
- **MW-S12 (MUST)** No metric enters the ready order. Ordering is the prioritization proposal's
  key; this proposal supplies columns to it and renders them.
- **MW-S13 (MUST)** No generated prose. Output is numbers, ids, and the owner's own words
  (titles, reasons, handoffs) — never a sentence the tool composed about them.
- **MW-S14 (MUST)** Reads never prune. `q`, `stats`, and `portfolio q/stats` do not touch
  `sequence.md`. **Ruling required** (narrows mw-chcqk6g).
- **MW-S15 (MUST)** Zero network; local checkouts only; `prime`'s scope widens by nothing beyond
  the union read the inbox already performs.

---

## 11. Build ladder (each rung independently useful; red first; each with its verify)

| rung | lands | verify | gated on |
|---|---|---|---|
| **0 — the spec** | `scripts/mine_views.py` — **landed beside this proposal, untracked**, the rank baseline's sibling: holds every view as SQL, runs one probe per public view against every registered store through per-repo `q`, never a portfolio verb, and prints the pulse rows, the pooled hazard, and the backlog-day count; `--sql` emits Appendix A, `--check` exits 0 iff every probe on every present store does. Then FORMAT.md §Views with that SQL; `fixtures/conformance/expected/views/*.json` blessed from the golden store; DataFusion notes (guard, bounds, no lateral unnest, no EXISTS in projections, unique union names) | `python3 scripts/mine_views.py --check` exit 0 (observed 2026-09-06, nine stores, 108 probes) · `run cargo test conformance::views_golden` | nothing |
| **1 — registration** | `clock` MemTable; views registered in `session_for`; `q` error path lists them; `[stats] window_days` | `run cargo test tables::views_registered` + `sh -c './docs/meshwork/meshwork q "SELECT count(*) FROM pulse"'` exit 0 | 0 |
| **2 — prime pulse** | the Rust pass; the five lines; the `cites` tail; the differential test; goldens re-blessed with a reviewed diff; `check-perf.sh` unchanged and green | `run cargo test e2e::prime_pulse_matches_view` | 1 |
| **3 — lint findings** | the five warnings of §5.4; `set --handoff` log line; `show` lines | `run cargo test lint::handoff_cites_closed` + `run cargo test lint::seq_collision` | 1 |
| **4 — `stats`** | the verb, `portfolio stats`, `--json`; `check-perf.sh` gains the 1 s row; `mine_rank.py` retired to `stats --json` + the Sidney script | `run cargo test e2e::stats_tables` + `check-perf.sh` green | §6 ruling |
| **5 — git-joined metrics** | per-task `commits` (count, last sha date) via the `commits:` tail's `git log --grep`, as a Rust-fed column and a `stats` table: "doing with zero commits since", filed-by-session from `Claude-Session` trailers | `run cargo test stats::commits_join` | 4; zero-network holds (local refs) |
| **6 — cached pulse (conditional)** | `.cache/pulse.json` keyed on FORMAT.md's content hash, only if the Rust pass ever misses the prime gate at 1K | bench | rung 2's perf result |

Not on the ladder: a chart, an aging boost, a solver, per-actor closes (needs a format change),
token telemetry, anything that runs when no verb is running.

---

## 12. Rulings requested

1. **Views as contract.** Thirteen named SQL views published in FORMAT.md as the derived
   projection, registered by the binary, additive under the minting-rule idiom. (Also settles
   MW-R17's "column in the projection" wording for the prioritization proposal.)
2. **The pulse in `prime`.** Five weather lines and the `cites` tail, inside existing sections,
   under the existing budget — proposed as behaviour, no §6 change; confirm the placement at the
   top of weather (before doing/blocked lines).
3. **`stats` as a verb** (§6 addition) — or fold into `lint --stats` as the prioritization
   proposal spelled it. Recommended: `stats`, and rule both proposals' spellings together.
4. **Reads never prune** — narrow mw-chcqk6g to `next`/`ready`.
5. **The two §3 scope rulings** the prioritization proposal requests (log-derived lifecycle
   statistics are not the rejected estimates/time tracking; ordering the ready set is not
   sprint semantics), extended by one clause: *period counts with denominators are not the
   rejected burndown* — no chart, no projection, no target.
6. **`set --handoff` mints a log line.** A minting rule, additive; makes handoff age derivable
   and F2's "stamp it" answerable without a format change.
7. **Lineage growth in the digest.** The pulse's flow line carries `filed 206 (49 from other
   tasks)`; should a growing-spawner line (`sa-72rrcck: 25 descendants this week, still open`)
   also enter `prime`, or stay in `stats`/`show`? Proposed: `stats`/`show` until a session is
   observed missing it.
8. **Filing.** Rungs 0–3 as tasks now, or after the ruling?

---

## 13. Falsifiers

| falsifier | cost to test | consequence |
|---|---|---|
| The pulse block is read as noise — sessions keep leading with stale handoffs at the same rate a month after rung 2 (measure: `handoff_stale_n` among tasks that were `next` in a session, from `attention`) | one query | the block is demoted to `stats`; `prime` keeps only the asks line and the `cites` tail |
| The Rust pulse misses the 100 ms prime gate at 1K tasks | `check-perf.sh` | rung 6 (cached pulse); the SQL contract is unchanged |
| A view's SQL cannot be reproduced by a second reader from FORMAT.md alone (the UI's M0/M1 finds a divergence that is not a meshwork bug) | the corpus | the derivation note is wrong; fix FORMAT.md, re-bless, never the binary alone |
| `implicit-edge` findings are overwhelmingly false (words that equal ids) | count after rung 3 | tighten the tokenizer to require the `repo#` form outside `handoff`/`body`, or drop the finding |
| `spawned_w` never discriminates — every active repo shows growth everywhere | rung 1 query over a month | the lineage line stays in `show` only; `gravity` remains a display quantity |
| `seq` collisions are deliberate ties the owner uses | ask | drop `seq-collision`; keep the `distinct_seq` column |
| The window (7 d) misses the refresh cadence | observe the BURNDOWN dates | `[stats] window_days` is the knob; nothing else moves |

---

## Appendix A — the view layer, verbatim

Executed against all nine stores on 2026-09-06T16:11Z, exit 0 everywhere (Appendix B).
`clock` is shown as a view over `now()` for readers without an override; the binary registers
it as a table. `p_win` is the window; the binary substitutes `[stats] window_days`.

```sql
CREATE VIEW clock AS
SELECT now() AS now, CAST(now() AS DATE) AS today, 'system' AS source;

CREATE VIEW p_win AS
SELECT c.now - INTERVAL '7 days' AS since, c.now FROM clock c;

CREATE VIEW events AS
SELECT t.gid, t.repo, 'created' AS kind, t.created AS stamp, CASE WHEN regexp_like(t.created, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$') THEN to_timestamp(t.created, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END AS at,
       CAST(NULL AS VARCHAR) AS actor, CAST(NULL AS VARCHAR) AS from_status, CAST(NULL AS VARCHAR) AS to_status,
       CAST(NULL AS VARCHAR) AS note, 0 AS ord
FROM tasks t WHERE t.status <> 'invalid' AND t.created IS NOT NULL
UNION ALL
SELECT l.gid, t.repo,
       CASE WHEN l.to_status IS NOT NULL THEN 'transition'
            WHEN l.note LIKE 'close attempt%' THEN 'close-attempt'
            ELSE 'note' END AS kind,
       l.date AS stamp, CASE WHEN regexp_like(l.date, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$') THEN to_timestamp(l.date, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END AS at,
       CASE WHEN l.note LIKE 'claimed by %' THEN substr(l.note, 12) END AS actor,
       l.from_status, l.to_status, l.note, l.ord
FROM log l JOIN tasks t ON t.gid = l.gid
UNION ALL
SELECT c.gid, t.repo, 'comment' AS kind, c.date AS stamp, CASE WHEN regexp_like(c.date, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$') THEN to_timestamp(c.date, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END AS at,
       c.author AS actor, CAST(NULL AS VARCHAR) AS from_status, CAST(NULL AS VARCHAR) AS to_status, c.text AS note, c.ord
FROM comments c JOIN tasks t ON t.gid = c.gid;

CREATE VIEW spans AS
SELECT gid, repo, state, entered, left_at, next_state,
       (to_unixtime(coalesce(left_at, c.now)) - to_unixtime(entered)) / 3600.0 AS hours
FROM (SELECT gid, repo, to_status AS state, at AS entered,
             lead(at) OVER (PARTITION BY gid ORDER BY ord) AS left_at,
             lead(to_status) OVER (PARTITION BY gid ORDER BY ord) AS next_state
      FROM events WHERE kind = 'transition' AND at IS NOT NULL) s
CROSS JOIN clock c;

CREATE VIEW f_tr AS
SELECT gid,
       min(at) FILTER (WHERE to_status = 'doing') AS first_doing,
       max(at) FILTER (WHERE to_status = 'doing') AS last_doing,
       max(at) FILTER (WHERE to_status = 'done') AS done_at,
       max(at) FILTER (WHERE to_status IN ('done','dropped')) AS terminal_at,
       max(at) FILTER (WHERE kind = 'transition') AS last_transition,
       count(*) FILTER (WHERE to_status = 'blocked') AS blocks,
       count(*) FILTER (WHERE to_status = 'open' AND from_status IS NOT NULL) AS reopens,
       count(*) FILTER (WHERE kind = 'close-attempt') AS close_attempts,
       count(*) FILTER (WHERE kind = 'transition' AND actor IS NOT NULL) AS claims,
       count(*) FILTER (WHERE kind = 'comment') AS comments,
       count(DISTINCT actor) AS actors,
       max(at) AS last_activity,
       max(stamp) AS last_stamp,
       count(*) FILTER (WHERE kind <> 'created') AS touches
FROM events GROUP BY gid;

CREATE VIEW f_bl AS
SELECT gid, sum(hours) AS blocked_h FROM spans WHERE state = 'blocked' GROUP BY gid;

CREATE VIEW f_dg AS
SELECT gid, sum(hours) AS doing_h FROM spans WHERE state = 'doing' GROUP BY gid;

CREATE VIEW facts AS
SELECT t.gid, t.repo, t.id, t.title, t.status, t.category, t.seq,
       t.status IN ('open','doing','blocked') AS live,
       CASE WHEN regexp_like(t.created, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$') THEN to_timestamp(t.created, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END AS created_at,
       tr.first_doing, tr.last_doing, tr.done_at, tr.terminal_at, tr.last_transition,
       (to_unixtime(coalesce(tr.terminal_at, c.now)) - to_unixtime(CASE WHEN regexp_like(t.created, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$') THEN to_timestamp(t.created, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END)) / 3600.0 AS age_h,
       (to_unixtime(tr.first_doing) - to_unixtime(CASE WHEN regexp_like(t.created, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$') THEN to_timestamp(t.created, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END)) / 3600.0 AS queue_h,
       CASE WHEN tr.first_doing <= tr.done_at
            THEN (to_unixtime(tr.done_at) - to_unixtime(tr.first_doing)) / 3600.0 END AS service_h,
       (to_unixtime(tr.done_at) - to_unixtime(CASE WHEN regexp_like(t.created, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$') THEN to_timestamp(t.created, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END)) / 3600.0 AS cycle_h,
       coalesce(bl.blocked_h, 0) AS blocked_h,
       coalesce(dg.doing_h, 0) AS doing_h,
       tr.last_activity, tr.last_stamp,
       (to_unixtime(c.now) - to_unixtime(coalesce(tr.last_activity, CASE WHEN regexp_like(t.created, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$') THEN to_timestamp(t.created, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END))) / 3600.0 AS idle_h,
       coalesce(tr.touches, 0) AS touches, coalesce(tr.comments, 0) AS comments,
       coalesce(tr.actors, 0) AS actors, coalesce(tr.claims, 0) AS claims,
       coalesce(tr.close_attempts, 0) AS close_attempts, coalesce(tr.reopens, 0) AS reopens,
       coalesce(tr.blocks, 0) AS blocks,
       t.verify IS NOT NULL AND trim(t.verify) <> '' AS has_verify,
       t.category IS NOT NULL AS has_category,
       t.seq IS NOT NULL AS has_seq,
       t.waived IS NOT NULL AS waived,
       t.handoff IS NOT NULL AS has_handoff,
       t.addressed_to IS NOT NULL AS is_ask,
       t.claimed_by
FROM tasks t
JOIN repos rp ON rp.repo = t.repo
LEFT JOIN f_tr tr ON tr.gid = t.gid
LEFT JOIN f_bl bl ON bl.gid = t.gid
LEFT JOIN f_dg dg ON dg.gid = t.gid
CROSS JOIN clock c
WHERE t.status <> 'invalid';

CREATE VIEW g_live AS
SELECT gid, repo, coalesce(seq, 999999) AS place FROM tasks WHERE status IN ('open','doing','blocked');

CREATE VIEW g_ln AS
SELECT e.src_gid, e.dst_gid FROM edges e
JOIN g_live a ON a.gid = e.src_gid JOIN g_live b ON b.gid = e.dst_gid
WHERE e.kind = 'needs';

CREATE VIEW g_up AS
SELECT dst_gid AS pre, src_gid AS dep, 1 AS d FROM g_ln
UNION ALL
SELECT g_up.pre, g_ln.src_gid, g_up.d + 1 FROM g_up JOIN g_ln ON g_ln.dst_gid = g_up.dep WHERE g_up.d < 64;

CREATE VIEW g_unl AS
SELECT pre AS gid, count(DISTINCT dep) AS unlock, max(d) AS depth FROM g_up GROUP BY pre;

CREATE VIEW g_inh AS
SELECT u.pre AS gid, min(l.place) AS inherit
FROM g_up u JOIN g_live p ON p.gid = u.pre JOIN g_live l ON l.gid = u.dep AND l.repo = p.repo
GROUP BY u.pre;

CREATE VIEW g_nodes AS
SELECT src_gid AS n FROM g_ln UNION SELECT dst_gid AS n FROM g_ln;

CREATE VIEW g_und AS
SELECT src_gid AS a, dst_gid AS b FROM g_ln
UNION ALL SELECT dst_gid AS a, src_gid AS b FROM g_ln
UNION ALL SELECT n AS a, n AS b FROM g_nodes;

CREATE VIEW g_lab AS
SELECT n AS node, n AS lane, 0 AS i, true AS changed FROM g_nodes
UNION ALL
SELECT node, lane, i, changed FROM (
  SELECT node, lane, i, changed, sum(CASE WHEN changed THEN 1 ELSE 0 END) OVER () AS n_changed
  FROM (SELECT g_und.a AS node, min(g_lab.lane) AS lane, g_lab.i + 1 AS i,
               min(g_lab.lane) < min(CASE WHEN g_und.a = g_und.b THEN g_lab.lane END) AS changed
        FROM g_lab JOIN g_und ON g_und.b = g_lab.node WHERE g_lab.i < 64
        GROUP BY g_und.a, g_lab.i) step) marked
WHERE n_changed > 0;

CREATE VIEW g_lanes AS
SELECT node AS gid, min(lane) AS lane FROM g_lab GROUP BY node;

CREATE VIEW g_lsize AS
SELECT lane, count(*) AS lane_size FROM g_lanes GROUP BY lane;

CREATE VIEW g_needs_open AS
SELECT e.src_gid AS gid, count(*) AS needs_open,
       count(*) FILTER (WHERE d.gid IS NULL) AS needs_unresolved,
       count(*) FILTER (WHERE d.repo IS NOT NULL AND d.repo <> t.repo) AS needs_open_foreign
FROM edges e JOIN tasks t ON t.gid = e.src_gid LEFT JOIN tasks d ON d.gid = e.dst_gid
WHERE e.kind = 'needs' AND (d.status IS NULL OR d.status NOT IN ('done','dropped'))
GROUP BY e.src_gid;

CREATE VIEW g_needed_by AS
SELECT e.dst_gid AS gid, count(*) AS needed_by_open,
       count(*) FILTER (WHERE s.repo <> t.repo) AS needed_by_foreign
FROM edges e JOIN tasks s ON s.gid = e.src_gid JOIN tasks t ON t.gid = e.dst_gid
WHERE e.kind = 'needs' AND s.status IN ('open','doing','blocked')
GROUP BY e.dst_gid;

CREATE VIEW g_children AS
SELECT e.dst_gid AS gid, count(*) AS live_children FROM edges e JOIN tasks ch ON ch.gid = e.src_gid
WHERE e.kind = 'parent' AND ch.status IN ('open','doing','blocked') GROUP BY e.dst_gid;

CREATE VIEW g_spawned AS
SELECT dst_gid AS gid, count(*) AS discovered FROM edges WHERE kind = 'discovered-from' GROUP BY dst_gid;

CREATE VIEW graph AS
SELECT t.gid, t.repo, t.id, t.status, t.seq,
       coalesce(u.unlock, 0) AS unlock, coalesce(u.depth, 0) AS depth,
       coalesce(t.seq, 999999) AS place,
       coalesce(i.inherit, coalesce(t.seq, 999999)) AS inherit,
       coalesce(i.inherit < coalesce(t.seq, 999999), false) AS needs_behind,
       ln.lane, coalesce(ls.lane_size, 1) AS lane_size,
       coalesce(no.needs_open, 0) AS needs_open,
       coalesce(no.needs_unresolved, 0) AS needs_unresolved,
       coalesce(no.needs_open_foreign, 0) AS needs_open_foreign,
       coalesce(nb.needed_by_open, 0) AS needed_by_open,
       coalesce(nb.needed_by_foreign, 0) AS needed_by_foreign,
       coalesce(ch.live_children, 0) AS live_children,
       coalesce(sp.discovered, 0) AS discovered,
       t.status = 'open' AND coalesce(no.needs_open, 0) = 0 AND coalesce(ch.live_children, 0) = 0 AS ready
FROM tasks t
LEFT JOIN g_unl u ON u.gid = t.gid
LEFT JOIN g_inh i ON i.gid = t.gid
LEFT JOIN g_lanes ln ON ln.gid = t.gid
LEFT JOIN g_lsize ls ON ls.lane = ln.lane
LEFT JOIN g_needs_open no ON no.gid = t.gid
LEFT JOIN g_needed_by nb ON nb.gid = t.gid
LEFT JOIN g_children ch ON ch.gid = t.gid
LEFT JOIN g_spawned sp ON sp.gid = t.gid
JOIN repos rp ON rp.repo = t.repo
WHERE t.status <> 'invalid';

CREATE VIEW a_ans AS
SELECT e.dst_gid AS ask_gid, min(x.gid) AS answer_gid, min(x.status) AS answer_status,
       min(fx.created_at) AS answered_at, max(fx.done_at) AS answer_done_at
FROM edges e JOIN tasks x ON x.gid = e.src_gid JOIN facts fx ON fx.gid = x.gid
WHERE e.kind = 'answers' AND x.status <> 'dropped'
GROUP BY e.dst_gid;

CREATE VIEW asks AS
SELECT a.gid, a.repo AS from_repo, split_part(a.addressed_to, '#', 1) AS to_repo, a.addressed_to AS to_ref,
       a.title, a.status, f.created_at, f.age_h, f.idle_h, f.seq,
       ans.answer_gid, ans.answer_status, ans.answered_at, ans.answer_done_at,
       (to_unixtime(ans.answered_at) - to_unixtime(f.created_at)) / 3600.0 AS response_h,
       ans.answer_gid IS NULL AND a.status NOT IN ('done','dropped') AS unanswered
FROM tasks a JOIN facts f ON f.gid = a.gid
LEFT JOIN a_ans ans ON ans.ask_gid = a.gid
WHERE a.addressed_to IS NOT NULL;

CREATE VIEW m_tok AS
SELECT gid, repo, 'handoff' AS field, CAST(NULL AS VARCHAR) AS stamp,
       unnest(string_to_array(regexp_replace(handoff, '[^a-z0-9#-]+', ' ', 'g'), ' ')) AS tok
FROM tasks WHERE handoff IS NOT NULL
UNION ALL
SELECT gid, repo, 'body' AS field, CAST(NULL AS VARCHAR) AS stamp,
       unnest(string_to_array(regexp_replace(body, '[^a-z0-9#-]+', ' ', 'g'), ' ')) AS tok
FROM tasks WHERE body IS NOT NULL AND body <> ''
UNION ALL
SELECT c.gid, t.repo, 'comment' AS field, c.date AS stamp,
       unnest(string_to_array(regexp_replace(c.text, '[^a-z0-9#-]+', ' ', 'g'), ' ')) AS tok
FROM comments c JOIN tasks t ON t.gid = c.gid
UNION ALL
SELECT l.gid, t.repo, 'log' AS field, l.date AS stamp,
       unnest(string_to_array(regexp_replace(l.note, '[^a-z0-9#-]+', ' ', 'g'), ' ')) AS tok
FROM log l JOIN tasks t ON t.gid = l.gid WHERE l.note IS NOT NULL;

CREATE VIEW m_pairs AS
SELECT k.gid AS src_gid, k.repo AS src_repo, k.field, r.gid AS ref_gid,
       min(CASE WHEN regexp_like(k.stamp, '^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}Z)?$') THEN to_timestamp(k.stamp, '%Y-%m-%dT%H:%MZ', '%Y-%m-%d') END) AS first_at, count(*) AS times
FROM (SELECT gid, repo, field, stamp,
             CASE WHEN strpos(tok, '#') > 0 THEN tok ELSE repo || '#' || tok END AS ref
      FROM m_tok WHERE tok <> '') k
JOIN tasks r ON r.gid = k.ref
WHERE r.gid <> k.gid
GROUP BY k.gid, k.repo, k.field, r.gid;

CREATE VIEW m_edge AS
SELECT DISTINCT a, b FROM (SELECT src_gid AS a, dst_gid AS b FROM edges UNION ALL SELECT dst_gid AS a, src_gid AS b FROM edges);

CREATE VIEW mentions AS
SELECT p.src_gid, p.src_repo, p.field, p.ref_gid, p.times, p.first_at,
       r.status AS ref_status, fr.terminal_at AS ref_terminal_at, fs.last_activity AS src_last_activity,
       coalesce(fr.terminal_at > fs.last_activity, false) AS moved_since_activity,
       eb.a IS NOT NULL AS edge_backed
FROM m_pairs p JOIN tasks r ON r.gid = p.ref_gid
JOIN facts fr ON fr.gid = r.gid JOIN facts fs ON fs.gid = p.src_gid
LEFT JOIN m_edge eb ON eb.a = p.src_gid AND eb.b = p.ref_gid;

CREATE VIEW li_df AS
SELECT src_gid AS child, dst_gid AS origin FROM edges WHERE kind = 'discovered-from';

CREATE VIEW li_sp AS
SELECT origin, child, 1 AS d FROM li_df
UNION ALL
SELECT li_sp.origin, li_df.child, li_sp.d + 1 FROM li_sp JOIN li_df ON li_df.origin = li_sp.child WHERE li_sp.d < 64;

CREATE VIEW li_spawn AS
SELECT s.origin AS gid,
       count(*) FILTER (WHERE s.d = 1) AS spawned_direct,
       count(*) AS spawned_total,
       count(*) FILTER (WHERE f.live) AS spawned_live,
       max(s.d) AS spawn_depth,
       max(f.created_at) AS last_spawn_at,
       count(*) FILTER (WHERE f.created_at >= w.since) AS spawned_w,
       count(*) FILTER (WHERE f.status = 'done') AS spawned_done
FROM li_sp s JOIN facts f ON f.gid = s.child CROSS JOIN p_win w
GROUP BY s.origin;

CREATE VIEW li_pa AS
SELECT src_gid AS child, dst_gid AS parent FROM edges WHERE kind = 'parent';

CREATE VIEW li_um AS
SELECT parent AS root, child, 1 AS d FROM li_pa
UNION ALL
SELECT li_um.root, li_pa.child, li_um.d + 1 FROM li_um JOIN li_pa ON li_pa.parent = li_um.child WHERE li_um.d < 64;

CREATE VIEW li_umbrella AS
SELECT u.root AS gid,
       count(*) FILTER (WHERE u.d = 1) AS children_direct,
       count(*) AS descendants_total,
       count(*) FILTER (WHERE f.live) AS descendants_live,
       max(u.d) AS umbrella_depth,
       max(f.created_at) AS last_child_at,
       count(*) FILTER (WHERE f.created_at >= w.since) AS children_w,
       count(*) FILTER (WHERE f.status = 'done') AS descendants_done
FROM li_um u JOIN facts f ON f.gid = u.child CROSS JOIN p_win w
GROUP BY u.root;

CREATE VIEW li_in AS
SELECT dst_gid AS gid,
       count(*) FILTER (WHERE kind = 'needs') AS needs_in,
       count(*) FILTER (WHERE kind = 'relates') AS relates_in,
       count(*) FILTER (WHERE kind = 'answers') AS answers_in,
       count(DISTINCT src_gid) AS referrers_edges
FROM edges GROUP BY dst_gid;

CREATE VIEW li_men AS
SELECT ref_gid AS gid, count(DISTINCT src_gid) AS mentioned_by,
       count(DISTINCT src_gid) FILTER (WHERE NOT edge_backed) AS mentioned_by_unlinked,
       count(DISTINCT src_gid) FILTER (WHERE field = 'comment' OR field = 'log') AS mentioned_in_talk
FROM mentions GROUP BY ref_gid;

CREATE VIEW lineage AS
SELECT t.gid, t.repo, t.id, t.status,
       coalesce(sp.spawned_direct, 0) AS spawned_direct, coalesce(sp.spawned_total, 0) AS spawned_total,
       coalesce(sp.spawned_live, 0) AS spawned_live, coalesce(sp.spawned_done, 0) AS spawned_done,
       coalesce(sp.spawn_depth, 0) AS spawn_depth, sp.last_spawn_at, coalesce(sp.spawned_w, 0) AS spawned_w,
       coalesce(um.children_direct, 0) AS children_direct, coalesce(um.descendants_total, 0) AS descendants_total,
       coalesce(um.descendants_live, 0) AS descendants_live, coalesce(um.descendants_done, 0) AS descendants_done,
       coalesce(um.umbrella_depth, 0) AS umbrella_depth, um.last_child_at, coalesce(um.children_w, 0) AS children_w,
       coalesce(li.needs_in, 0) AS needs_in, coalesce(li.relates_in, 0) AS relates_in, coalesce(li.answers_in, 0) AS answers_in,
       coalesce(li.referrers_edges, 0) AS referrers_edges,
       coalesce(mn.mentioned_by, 0) AS mentioned_by, coalesce(mn.mentioned_by_unlinked, 0) AS mentioned_by_unlinked,
       coalesce(mn.mentioned_in_talk, 0) AS mentioned_in_talk,
       coalesce(sp.spawned_total, 0) + coalesce(um.descendants_total, 0) + coalesce(li.referrers_edges, 0) + coalesce(mn.mentioned_by, 0) AS gravity
FROM tasks t
LEFT JOIN li_spawn sp ON sp.gid = t.gid
LEFT JOIN li_umbrella um ON um.gid = t.gid
LEFT JOIN li_in li ON li.gid = t.gid
LEFT JOIN li_men mn ON mn.gid = t.gid
JOIN repos rp ON rp.repo = t.repo
WHERE t.status <> 'invalid';

CREATE VIEW fl_days AS
SELECT r.repo, d.day FROM repos r
CROSS JOIN (SELECT unnest(generate_series((SELECT CAST(min(created_at) AS DATE) FROM facts),
                                          (SELECT today FROM clock), INTERVAL '1 day')) AS day) d;

CREATE VIEW fl_ev AS
SELECT repo, CAST(created_at AS DATE) AS day, 1 AS filed, 0 AS done, 0 AS dropped, 0 AS doing_in, 0 AS doing_out
FROM facts WHERE created_at IS NOT NULL
UNION ALL
SELECT repo, CAST(terminal_at AS DATE) AS day, 0 AS filed, CASE WHEN status = 'done' THEN 1 ELSE 0 END AS done,
       CASE WHEN status = 'dropped' THEN 1 ELSE 0 END AS dropped, 0 AS doing_in, 0 AS doing_out
FROM facts WHERE terminal_at IS NOT NULL
UNION ALL
SELECT repo, CAST(entered AS DATE) AS day, 0 AS filed, 0 AS done, 0 AS dropped, 1 AS doing_in, 0 AS doing_out
FROM spans WHERE state = 'doing'
UNION ALL
SELECT repo, CAST(left_at AS DATE) AS day, 0 AS filed, 0 AS done, 0 AS dropped, 0 AS doing_in, 1 AS doing_out
FROM spans WHERE state = 'doing' AND left_at IS NOT NULL;

CREATE VIEW fl_day AS
SELECT repo, day, sum(filed) AS filed, sum(done) AS done, sum(dropped) AS dropped,
       sum(doing_in) AS doing_in, sum(doing_out) AS doing_out
FROM fl_ev GROUP BY repo, day;

CREATE VIEW flow AS
SELECT repo, day, filed, done, dropped,
       sum(filed) OVER w AS filed_cum, sum(done) OVER w AS done_cum, sum(dropped) OVER w AS dropped_cum,
       sum(filed - done - dropped) OVER w AS backlog,
       filed - done - dropped AS backlog_delta,
       sum(doing_in - doing_out) OVER w AS doing_eod
FROM (SELECT d.repo, d.day, coalesce(e.filed, 0) AS filed, coalesce(e.done, 0) AS done, coalesce(e.dropped, 0) AS dropped,
             coalesce(e.doing_in, 0) AS doing_in, coalesce(e.doing_out, 0) AS doing_out
      FROM fl_days d LEFT JOIN fl_day e ON e.repo = d.repo AND e.day = d.day) x
WINDOW w AS (PARTITION BY repo ORDER BY day);

CREATE VIEW hz_life AS
SELECT gid, repo, status, age_h / 24.0 AS age_d,
       (to_unixtime(c.now) - to_unixtime(created_at)) / 86400.0 AS observed_d
FROM facts CROSS JOIN clock c WHERE created_at IS NOT NULL;

CREATE VIEW hazard AS
SELECT l.repo, b.age_d, count(*) AS at_risk,
       count(*) FILTER (WHERE l.status = 'done' AND l.age_d < b.age_d + 1) AS done_in,
       count(*) FILTER (WHERE l.status = 'dropped' AND l.age_d < b.age_d + 1) AS dropped_in
FROM (SELECT unnest(generate_series(0, 60)) AS age_d) b
JOIN hz_life l ON l.age_d >= b.age_d AND l.observed_d >= b.age_d + 1
GROUP BY l.repo, b.age_d;

CREATE VIEW attention AS
SELECT f.gid, f.repo, f.id, f.status, f.idle_h, f.actors,
       count(DISTINCT e.actor) AS touched_since_move,
       min(e.at) AS first_touch_since_move
FROM facts f LEFT JOIN events e ON e.gid = f.gid AND e.actor IS NOT NULL
     AND (f.last_transition IS NULL OR e.at > f.last_transition)
WHERE f.live
GROUP BY f.gid, f.repo, f.id, f.status, f.idle_h, f.actors;

CREATE VIEW sessions AS
SELECT actor, repo, min(at) AS first_seen, max(at) AS last_seen,
       count(DISTINCT gid) AS tasks_touched,
       count(*) FILTER (WHERE kind = 'comment') AS comments,
       count(*) FILTER (WHERE kind = 'transition') AS claims
FROM events WHERE actor IS NOT NULL GROUP BY actor, repo;

CREATE VIEW p_ev AS
SELECT e.repo,
       count(*) FILTER (WHERE kind = 'created') AS filed,
       count(DISTINCT gid) FILTER (WHERE to_status = 'done') AS done,
       count(DISTINCT gid) FILTER (WHERE to_status = 'dropped') AS dropped,
       count(DISTINCT gid) FILTER (WHERE to_status = 'doing') AS started,
       count(*) FILTER (WHERE to_status = 'blocked') AS blocks,
       count(*) FILTER (WHERE to_status = 'open' AND from_status IS NOT NULL) AS reopens,
       count(*) FILTER (WHERE kind = 'close-attempt') AS close_attempts,
       count(*) FILTER (WHERE kind = 'comment') AS comments,
       count(DISTINCT actor) AS actors
FROM events e CROSS JOIN p_win w WHERE e.at >= w.since GROUP BY e.repo;

CREATE VIEW p_state AS
SELECT repo,
       count(*) FILTER (WHERE status = 'open') AS open_n,
       count(*) FILTER (WHERE status = 'doing') AS doing_n,
       count(*) FILTER (WHERE status = 'blocked') AS blocked_n,
       count(*) FILTER (WHERE status = 'done') AS done_n,
       count(*) FILTER (WHERE status = 'dropped') AS dropped_n,
       count(*) FILTER (WHERE status = 'doing' AND idle_h >= 72) AS doing_stale,
       round(median(age_h) FILTER (WHERE status = 'open') / 24.0, 1) AS open_age_med_d,
       count(*) FILTER (WHERE live AND age_h >= 14 * 24) AS past_triage,
       count(*) FILTER (WHERE live AND NOT has_verify) AS no_verify,
       count(*) FILTER (WHERE live AND NOT has_seq) AS no_seq,
       count(*) FILTER (WHERE live AND NOT has_category) AS no_category,
       count(*) FILTER (WHERE live AND has_seq AND idle_h >= 14 * 24) AS parked_in_place,
       count(*) FILTER (WHERE status = 'done' AND waived) AS waived_n
FROM facts GROUP BY repo;

CREATE VIEW p_graph AS
SELECT repo,
       count(*) FILTER (WHERE ready) AS ready_n,
       count(*) FILTER (WHERE unlock > 0) AS unlockers,
       count(DISTINCT lane) FILTER (WHERE lane_size > 1) AS lanes_multi,
       count(*) FILTER (WHERE needs_behind) AS needs_behind_n,
       count(*) FILTER (WHERE status IN ('open','doing','blocked') AND needs_open_foreign > 0) AS blocked_foreign,
       count(*) FILTER (WHERE status IN ('open','doing','blocked') AND needs_unresolved > 0) AS blocked_unresolved,
       count(*) FILTER (WHERE status IN ('open','doing','blocked') AND needed_by_foreign > 0) AS owed_foreign
FROM graph GROUP BY repo;

CREATE VIEW p_asks_in AS
SELECT to_repo AS repo, count(*) FILTER (WHERE unanswered) AS asks_in_open,
       round(max(age_h) FILTER (WHERE unanswered) / 24.0, 1) AS asks_in_oldest_d
FROM asks GROUP BY to_repo;

CREATE VIEW p_asks_out AS
SELECT from_repo AS repo, count(*) FILTER (WHERE unanswered) AS asks_out_open,
       round(max(age_h) FILTER (WHERE unanswered) / 24.0, 1) AS asks_out_oldest_d
FROM asks GROUP BY from_repo;

CREATE VIEW p_thrash AS
SELECT repo, count(*) AS thrash_n FROM attention WHERE touched_since_move >= 2 GROUP BY repo;

CREATE VIEW p_refs AS
SELECT src_repo AS repo, count(DISTINCT src_gid) AS handoff_stale_n FROM mentions WHERE field = 'handoff' AND ref_status IN ('done','dropped') GROUP BY src_repo;

CREATE VIEW p_lin AS
SELECT l.repo,
       count(*) FILTER (WHERE l.spawned_w > 0) AS spawners_w,
       max(l.spawned_w) AS spawned_w_max,
       count(*) FILTER (WHERE l.children_w > 0) AS umbrellas_w,
       sum(l.mentioned_by_unlinked) AS unlinked_mentions
FROM lineage l GROUP BY l.repo;

CREATE VIEW p_from AS
SELECT f.repo, count(DISTINCT f.gid) AS filed_from_w
FROM facts f JOIN edges e ON e.src_gid = f.gid AND e.kind = 'discovered-from' CROSS JOIN p_win w
WHERE f.created_at >= w.since GROUP BY f.repo;

CREATE VIEW p_implicit AS
SELECT m.src_repo AS repo, count(*) AS implicit_live_n
FROM mentions m JOIN facts a ON a.gid = m.src_gid JOIN facts b ON b.gid = m.ref_gid
WHERE NOT m.edge_backed AND a.live AND b.live GROUP BY m.src_repo;

CREATE VIEW pulse AS
SELECT r.repo,
       s.open_n, s.doing_n, s.blocked_n, s.done_n, s.dropped_n,
       coalesce(e.filed, 0) AS filed_w, coalesce(e.done, 0) AS done_w, coalesce(e.dropped, 0) AS dropped_w,
       coalesce(e.started, 0) AS started_w,
       coalesce(e.filed, 0) - coalesce(e.done, 0) - coalesce(e.dropped, 0) AS backlog_delta_w,
       s.doing_stale, s.open_age_med_d, s.past_triage, s.parked_in_place,
       coalesce(g.ready_n, 0) AS ready_n, coalesce(g.unlockers, 0) AS unlockers, coalesce(g.lanes_multi, 0) AS lanes_multi,
       coalesce(g.needs_behind_n, 0) AS needs_behind_n, coalesce(g.blocked_foreign, 0) AS blocked_foreign,
       coalesce(g.blocked_unresolved, 0) AS blocked_unresolved, coalesce(g.owed_foreign, 0) AS owed_foreign,
       coalesce(ai.asks_in_open, 0) AS asks_in_open, ai.asks_in_oldest_d,
       coalesce(ao.asks_out_open, 0) AS asks_out_open, ao.asks_out_oldest_d,
       coalesce(e.close_attempts, 0) AS close_attempts_w, coalesce(e.reopens, 0) AS reopens_w,
       coalesce(e.blocks, 0) AS blocks_w, s.waived_n, s.no_verify, s.no_seq, s.no_category,
       coalesce(e.comments, 0) AS comments_w, coalesce(e.actors, 0) AS actors_w,
       coalesce(th.thrash_n, 0) AS thrash_n, coalesce(rf.handoff_stale_n, 0) AS handoff_stale_n,
       coalesce(fr.filed_from_w, 0) AS filed_from_w, coalesce(li.spawners_w, 0) AS spawners_w,
       coalesce(li.spawned_w_max, 0) AS spawned_w_max, coalesce(li.umbrellas_w, 0) AS umbrellas_w,
       coalesce(li.unlinked_mentions, 0) AS unlinked_mentions, coalesce(im.implicit_live_n, 0) AS implicit_live_n
FROM repos r
LEFT JOIN p_state s ON s.repo = r.repo
LEFT JOIN p_ev e ON e.repo = r.repo
LEFT JOIN p_graph g ON g.repo = r.repo
LEFT JOIN p_asks_in ai ON ai.repo = r.repo
LEFT JOIN p_asks_out ao ON ao.repo = r.repo
LEFT JOIN p_thrash th ON th.repo = r.repo
LEFT JOIN p_refs rf ON rf.repo = r.repo
LEFT JOIN p_lin li ON li.repo = r.repo
LEFT JOIN p_from fr ON fr.repo = r.repo
LEFT JOIN p_implicit im ON im.repo = r.repo;
```

## Appendix B — validation record

**Method.** Each view body above was held as a CTE and every probe assembled as
`WITH RECURSIVE <all views in dependency order> SELECT …`, then run as one
`meshwork --json q` per (store, probe) from the store's own checkout with the v0.4.0 release
binary — fifteen probes covering every public view — against meshwork, sazed, leras, marasi,
tensoon, oreseur, wyndam, marasi-applied-r-and-d, and portfolio. No `portfolio` verb was run;
no file was written. Result at 2026-09-06T16:11Z: **135 of 135 probes exit 0**, zero
DataFusion errors. Wall time per probe (process start + load + plan + execute): sazed max
0.94 s (the full `pulse`), every other store max ≤ 0.43 s, 105 of 135 probes under 0.2 s.
The harness is `scripts/mine_views.py`; its `--sql` output is this appendix byte for byte, and
`--check` re-runs the validation (108 probes at twelve per store, exit 0 observed the same day).

**Per-repo pulse rows** (the source of §1's totals; `_w` = 7 days):

| repo | open/doing/blocked | done/dropped | filed_w | done_w | Δbacklog_w | ready | past 14 d | parked | asks out (oldest) | close att._w | thrash | handoff stale | filed_from_w |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| sazed | 259/2/2 | 321/22 | 206 | 155 | +47 | 239 | 173 | 155 | 10 (7.0 d) | 15 | 3 | 69 | 49 |
| leras | 54/2/7 | 69/1 | 25 | 24 | +1 | 48 | 53 | 39 | 4 (15.0 d) | 0 | 2 | 4 | 5 |
| marasi | 34/1/2 | 53/1 | 43 | 37 | +6 | 17 | 11 | 6 | 7 (5.8 d) | 3 | 2 | 0 | 23 |
| tensoon | 39/0/0 | 53/1 | 30 | 22 | +8 | 33 | 8 | 6 | 0 | 0 | 3 | 5 | 8 |
| marasi-applied-r-and-d | 15/0/0 | 63/0 | 54 | 49 | +5 | 9 | 0 | 0 | 8 (8.1 d) | 11 | 2 | 2 | 41 |
| portfolio | 15/0/0 | 5/0 | 14 | 2 | +12 | 10 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| meshwork | 11/0/0 | 141/1 | 1 | 0 | +1 | 6 | 9 | 9 | 0 | 0 | 0 | 5 | 0 |
| oreseur | 9/0/0 | 10/2 | 3 | 1 | +2 | 8 | 4 | 3 | 2 (5.5 d) | 0 | 1 | 0 | 2 |
| wyndam | 1/0/1 | 26/2 | 4 | 15 | −13 | 1 | 2 | 1 | 1 (17.9 d) | 0 | 0 | 0 | 3 |

**Two corrections made during validation, recorded because they are design facts.** The first
components query — an undirected recursive walk — ran past two minutes on sazed and was killed;
undirected closure has no terminating depth on a graph where every edge is a 2-cycle. The
label-propagation form with a fixed 64-iteration bound ran to completion in ~5 s in the debug
build because every iteration pays planning; the convergence stop (`sum(changed) OVER () > 0`)
brought the whole `graph` view to 79 ms in release. The first `mentions` resolution used
`ON (r.id = tok AND r.repo = k.repo) OR r.gid = tok`, a nested loop at 180K tokens × 606 tasks
(1.38 s); normalizing tokens to `repo#id` first and joining on equality brought it to 90 ms.
Both forms are in Appendix A as they finally ran.

**Caveats.** Stores were live during the run; a re-run drifts by units. Single-repo loads see
no inbound asks (§6.2) and count cross-repo dependents as zero; the union numbers in the UI
proposal (161 cross-repo edges, 11 unanswered inbound) are the portfolio-scope truth those
columns would show under `portfolio q`. `done` in `flow` and `hazard` is by *current* status, so
a reopened task is not a closure — the 09-01 baseline's `min(→done)` convention counts 21 more
in sazed. Date-only stamps round to midnight. The binary's `q` runs one statement, so views were
validated as CTE chains; `CREATE VIEW` itself was confirmed accepted by DataFusion 51 in the
same binary (`q "CREATE VIEW v AS SELECT …"` exits 0), and rung 0's conformance test is where
the registered form is pinned.
