# SPIKE — the prioritization rubric, measured

What every candidate dimension of a task rank discriminates on the nine registered stores, the
key shape those numbers support, a declared-weight rubric run against today's order with every
disagreement named, and the authored-effort question answered with numbers. Nothing here is
authored, no format changes, no ruling is made: `mw-ryd25rq` rules on it.

Measured 2026-09-21T15:47Z on 504 ready tasks of 615 live across portfolio, sazed, leras,
meshwork, marasi, tensoon, oreseur, wyndam and marasi-applied-r-and-d. Every table regenerates
from `scripts/mine_rubric.py` (read-only, per-repo `q --json`, never a `portfolio` verb):

```
python3 scripts/mine_cost.py --json cost.json          # per-task tokens (the cost dimension)
MESHWORK_COST_JSON=cost.json python3 scripts/mine_rubric.py            # every table below
MESHWORK_COST_JSON=cost.json python3 scripts/mine_rubric.py --falsify --weights age=-1   # a sensitivity run
python3 scripts/mine_rubric.py --hubs                                  # the hub docs
```

## 0. The answer in five lines

1. **Structure is rare and concentrated.** 7% of ready tasks unlock anything, but among today's
   top-10 pairs `unlock` decides 27%. `depth` is `unlock` again (2,771 vs 2,773 pairs) — struck.
2. **Doc cohesion is the broadest derived signal after `seq`** — 46% of ready tasks carry a
   `docs:` link, 23% share one with a closed task — and it has a hub problem this doc names.
3. **Age is not a boost.** It orders 92% of pairs because it is `created` inverted, which the key
   already carries; the daily close hazard falls from 44% on day 0 to 9% on day 1 and to 1–4% a day
   past day 3, so an age boost promotes exactly the tasks least likely to close. The sensitivity run shows what it
   does. C.4 holds: age triages, never ranks.
4. **The key is rank-space, term by term.** A term moves a task by *positions* from where `seq`
   put it; the points-space alternative failed its own falsifier (sazed: a two-step term moved
   seq-4 work 27 places, because the top is numbered 1 apart and the median gap is 10).
5. **Measured cost by category does not do an authored size's work**: it explains 11% of
   per-task token variance, covers 37% of the ready set, and its headline spread halved (24.2× →
   9.6×) in the 13 days since it was measured. The authored field goes to the ruling with these
   numbers (§6); this spike does not resolve it.

## 1. Method, and the null every dimension had to beat

- **The unit is the ready set.** A rank orders only what is ready (open, no unmet `needs`, no live
  children — `graph.ready`); blocked and umbrella tasks are not ranked, and a dimension that
  discriminates among them is discriminating nothing a session sees. 504 tasks.
- **Today's order** is DESIGN §5's frozen key: `coalesce(seq, 999999), created, id`.
- **The null, stated before measuring**: a dimension is struck when it sits at its default (null
  or zero) on nearly every ready task, or when it is monotone in a key term already present. It
  is *not* rescued by a weight — a weight cannot create differences that are not there.
- **Discriminating power** is the share of ready-task pairs a dimension strictly orders (both
  values present and different), on the whole ready set and on today's top 10 — the region a
  rank can actually change. "Active" is the share of tasks where the dimension is not at its null.
- **Nothing is fitted.** The weights in §4 are declared and the falsifier in §5 reports what
  they do; the last attempt derived `w` from the current placement and reproduced it (the plan's
  §3 R-C). Every number here can come out either way, and §2's struck rows show that it did.

## 2. The dimensions, measured

Pooled over every store's ready set. `reads` names the view or file the dimension is derived
from — every one is a derived quantity a third party can recompute from the tables.

| dimension | reads | covered | active (≠ null) | pairs ordered, whole ready set | pairs ordered, today's top 10 |
|---|---|---:|---:|---:|---:|
| `seq` | tasks.seq | 449/504 (89%) | 449/504 (89%) | 38714/44229 (88%) | 317/343 (92%) |
| `unlock` | graph.unlock | 504/504 (100%) | 34/504 (7%) | 2773/44229 (6%) | 91/343 (27%) |
| `depth` | graph.depth | 504/504 (100%) | 34/504 (7%) | 2771/44229 (6%) | 91/343 (27%) |
| `inherit_gain` | graph.inherit − place | 504/504 (100%) | 5/504 (1%) | 737/44229 (2%) | 9/343 (3%) |
| `age_d` | facts.age_h | 504/504 (100%) | 504/504 (100%) | 40704/44229 (92%) | 289/343 (84%) |
| `queue_h` | facts.queue_h | 1/504 (0%) | 1/504 (0%) | 0/44229 (0%) | 0/343 (0%) |
| `sib_done` | tasks.parent + siblings' status | 55/504 (11%) | 55/504 (11%) | 221/44229 (0%) | 10/343 (3%) |
| `spawn_done` | edges discovered-from + siblings' status | 53/504 (11%) | 53/504 (11%) | 316/44229 (1%) | 17/343 (5%) |
| `doc_peers_done` | docs: links (files) + peers' status | 232/504 (46%) | 115/504 (23%) | 4794/44229 (11%) | 105/343 (31%) |
| `doc_done_share` | docs: links (files) + peers' status | 154/504 (31%) | 154/504 (31%) | 3063/44229 (7%) | 73/343 (21%) |
| `docpath_peers_done` | docs: paths (files) + peers' status | 232/504 (46%) | 141/504 (28%) | 5200/44229 (12%) | 112/343 (33%) |
| `cost_k` | cost-baseline: category median tokens | 186/504 (37%) | 186/504 (37%) | 1998/44229 (5%) | 36/343 (10%) |

Per store, the active share of the ready set:

| store | ready | `seq` | `unlock` | `depth` | `inherit_gain` | `age_d` | `queue_h` | `sib_done` | `spawn_done` | `doc_peers_done` | `doc_done_share` | `docpath_peers_done` | `cost_k` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| portfolio | 33 | 100% | 9% | 9% | 0% | 100% | 0% | 0% | 0% | 0% | 27% | 6% | 15% |
| sazed | 283 | 96% | 2% | 2% | 1% | 100% | 0% | 6% | 8% | 22% | 28% | 23% | 23% |
| leras | 47 | 83% | 9% | 9% | 0% | 100% | 0% | 49% | 0% | 4% | 6% | 6% | 83% |
| meshwork | 15 | 100% | 33% | 33% | 0% | 100% | 0% | 0% | 0% | 73% | 87% | 93% | 0% |
| marasi | 23 | 78% | 22% | 22% | 0% | 100% | 0% | 0% | 22% | 61% | 61% | 74% | 13% |
| tensoon | 34 | 59% | 12% | 12% | 0% | 100% | 0% | 6% | 3% | 35% | 53% | 44% | 44% |
| oreseur | 8 | 88% | 12% | 12% | 0% | 100% | 0% | 12% | 38% | 12% | 25% | 12% | 100% |
| wyndam | 1 | 100% | 0% | 0% | 0% | 100% | 0% | 0% | 0% | 100% | 100% | 100% | 100% |
| marasi-applied-r-and-d | 60 | 75% | 8% | 8% | 5% | 100% | 2% | 22% | 35% | 22% | 27% | 37% | 85% |

Reading each row against its null:

- **`seq` — kept, as the base.** Present on 89% and ordering 88% of pairs, it is the owner's hand
  and the only dimension that speaks about most tasks. The owner's objection is that it is *all*
  there is, not that it is wrong. `sequence.md` stays the portfolio override and always wins.
- **`unlock` — kept.** Null on 93% of ready tasks, so as a term it is silent for most of the
  list; but the 7% sit where a rank matters: 27% of top-10 pairs are decided by it (meshwork 33%
  active, marasi 22%). It says something specific when it speaks — a prerequisite is as urgent as
  what waits on it.
- **`depth` — struck.** The longest chain below a task orders the same pairs as the count of
  tasks below it (2,771 against 2,773 of 44,229). Two terms for one fact is a rank that cannot
  say why; `unlock` carries it.
- **`inherit` — kept as a rule, not a weight.** Five ready tasks have a live dependent placed
  sooner than themselves (sazed `sa-cytn77t` 190 → 30, `sa-f6yy7w9` 710 → 570;
  marasi-applied-r-and-d `ar-y12vczj` 27 → 5, `ar-jfq4nwv` 200 → 185, `ar-dhxx2aw` −38 → −40).
  A weight cannot express "start where your dependent starts"; a base substitution can, and the
  falsifier shows it moving `ar-y12vczj` from 24 to 5 with the dependent named.
- **`age` — struck as a boost, kept as triage.** It orders 92% of pairs because every task has
  one and few share it — but it is `created` inverted, and `created` is already the key's
  tie-break. What a weight would add is a *direction*: older sooner. The pooled discrete daily
  close hazard says which tasks that promotes:

  | age (days) | 0 | 1 | 2 | 3 | 4 | 5–9 | 10–15 | 16–22 | 24+ |
  |---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
  | at risk | 1918 | 1034 | 934 | 861 | 788 | 746…626 | 595…425 | 407…319 | 306…228 |
  | closed done that day | 43.8% | 8.7% | 7.0% | 5.6% | 2.9% | 1.2–4.3% | 1.2–4.1% | 0.6–2.1% | 0.0% |

  A task past two weeks closes at 1–4% a day and past three weeks at under 1%; boosting age
  promotes the population least likely to close. The §5 sensitivity run (−1 position per week)
  changes top-10 membership in 6 of 9 stores and pulls a 6.5-week-old sazed task from 12 to 8.
  C.4's posture holds on the numbers: past-triage tasks are a decision queue, never a rank input.
- **`queue_h` — struck.** By construction: a ready task has never started (1 of 504 has a
  `→doing` behind a reopen). It is the same quantity as age on this set.
- **`sib_done` and `spawn_done` — kept, folded into one `almost` term.** Each is present on 11%
  of ready tasks and orders under 1% of pairs overall — thin — but they are the literal reading
  of "boost if it's almost done" for `parent` umbrellas and `discovered-from` families, and where
  they exist they are dense: leras 49% (its one umbrella), marasi-applied-r-and-d 22% / 35%,
  oreseur 38%. A term that is null on most tasks and decisive on a few is the `unlock` shape.
- **doc cohesion — kept at anchor level, struck at path level, and its hub problem named.**
  Anchor-level sharing (`path#anchor` equal) covers 46% of ready tasks and orders 31% of top-10
  pairs; path-level covers the same tasks and orders slightly more (33%) by treating every
  section of a design doc as one — which is how a 500-line design document becomes a hub. The
  hubs, by done tasks citing one link:

  | docs: link | done tasks citing it | live tasks citing it |
  |---|---:|---:|
  | `../portfolio/DESIGN-binary-exploration-workbench.md` | 27 | 15 |
  | `docs/design/rig.md#the-executor-marasi-exec-2026-08-14` | 19 | 0 |
  | `docs/DESIGN-meshwork.md#§-6-cli-surface` | 18 | 0 |
  | `docs/leras-relay.md` | 16 | 0 |
  | `docs/reviews/2026-08-19-field-lessons.md#§-punchlist` | 16 | 0 |
  | `docs/design/entity.md#9-the-seam-map-and-the-asks` | 15 | 3 |
  | `docs/design/one-api.md` | 14 | 2 |
  | `docs/design/view-source-seam.md#4-name-resolution-at-install` | 13 | 1 |
  | `docs/DESIGN-meshwork.md#§-12b-trust-boundary` | 13 | 3 |
  | `docs/design/multi-engine.md` | 12 | 2 |

  Fifteen live marasi tasks cite the workbench design beside 27 done ones: each gets the same
  `almost` share (0.64) and the same `docs` count (27). Within that group the term discriminates
  nothing; against tasks on fresh docs it promotes the whole group. That is what the owner asked
  for — "tickets tied to the same design docs … that have already been completed" — and it is
  also a constant offset for a hub. The weight on `almost` is therefore the one the ruling should
  set with this table in view; the `docs` count term is logarithmic for this reason.
- **cost by category — kept as a term, inert at any defensible weight; see §6.** It covers 37%
  of the ready set (leras 83%, marasi-applied-r-and-d 85%, oreseur 100%; meshwork 0%, portfolio
  15%) and its ratio to the corpus median spans 0.23–2.2×, so at half a position per doubling it
  moves a task by ±1 at most; in the falsifier it moved nothing on its own.

## 3. The key — rank space, term by term

The rank of a ready task is a **score**, sorted ascending, then `created`, then `id`:

```
base   = the task's position in today's order (seq, created, id)
         unplaced → the middle of the placed tasks, not the end
         inherit  → min(base, the position of its soonest-placed live dependent)
score  = base
       + unlock · log2(1 + live transitive dependents)
       + almost · max(sibling done share, spawn-sibling done share, doc-peer done share)
       + docs   · log2(1 + done tasks sharing a docs: anchor)
       + cost   · log2(category median tokens ÷ corpus median)
       + age    · weeks old
```

Every term's contribution is in **positions** — "unlock −1.0" means one place sooner than `seq`
put it — and every term renders. `placed by:` becomes the term string; `why <id>` prints it, and
`ready --json` carries it as a list of `{term, value, contribution}`:

```
mw-ps4fzn2  placed by: seq 530 (#6) · unlock −1.0 · almost −1.7 · docs −2.1 → #4
ar-y12vczj  placed by: seq 27 (#24) · inherit → #7 (ar-…) · unlock −2.3 · almost −1.2 · docs −1.0 → #5
te-fc4jpdb  placed by: unplaced → #16 · unlock −2.0 · almost −0.5 · docs −0.5 → #10
```

A rank that cannot say why it ranked is out of scope by construction; this one says so in one
line per task, and a `seq` edit still moves the base.

**Why positions and not `seq` points.** The first shape tried was points: a term moves a task by
steps of the store's median gap between neighbouring `seq` values (measured per store — sazed 10,
leras 3, meshwork 15, portfolio 1). Its own falsifier struck it: sazed's median gap is 10 but its
top ten is numbered 3, 3, 4, 5, 6, 6, 8, 8, 8, 8, so a −2-step term (20 points) carried tasks at
seq 12–15 from positions 28–43 into the top 6 and pushed the seq-4 and seq-6 tasks to 31–36.
Four of sazed's top-10 members and three of tensoon's were replaced. In rank space the same
weights replace none on 8 of 9 stores. Positions are also the only unit that is the same in a
store numbered by ones and one numbered by tens — the invariance a portfolio-wide policy needs,
without measuring a step.

**The unplaced default.** Today `coalesce(seq, 999999)` sorts a new unsequenced task below work
the owner parked at 900 (C.3's wrong-signed default). In rank space an unplaced task's base is
the middle of the placed ones: `te-fc4jpdb` (no seq, unlocks 3) moves from 31 to 10;
`or-s8k9wh6` from 8 to 6. Nothing is authored to get there.

**Exemption.** A task carrying the `owner` label keeps its base and takes no term — a hold is
not work, and a rubric must not move the owner's marker. Today one ready task in nine stores
carries the label (marasi); this repo's own reveal marker `mw-59f0t1q` does not, and the
falsifier duly moved it from 1 to 3. The exemption has to be the label, and the marker needs it.

## 4. Weights as policy, portfolio-wide

One table, shared by every store, declared and not fitted, never per-repo:

| term | weight | unit | what one unit means |
|---|---:|---|---|
| `unlock` | −1.0 | positions per doubling of live transitive dependents | a task that 3 tasks wait on starts 2 places sooner |
| `almost` | −2.0 | positions × done share of its siblings, spawn-siblings or doc peers, whichever is highest | the last open task of a finished family starts 2 places sooner |
| `docs` | −0.5 | positions per doubling of done tasks sharing a `docs:` anchor | a section with 7 closed tasks behind it: 1.5 sooner |
| `cost` | +0.5 | positions per doubling of the category's median tokens over the corpus median | a 4×-cheaper category: 1 sooner; 4×-dearer: 1 later |
| `age` | 0 | positions per week | triage, not rank (C.4) — measured in §5, set to zero by policy |
| `inherit` | on | rule | base := its soonest-placed live dependent's position |

Where it lives: `rubric.toml` beside `repos.toml` in the portfolio repo — the registry every store
already reads through `MESHWORK_PORTFOLIO` — with these values compiled in as the default, so a
store with no registry ranks the same way and a repo never carries its own copy (C.2's
objection was the per-repo file; a shared one is what "reasoned about in one place" means). The
file is data, reviewed like code; a store cannot override it. `seq` and `sequence.md` remain the
owner's two hands: `seq` moves the base, `sequence.md` wins outright.

## 5. The falsifier — the rubric against today's order, every disagreement named

Rank space, the §4 weights, on every store. "Moved > 1" counts tasks that shifted more than one
position anywhere in the list; "top-10 members changed" counts tasks that entered or left the
top ten; the per-store tables list every task that moved into, out of, or within the top ten,
with the term that moved it — or `displaced` when its own terms sum to under half a position and
something else moved past it.

| store | ready | positions moved | moved > 1 | top-10 members changed | top-8 order changed | rows below |
|---|---:|---:|---:|---:|---|---:|
| portfolio | 33 | 6 | 0 | 0 | yes | 2 |
| sazed | 283 | 254 | 191 | 0 | yes | 9 |
| leras | 47 | 30 | 28 | 0 | yes | 3 |
| meshwork | 15 | 10 | 3 | 0 | yes | 10 |
| marasi | 23 | 22 | 16 | 0 | yes | 9 |
| tensoon | 34 | 33 | 26 | 1 | yes | 10 |
| oreseur | 8 | 7 | 2 | 0 | yes | 7 |
| wyndam | 1 | 0 | 0 | 0 | no | 0 |
| marasi-applied-r-and-d | 60 | 56 | 45 | 1 | yes | 9 |

Fifty-nine rows in nine stores; by cause: `almost` 21, `displaced` 24, `unlock` 6, `docs` 5,
`unplaced` 2, `inherit` 1, `cost` 0. The membership of every top ten is the owner's on eight
stores; the two exceptions are the unplaced task and the inherit finding named above. Inside the
top ten the rubric re-orders by one to three places, and each move reads as one line.

### portfolio

| id | today | rubric | term that moved it | terms | title |
|---|---:|---:|---|---|---|
| po-nwtwwz7 | 2 | 1 | unlock | seq 4 · unlock -1.0 | The readiness chain — every surface the post lands on goes current bef |
| po-af4cmqn | 1 | 2 | displaced | seq 1 | The push debt is the portfolio's largest single exposure — 660 commits |

### sazed

| id | today | rubric | term that moved it | terms | title |
|---|---:|---:|---|---|---|
| sa-4z4x43m | 3 | 2 | almost | seq 3 · almost -2.0 · docs -0.8 · cost -0.2 | marasi-applied-r-and-d: personalised PageRank runs on BOTH engines and |
| sa-12a1w3n | 2 | 3 | almost | seq 3 · almost -0.8 · docs -0.8 | Land the foreign-operator hook (Seam G) — a Rust operator at a copy bo |
| sa-efs0gb2 | 5 | 4 | almost | seq 5 · almost -1.0 · docs -0.8 · cost -0.2 | marasi: lift a `personal` key for the graph door's tenth row — persona |
| sa-tcmq29m | 4 | 5 | displaced | seq 4 | Gate the per-QUESTION resident growth on the personalised walk — a set |
| sa-k6smz20 | 9 | 6 | almost | seq 8 · almost -2.0 · docs -1.4 | The in-flight wall reaches 4 of 9 leras view families — six sessions s |
| sa-rrdy1bp | 6 | 7 | displaced | seq 6 | Overlay::land re-folds the WHOLE δ log on every push — a run of N sing |
| sa-dfkeedk | 7 | 8 | displaced | seq 6 | Finish the 779afc4 → 9139523 re-pin: acts 2 and 3 were DEFERRED, so ev |
| sa-bbzy7k8 | 10 | 9 | almost | seq 8 · almost -1.6 · docs -0.8 | Measure the graph door on BOTH axes and BOTH engines — install wall, p |
| sa-w2fftwz | 8 | 10 | displaced | seq 8 | check-consumers.sh builds ONE of at least four path-pinned consumer re |

### leras

| id | today | rubric | term that moved it | terms | title |
|---|---:|---:|---|---|---|
| le-5qbjq77 | 6 | 5 | almost | seq 70 · almost -2.0 · cost +0.2 | Finish the why-investigation: S-B2 rung 2 (per-epoch argmax probe), th |
| le-zfke2g7 | 7 | 6 | unlock | seq 75 · unlock -1.0 · almost -0.8 · docs -0.8 | Digest every durable extent and record and chain the digests through C |
| le-65q9dgz | 5 | 7 | displaced | seq 65 | The fold kernels carry the verdict path's `.2d`-only shape — profile ` |

### meshwork

| id | today | rubric | term that moved it | terms | title |
|---|---:|---:|---|---|---|
| mw-pcjm4pb | 2 | 1 | almost | seq 460 · almost -1.3 · docs -0.8 | Take outbound asks out of the sender's ready and next and list them un |
| mw-21qzw9n | 3 | 2 | unlock | seq 470 · unlock -1.0 · almost -1.0 · docs -0.8 | Keep an ask surfaced until its answering task is terminal, rendered an |
| mw-59f0t1q | 1 | 3 | displaced | seq 180 | Flip the meshwork reveal — one owner word, but the trust surface harde |
| mw-ps4fzn2 | 6 | 4 | docs | seq 530 · unlock -1.0 · almost -1.7 · docs -2.1 | Let run cargo test scope to a crate and a test target with dash-free p |
| mw-vwdm3ed | 4 | 5 | unlock | seq 480 · unlock -1.0 | Add --to, --answers and --relates to add and set |
| mw-rwb77wp | 7 | 6 | docs | seq 540 · almost -1.6 · docs -1.9 | Add the lacks predicate — the inverse of contains, same reader, same c |
| mw-qhmek05 | 8 | 7 | docs | seq 550 · almost -1.6 · docs -1.9 | Let exists take a single-segment glob, and refuse a directory target f |
| mw-zhdypdz | 5 | 8 | displaced | seq 490 | Add set --body, --parent, --from, and a docs replacement to set |
| mw-ge51hf1 | 10 | 9 | almost | seq 570 · almost -2.0 · docs -1.2 | Point smoke's fast tier at the lib target — cargo test --bins runs zer |
| mw-1cmpywn | 9 | 10 | almost | seq 560 · almost -1.3 · docs -0.8 | Put the four session rules in prime's footer in two lines, inside the  |

### marasi

| id | today | rubric | term that moved it | terms | title |
|---|---:|---:|---|---|---|
| ma-w2knmyt | 2 | 1 | almost | seq 66 · almost -2.0 · docs -1.0 | oreseur: a `column` projection on the `jsonl` family — `column <key> s |
| ma-tpmcdnk | 1 | 2 | displaced | seq 30 | meshwork: a `run cargo test` verify cannot scope to a package or a tes |
| ma-ssfj0b4 | 4 | 3 | almost | seq 80 · almost -1.6 · docs -1.2 | Land a columnar door — a `parquet` source kind whose schema is read fr |
| ma-zvjh1an | 7 | 4 | unlock | seq 120 · unlock -2.0 · almost -1.3 · docs -1.9 | Build the injectable source and step control — drive epochs from a liv |
| ma-eh03e8g | 3 | 5 | unlock | seq 70 · unlock -1.0 · cost +0.2 | Build the serve socket (Seam C.1) with replay after disconnect |
| ma-56pjx3a | 8 | 7 | docs | seq 140 · unlock -1.0 · almost -1.3 · docs -1.9 | Build the workbench example rig — an unknown binary, scanner views, gr |
| ma-3fk5s6v | 5 | 8 | displaced | seq 95 | marasi-clock retracts: emit assert tick(n) + retract tick(n−1) — keyed |
| ma-rvgf4qc | 10 | 9 | docs | seq 160 · unlock -1.0 · almost -1.3 · docs -1.9 | Land the instruments that inform — dependency_candidates, suggestions, |
| ma-xedcfas | 9 | 10 | almost | seq 150 · almost -2.0 · docs -1.0 | Persist Life forks between verbs — a delta-log garden store over Recor |

### tensoon

| id | today | rubric | term that moved it | terms | title |
|---|---:|---:|---|---|---|
| te-jaahmpg | 2 | 1 | almost | seq 25 · almost -1.6 · docs -1.5 · cost +0.2 | Build the lens vocab registry (use vocab …) — declaration-schema.md §5 |
| te-k5g8yd2 | 1 | 2 | displaced | seq 20 · cost +0.2 | Prove the field-grain span contract at the seam — per-concept spans ri |
| te-x48ej1y | 4 | 3 | almost | seq 30 · almost -2.0 · docs -0.8 | kwaan: design the hosted re-dissection stage as a CASCADE consumer of  |
| te-m4399se | 5 | 4 | almost | seq 35 · almost -1.6 · docs -1.5 | CVE-inexpressibility as one runnable artifact — a hostile fixture per  |
| te-cm7wr6d | 3 | 5 | displaced | seq 30 · cost +0.2 | Flatten LenExpr evaluation to a load-time postfix opcode array (catalo |
| te-fmnyqcz | 9 | 7 | almost | seq 44 · almost -1.6 · docs -1.6 | Extract the corpus-stats core from tensoon-probe (PLAN §2.3) — field c |
| te-76sk854 | 7 | 8 | displaced | seq 40 | kwaan v2 pass: static-offset composition through nested units — lift h |
| te-t7jxvkc | 8 | 9 | displaced | seq 42 | kwaan v2 pass: prune the body — a sized region or budgeted repetition  |
| te-fc4jpdb | 31 | 10 | unplaced | seq — · unlock -2.0 · almost -0.5 · docs -0.5 | switch heuristic — slice 4, the single-message budget: `budget hyps N, |
| te-b58qpv2 | 10 | 14 | displaced | seq 44 | kwaan symbolic form: follow a bitfield sub-field feed (FeedKey::Sub) a |

### oreseur

| id | today | rubric | term that moved it | terms | title |
|---|---:|---:|---|---|---|
| or-gezmda4 | 4 | 1 | almost | seq 130 · almost -2.0 · docs -1.2 · cost +0.2 | Parse EVTX's binary container into per-event records with refs into th |
| or-5paebyy | 1 | 2 | displaced | seq 110 · cost +0.2 | Add a live capture source with metered drops |
| or-yqy6gzw | 2 | 3 | displaced | seq 120 · cost +0.2 | Port the big-fixture cold-cache protocol to Linux |
| or-kck7v9a | 3 | 4 | displaced | seq 130 · cost +0.2 | Resolve a stitched artifact to a span set, not a single span |
| or-s8k9wh6 | 8 | 6 | unplaced | seq — · cost +0.2 | Signed integer lane columns — the Life offsets lane is the first refus |
| or-9c7n6hp | 6 | 7 | displaced | seq 160 · cost +0.2 | Key the shim's author block on CLAUDE_CODE_SESSION_ID as well as the b |
| or-j2qxf6j | 7 | 8 | displaced | seq 170 · cost +0.2 | Give docs: a cross-repo form so a sibling design doc does not dead-end |

### marasi-applied-r-and-d

| id | today | rubric | term that moved it | terms | title |
|---|---:|---:|---|---|---|
| ar-pp5jbep | 4 | 3 | almost | seq 2 · almost -2.0 · docs -0.5 · cost +0.3 | Answer FP-AMB with a graph walk, not a ranking — personalised PageRank |
| ar-xyns8w1 | 3 | 4 | almost | seq -30 · almost -1.0 · cost +0.4 | Rule whether shell arithmetic is on the bench — the refusal now names  |
| ar-y12vczj | 24 | 5 | inherit | seq 27 · inherit → #7 · unlock -2.3 · almost -1.2 · docs -1.0 | Give this lab a skill that runs an external benchmark from the harness |
| ar-qfss3bq | 5 | 6 | almost | seq 4 · almost -1.2 · cost +0.3 | Find a model-free proxy for when contiguity beats global selection — a |
| ar-7n0wwyn | 6 | 7 | displaced | seq 4 · cost -0.2 | marasi: a graph roster row the door cannot build ABORTS the host — per |
| ar-ewn6p4p | 7 | 8 | displaced | seq 5 · cost +0.2 | Answer sazed's question about Overlay::land — is the leras arm's +26%  |
| ar-43npnnk | 8 | 9 | displaced | seq 6 · cost -0.2 | marasi: a marked edge re-lands one table per view, so a consuming engi |
| ar-92ejgag | 9 | 10 | almost | seq 6 · almost -1.2 · cost +0.3 | Fork the QUESTION, not the declaration — N query variants scored in on |
| ar-5mjeabe | 10 | 12 | displaced | seq 7 · cost -0.2 | marasi: a rig cannot choose what reaches the tap — `sink.views` is dec |

### Sensitivity — what each weight is buying

The same falsifier with one weight changed; the column to read is top-10 members changed.

| run | portfolio | sazed | leras | meshwork | marasi | tensoon | oreseur | wyndam | m-a-r-d | reading |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| declared (§4) | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | the owner's top ten, re-ordered inside |
| `age = −1` per week | 0 | 1 | 1 | 1 | 1 | 2 | 0 | 0 | 1 | age enters six top tens; the entrants are the oldest, not the likeliest to close |
| `unlock = −3` | 1 | 0 | 0 | 2 | 0 | 1 | 0 | 0 | 1 | this repo's parked mirror `mw-cvw8` (seq 900, unlocks 5) climbs 14 → 8: a graph weight strong enough to matter un-parks a ruling |
| `almost = 0, docs = 0` | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | without the doc and family terms the rubric is `unlock` + the unplaced fix; sazed's top ten does not move at all |
| `inherit = 0` | 0 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | 1 | membership identical; `ar-y12vczj` stays at 24 — the one finding the rule exists for |
| points space, §4 weights | 0 | 4 | 0 | 0 | 0 | 3 | 0 | 0 | 2 | struck: the numbering-dependent shape (§3) |

Two readings the owner should have beside the tables. First, `almost`/`docs` are the terms doing
the work: 26 of the 35 attributed moves are theirs, and nearly all of them are 1–3 places inside
a top ten. Second, no weight tried here changes who is in the top ten on the two largest stores
(sazed, portfolio) except age and points-space, both struck — the rubric as declared is a
re-ordering of the owner's list, not a replacement for it. Whether that is the right strength
is the ruling; the number that would change it is `almost`.

## 6. The effort question, answered with numbers

The owner named t-shirt-sized effort as a candidate dimension. An authored size is an estimate
and outside REQUIREMENTS §3 as written; measured cost by category is an observation inside it
(the R-A scope ruling). The test: does the measured proxy do the work an authored size would?

Sources: `mine_cost.py --json` on 2026-09-21 (transcripts through that morning; 273 attributed
tasks, 66 categories, corpus median 105k fresh tokens per task).

| question | number | denominator |
|---|---|---|
| categories with n ≥ 7 costed tasks | 10 | 66 categories seen |
| spread of category medians | **9.6×** (`meta/distribution` 24k → `kwaan` 227k) | was **24.2×** on 2026-09-08 (`docs/cost-baseline.md`), 8 categories then |
| share of per-task log-token variance the category explains (η², one-way) | **0.11** | 154 tasks in the 10 categories |
| within-category spread, p90 ÷ median | 1.8× – 4.7× | per category, below |
| ready tasks whose category has a costed row | **186 / 504 (37%)** | leras 83%, m-a-r-d 85%, oreseur 100%; meshwork 0%, portfolio 15%, sazed 23% |
| what the term moved in the falsifier at 0.5 positions per doubling | 0 tasks on its own | 59 rows |

| category | n | median | p90 | p90 ÷ median |
|---|---:|---:|---:|---:|
| (none) | 59 | 139k | 658k | 4.7× |
| seam/marasi | 15 | 83k | 201k | 2.4× |
| experiment/memory | 14 | 149k | 278k | 1.9× |
| seam/sazed | 13 | 154k | 315k | 2.0× |
| experiment/thesis-gate | 12 | 191k | 732k | 3.8× |
| rig/exec | 11 | 105k | 187k | 1.8× |
| kwaan | 9 | 227k | 888k | 3.9× |
| experiment/dock-firmware | 7 | 187k | 379k | 2.0× |
| meta/distribution | 7 | 24k | 62k | 2.6× |
| engine/plan | 7 | 98k | 211k | 2.2× |

Three things these numbers say, in order of weight:

1. **The proxy is a category constant, and category explains a tenth of the variance.** Two
   tasks in one category differ by 2–5× at the p90; the category tells you 11% of what a task
   will cost. An authored size that landed a task in the right half of its category's spread
   would carry more information than the category does — the test the proxy needed to pass, and
   did not.
2. **The proxy is absent where a rank is most needed.** It reaches 37% of ready tasks, and 0% of
   this repo's and 15% of the portfolio's. A term that is null on the two stores the owner ranks
   by hand most is not doing the authored field's job on them.
3. **The headline moved.** The 24.2× spread that answered rung 0 "yes" on 09-08 is 9.6× thirteen
   days later on a corpus that grew from 8 costed categories to 10 — still past the 3× threshold,
   and still an observation that reorders as the corpus does. A weight fitted to it would be
   fitted to a date.

So: **measured cost by category does not do the work an authored size would**, and this spike
does not write the field in. The ruling has three shapes to choose among, with the numbers above
attached: (a) an authored `size:` (S/M/L, or hours) as a fenced exception to §3, its weight
declared in §4's table in positions per size step, `lint` warning when it is absent on a ready
task; (b) no size — the `cost` term is struck and the rubric runs on structure, family and
docs, which §5 shows is where the movement is anyway; (c) a derived proxy that is not a
category — none was found inside this brief (body length and verify shape were not tested,
because neither is effort).

## 7. What this spike proposes, and what it leaves to the ruling

For `mw-ryd25rq`:

1. **Adopt the rank-space key** (§3): `seq` as the base, `sequence.md` winning outright, an
   unplaced task at the middle, `inherit` as a base rule, four weighted terms in positions, every
   term rendered in `placed by:` and `why`. Struck on the numbers: `depth`, `queue_h`, age as a
   boost, points space, path-level doc cohesion.
2. **Declare the weights once, portfolio-wide** (§4): `rubric.toml` beside `repos.toml`, defaults
   compiled in, never per-repo, never fitted; the `owner` label exempts a task from every term,
   and this repo's reveal marker needs the label.
3. **Set `almost` with the hub table in view** (§2): it is the term that moves tasks, and on a hub
   doc it moves them as a group.
4. **Rule the effort field** (§6): (a) authored size as a fenced exception, (b) no size and the
   cost term struck, or (c) a non-category proxy nobody has yet. The measured proxy does not stand
   in for it.
5. **Rewrite the two build tasks after the ruling**: `mw-ex5x0y2` (bands, the composite key,
   `placed_by`) becomes the rank-space key with `placed_by` as the term string and no `[bands]`;
   `mw-jzga8yj` (decay and WIP as config) keeps its triage and WIP halves and loses nothing — age
   is theirs, not the rubric's.

What this spike did not do, by its brief: no code, no format change, no field written, no weight
fitted. `scripts/mine_rubric.py` is the runnable falsifier — re-run it after any ruling to see
what the ruled weights do to today's lists before a line of Rust is written.
