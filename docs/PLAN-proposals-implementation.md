# PLAN — the six unruled documents, as a queue

**Status: implementation plan, unruled. Nothing filed, no store touched.** Written 2026-09-07
from `docs/ASKS-analytics-and-field-study.md`, `docs/FIELD-STUDY-session-transcripts.md`,
`docs/PROPOSAL-analytics.md`, `docs/PROPOSAL-prioritization.md`,
`docs/PROPOSAL-spec-traceability.md` and `portfolio/DRAFT-tasks-weft.md`, each read whole,
plus the source the tasks below name (`src/cli/prime.rs`, `src/addressed.rs`, `src/tables.rs`,
`src/lint.rs`, `src/docs.rs`, `src/verify_dsl.rs`, `src/cli/add_batch.rs`) and the live store
(`q` over this repo; `show` on the five inbound asks in their own stores). §8 separates what was
observed from what is argued. The evidence companion (`PROPOSAL-prioritization-evidence.md`)
was not re-read; every number quoted from it is quoted through its proposal.

**What this file is.** The review the owner asked for (§1–§2), the decisions only the owner can
make, gathered into one ledger with a recommendation per row (§3), the order the work should
follow and why (§4), and the work itself cut into store-sized tasks as `add --batch` documents
with handles, edges, verifies and doc refs already written (§5). §7 is the filing ritual; it
was dry-run against the binary before this file was committed.

---

## 0. The one-paragraph version

Fifty-eight tasks in six waves. **Wave 0** needs no ruling and starts today: the five defects
adopters have already filed asks for, the three transcript measurements the first reader asked
for, and the skill sentences every eruption in the field study keeps asking for. **Wave 1** is the
analytics view layer — the spec first (FORMAT.md §Views with a multi-store fixture blessed at two
clock stamps), then registration — and nearly everything after it consumes those views. **Wave 2**
is the field study's Tier 1: the inbox made honest, the write flags, the handoff given an author
and an age, the DSL grown past `-p`. **Wave 3** is the pulse in `prime` and the graph fields, in
Rust, pinned to the SQL by differential tests. **Wave 4** is the prioritization floor (bands,
placement, decay, WIP) and the lint findings the views make cheap. **Wave 5** is the verbs that
need a §6 nod (`stats`, `asks`, `portfolio search`), reads-never-prune, and the skill rewrite.
**Wave 6** is spec traceability, gated on the weft's first pointer-rot numbers. Six owner rulings
(§3) gate waves 1–6; the graph carries that gating as `needs:` edges on real tasks, so `ready`
shows exactly what is waiting on the owner and nothing pretends otherwise.

---

## 1. The six documents and how they lock together

| document | its claim | what it needs from the others | what the others need from it |
|---|---|---|---|
| **Field study** (09-06) | the local loop is trusted; the cross-repo half is invisible (inbox capped, suppressed, mis-queried; no CLI path for `to:`/`answers:`; the handoff outranks the human); 723 hand-edits, 408 `--help` probes are discoverability tax | the `asks` view for an honest inbox line; a `mentions` pass for "cites N closed" | the eruptions are the evidence every ruling in §3 weighs; Tier 1 #2 (write flags) is the UI's whole write surface |
| **Analytics** (09-06) | thirteen SQL views over the six tables plus `clock`, published in FORMAT.md, consumed by a five-line pulse in `prime`, a `stats` verb, and external readers | a ruling that views are contract; reads-never-prune; `set --handoff` minting a log line | prioritization rungs 1 and 3 collapse into "consume `graph`, `facts`, `hazard`"; the UI's analytics feed; the weft's `mentions`-shaped W6 within a store |
| **Prioritization** (09-01) | a sealed computed floor under `sequence.md` and `seq`; bands per category; `placed_by` recorded; age triages, never boosts; lanes | `graph` (unlock, depth, inherit, lane) and `hazard` from the analytics views; six rulings | the `[decay]`/`[wip]` config the pulse renders; the `needs-behind` finding; the placed-by histogram that decides its own next rung |
| **ASKS** (09-07) | the owner is a resource with a queue: 588 h of agent idle, 153 h of it while the owner was demonstrably working elsewhere; the conformance corpus cannot test the union third of the views; `clock` should be a parameter | nothing — it is the first external reader's shopping list | A1–A8 are folded into Waves 0–1 below; A3/A4/A8 are the measurement tasks; A2/A5/A6 are inside the views spec task |
| **Spec traceability** (08-14) | `covers:` edges with a pinned content hash; `spec-drift` lint; `spec audit` | a cross-repo `docs:` form (the oreseur ask); three new verbs (§6 rulings); `gestalt/` in a repo | the weft reads pinned `covers:` as exact edges (§3.8 there) |
| **Weft draft** (portfolio, 09-01) | a cross-plane freshness instrument in `portfolio/weft/`: stamped artifacts, `docs:` pointer rot across nine stores at committed refs with `--as-of`, evidence moved after the close anchor, prose↔store contradictions; it federates and never ranks | the conformance corpus byte-equal (WF2); per-repo `q --json` that never prunes; the cross-repo `docs:` form so its resolver reads one spelling; committed cost data (WF12) | WF2's first run is the falsifier the traceability lane should wait for; W1 (`verify-target-missing`) is single-store-decidable and belongs in meshwork's lint, not the weft |

The division of labour the weft states (its §3.2) is the rule this plan applies everywhere:
*a check belongs to the weft iff no single member can decide it.* Ordering, the pulse, the
inbox, verify hygiene and `covers:` are all decidable inside one store, so they are meshwork's.
Aggregator lag, cross-repo pointer rot at a ref, and prose contradictions in STATUS are not, so
they stay the weft's, and nothing below duplicates them.

---

## 2. Review findings — where the documents disagree with each other or with the tree

Each of these changes a task below; none is a reason to wait.

1. **Two proposals mint the same requirement IDs.** Spec traceability uses `MW-S1`…`MW-S10`;
   analytics uses `MW-S1`…`MW-S15`, and the ASKS document and the weft draft already cite the
   analytics numbering (`MW-S10`, `MW-S14`) and the traceability numbering (`MW-S4`–`S6`)
   respectively. Traceability is the smaller and older document; it renumbers to `MW-T*`
   (task `t-renumber`, Wave 0). Prioritization's `MW-R*` collides with nothing.
2. **The `asks` view encodes the inbox rule the field study calls a defect.** Appendix A's
   `a_ans` drops an ask the moment any non-dropped answer exists — exactly the suppression that
   hid `ar-hcvyxy5` behind an open answer filed 72 minutes earlier. A view is data, not policy:
   `asks` must expose the state (`answer_open`, `answer_done`, `unanswered`) and let `prime`
   apply whichever rule is ruled (`a-views-spec`). Blessing the view as written would pin the
   defect into the conformance corpus.
3. **Rung 1's `-unlock` tie-break breaks MW-R2's own promise.** The composite key in
   prioritization §4 places `-unlock` at rung 1, while MW-R2 says a store without `[bands]`
   orders byte-identically to today. Two open tasks at one `seq` where one unlocks something
   would reorder. The tie-break enters the key only when `[bands]` exists (`q-bands`).
4. **The field study's `no-seq` lint contradicts the floor.** Tier 2 #9 asks for a warning on
   live tasks with no `seq` because they sort last. The prioritization proposal's reading 4 says
   the default is wrong-signed and fixes the default. Fixing the default retires the warning;
   the warning is not filed. Until `q-bands` lands, `pulse.no_seq` is the count.
5. **A1 is already an environment variable.** `MESHWORK_TODAY` is contract (DESIGN §15.6) and
   MW-S1 has `clock` honour it, so `MESHWORK_TODAY=2026-08-15 meshwork q "SELECT * FROM pulse"`
   is the time machine with no flag. What is missing is the *test* at a second stamp, which
   `a-views-spec` adds. The `--as-of` flag is sugar and waits for the verb ruling. Two caveats
   the spec must state: the variable also fixes minting stamps, so it is a read-only idiom; and
   it ages the *current* store, whereas the weft's `--as-of` reads the store *at a ref* — both
   halves together are the true as-of, and FORMAT.md §Views says which is which.
6. **Rung 0's verify depends on rung 1.** `conformance::views_golden` cannot pass until the
   binary registers the views. The spec task carries the red test; the registration task turns
   it green (`a-views-spec` → `a-views-register`).
7. **Two Rust passes, not one.** The analytics proposal computes the pulse in Rust for the
   100 ms `prime` gate. `ready` has the same gate and, from rung 2 on, orders by `unlock`, so
   the `graph` columns need a Rust pass too. Two views get differential tests against their SQL
   (`g-graph-pass`, `p-prime-pulse`); `why`, `lint`, `show` and `stats` have no gate and read
   the SQL views directly.
8. **The write flags need a reading of §15.12, not a new fence.** DESIGN §15.12 says `to:` /
   `answers:` are "frontmatter only — no verb, no flag, no transport". The field study read that
   as "no `--to`". The ruling's substance is *no transport*; a flag that writes local frontmatter
   is the same act as the hand-edit it replaces, and the first reader's argument is the sharper
   one: FORMAT.md requires third-party writes to go through the binary, so a field with no flag
   is a field no external tool can set. That is ruling R-B's first row.
9. **The views' SQL is not portable, and FORMAT.md must say so.** The weft's `q` is SQLite; the
   UI is not DataFusion. `regexp_like`, `FILTER (WHERE)`, `unnest(generate_series())` and the
   label-propagation CTE will not run there. §Views therefore publishes each column's
   *derivation* in prose as the normative text, the DataFusion SQL as the reference
   implementation, and `expected/views/*.json` as the portable contract.
10. **The traceability build has a cheaper falsifier than its own incident.** The weft's WF2
    stamps every `docs:` pointer (assertion commit vs target's last commit) across nine stores —
    the approximate form of `covers:`+hash. If WF2's first run shows pointer targets rarely move
    after assertion, exact pins are not worth a format change; if they move constantly, the
    approximate signal is noise and pins are justified. `t-covers` waits on that number.
11. **The registration cost is unmeasured.** DataFusion plans a `CREATE VIEW` body at
    registration. Forty views in `session_for` would be paid by every verb, including the two
    gated ones. The registration task registers views only in the sessions that query them
    (`q`, `portfolio q`, `stats`, lint findings) and adds a perf row; this is an assumption to
    test, not a fact observed.
12. **Measurements do not block the eruption fixes.** The first reader would put A3/A4 in front
    of everything. They are a day each and they will re-price the tiers, but the Tier 1
    mechanisms are evidenced by the eruptions themselves and need no price. They run in
    parallel (Wave 0), not in front.
13. **The owner-latency remedy is not a store feature.** 588 h is a session-layer defect; the
    tool runs nothing in the background (REQUIREMENTS §3) and cannot see a stopped session. The
    measurement is filed (`m-waits`); the remedy belongs to meshwork-ui's Rail and to the harness,
    and this plan says so rather than inventing a verb for it.
14. **W1 belongs to meshwork.** The weft's WF3 bundles "verify names a path absent at HEAD"
    with "evidence moved after the close anchor". The first is decidable by one store with no
    git and is the sazed ask (`d-verify-path-missing`); the second needs history and stays with
    the weft until analytics rung 5 (git-joined) has its own evidence.
15. **`clock` honouring `MESHWORK_TODAY` is not the same as the shim's stamp.** The binary's
    `clock::today()` returns the override verbatim; the `clock` table must parse it in both
    conforming forms and reject anything else loudly, or a golden at `MESHWORK_TODAY=fixed`
    would silently produce NULL ages.

---

## 3. Rulings

Six owner-gated tasks, one per group. Each row names the decision, where it is argued, and the
recommendation this plan was written under. When a group is ruled, the owner replaces the last
cell of its header row with the word RULED, the date, and one line per item; the group's task
verifies on that marker and the tasks behind it unblock. Recording a ruling also means: the
accepted MUSTs are appended to REQUIREMENTS under the proposal's IDs, DESIGN §15 gains the
decision, and TRACE.md gains `planned` rows in the same commit.

### R-A — analytics as contract

| | |
|---|---|
| **R-A** | analytics rulings 1, 2, 4, 5, 6 · argued in `PROPOSAL-analytics.md` §12 · **unruled** |

| item | decision | recommendation |
|---|---|---|
| A.1 | thirteen named views + `clock` are contract, published in FORMAT.md §Views, additive (no format or schema bump) | yes; the derivation prose is normative, the SQL is reference, the blessed JSON is the portable test |
| A.2 | the pulse renders inside `prime`'s weather, ≤ 5 lines, ≤ 800 bytes, top of weather | yes, as behaviour; the `cites N closed` tail on the next block with it |
| A.3 | reads never prune: `q`, `portfolio q`, `stats` never touch `sequence.md`; `next`/`ready` keep pruning | yes — the UI, the weft and every analysis script already avoid the verb for this reason |
| A.4 | the three §3 scope clauses: log-derived lifecycle statistics are not estimates or time tracking; ordering the ready set is not sprint semantics; period counts with denominators are not a burndown | yes, recorded in REQUIREMENTS §3 in the style of the three existing rulings |
| A.5 | `set --handoff` mints a log line so a handoff has an author and an age | yes; the line form lands in FORMAT.md's log grammar |
| A.6 | closure convention pinned in FORMAT.md: `done` means current status; a reopened task is not a closure; date-only stamps are midnight UTC | yes (ASKS A5) |

### R-B — the session ritual: inbox, write surface, skill

| | |
|---|---|
| **R-B** | field study Tier 1 #1–#3, Tier 3 #17a; DESIGN §15.12 amendments · argued in `FIELD-STUDY-session-transcripts.md` §2.1–§2.3 and `ASKS-analytics-and-field-study.md` §4 · **unruled** |

| item | decision | recommendation |
|---|---|---|
| B.1 | `--to`, `--answers`, `--relates` on `add` and `set`; `set --body`, `--parent`, `--from`; `set --docs <old> <new>` — read §15.12's "no flag" as "no transport": a flag writing local frontmatter is in scope | yes; precedent is `set --cat/--verify/--title` (ruling 2026-08-10); `e2e::cli_surface_frozen` re-blessed once |
| B.2 | an ask stays surfaced in the addressee's `prime`/`ready` until its answering task is **terminal**, rendered `answered-by <gid> (open)` meanwhile — amends §15.12's "un-surfaces the moment a non-dropped answer exists" | yes; an open intent to answer is not an answer, and inbox-zero by frontmatter alone is the defect |
| B.3 | `prime` lists every inbound ask, or the count plus the oldest age plus the exact verb that lists them; never a bare "and N more" | yes, as behaviour under the byte budget |
| B.4 | asks older than N days rank at parity with `next →` (the product-intent conflict: "never displace next" vs the owner's model) | rule it explicitly either way; recommendation: an ask past the triage age leads the next block once, then yields |
| B.5 | outbound `to:` tasks leave the sender's `ready`/`next` and list under an `asks out` line | yes; "Nothing to build here" handoffs are a status spelled by hand |
| B.6 | the SessionStart hook injects the skill body alongside `prime` (plugin-side) | yes, after `f-prime-footer-skill` ships the one-line fallback |

### R-C — prioritization

| | |
|---|---|
| **R-C** | prioritization rulings 1–9 · argued in `PROPOSAL-prioritization.md` §10 · **unruled** |

| item | decision | recommendation |
|---|---|---|
| C.1 | bands as the tier mechanism: declare once per category, keep `seq` for exceptions | yes |
| C.2 | bands live in per-repo `config.toml` only; `sequence.md` stays the only cross-repo override | yes |
| C.3 | `default` band = 500 once a store opts in; 999999 stays for stores that have not | yes |
| C.4 | aging posture: past-triage tasks are a decision queue rolled up by category, never boosted, never auto-dropped; `owner`-labelled markers exempt; triage age is config (14 d) | yes — the most disciplined call in either proposal, and ASKS §5 agrees |
| C.5 | value stays unit (no authored `weight`/`cod`/`due`) until the placed-by histogram argues otherwise | yes |
| C.6 | `needs-behind` is a finding; `[bands] inherit = true` is opt-in, default off | yes; the mirror chain under the v1 gate (`mw-v4ej` at 150 waiting on `mw-cvw8` at 900) wants its own one-line ruling: re-seq the gate behind the park, or un-park |
| C.7 | the token measurement from transcripts is authorized as the gate on rungs 5–6 — the same ruling the weft's Q6 and the field study's A4 pricing need | yes; one script, one committed data file, three consumers |
| C.8 | `stats` carries the placed-by histogram and the hazard table (not `lint --stats`) | yes; lint reports findings, `stats` reports measurements — ruled together with R-D |

### R-D — verbs that touch §6

| | |
|---|---|
| **R-D** | `stats`, `asks`, `portfolio search`; `show --comments <N>` · argued in `PROPOSAL-analytics.md` §5.3, `FIELD-STUDY-session-transcripts.md` §1.2 and §3 Tier 1 #3/#3a · **unruled** |

| item | decision | recommendation |
|---|---|---|
| D.1 | `stats [--window 7d\|28d] [--json]` and `portfolio stats` | yes; canned SELECTs over the views, the `ready`/`search` pattern, no language |
| D.2 | `asks` — inbound and outbound, unsuppressed, with answered-by state and age; the verb agents typed as `inbox`/`addressed`/`portfolio show` 22 times | yes; the `asks` view makes it one canned query |
| D.3 | `portfolio search <term>` (or `search --portfolio`) | yes; the lab replaced `prime` with a hand-rolled cross-repo query for lack of it |
| D.4 | `show --comments <N>` (count, not boolean) | optional; cheap, low evidence |

### R-E — the verify grammar

| | |
|---|---|
| **R-E** | leras ask `le-yppwtpq`, field study Tier 2 #6 · argued in `FIELD-STUDY-session-transcripts.md` §2.5 item 3 and DESIGN §12b · **unruled** |

| item | decision | recommendation |
|---|---|---|
| E.1 | `run cargo test package=<crate> target=<name> <filter>`: dash-free tokens meshwork translates to `-p`/`--test`; no author text ever becomes a flag | yes — the measured 18-minute compile against a 5-minute timeout is the money leak |
| E.2 | `lacks <path> <lit\|/regex/>` — the inverse of `contains`, same reader, same confinement | yes; 19 of leras's 43 shell verifies |
| E.3 | `contains <dir>/ <pat>` (recursive, confined) and `exists <glob>` (one `*` segment) | rule separately; both widen the read surface within the repo and each needs its own class check |

This task carries `answers: leras#le-yppwtpq`; the ask closes on a dated ANSWERED marker in
leras's file, written when the ruling lands — including a ruling of "won't build".

### R-F — spec traceability

| | |
|---|---|
| **R-F** | `covers:` key + hash, `cover`/`spec list`/`spec audit`, `gestalt/` into a repo, `MW-S*`→`MW-T*` · argued in `PROPOSAL-spec-traceability.md` and `DRAFT-tasks-weft.md` §3.8 · **unruled** |

| item | decision | recommendation |
|---|---|---|
| F.1 | a `covers:` frontmatter key carrying `ref` + content `sha`, projected as a `covers` table (not an `edges` row: the target is a clause, not a task) | yes, **after** the weft's WF2 first-run numbers say pointer rot is real |
| F.2 | `cover <task> <ref>` and `cover --repin` write the pin; hand-written pins are lint errors | yes with F.1 |
| F.3 | `spec list <doc>`, `spec audit <doc>`, portfolio variant | after `spec-drift` has fired on a real task at least once |
| F.4 | clause refs are `docs:` refs (`repo#path#§-anchor`, the slug-prefix rule in `src/docs.rs`); no new `{#sp-slug}` syntax in v1 | yes; the anchor rule already survives heading edits |
| F.5 | `gestalt/` becomes a registered repo or moves into the portfolio repo | the owner's, outside the tool |

---

## 4. Sequencing

### 4.1 The waves

```
Wave 0  owed defects · measurements · skill sentences · ID renumber        no ruling
   │
   ├──▶ R-A ──▶ Wave 1  views spec (fixtures, two stamps) ──▶ views registered
   │                │
   │                ├──▶ Wave 3  graph Rust pass ──▶ pulse in prime
   │                │                  │
   │                │                  └──▶ R-C ──▶ Wave 4  bands · placement · decay/WIP · lint findings · show lines
   │                │
   │                └──▶ R-D ──▶ Wave 5  stats · asks · portfolio search · reads-never-prune · skill rewrite
   │
   ├──▶ R-B ──▶ Wave 2  inbox honest · outbound asks · write flags · handoff provenance · hook injects skill
   ├──▶ R-E ──▶ Wave 2  DSL: package=/target= · lacks · dir/glob
   └──▶ R-F ──▶ Wave 6  covers: + hash ──▶ spec-drift ──▶ spec audit · prime line
                  ▲
                  └── waits on portfolio WF2's first run (pointer-rot numbers)
```

### 4.2 Why this order

- **Wave 0 first because it is owed and unblocked.** Five adopter asks sit in this repo's inbox
  (two of them below `prime`'s three-row fold today). Each is a pure fix with a test; none needs a
  ruling. The three measurements are scripts over transcripts the repo already parses. The skill
  sentences are the cheapest fix in the corpus for the behaviours in field study §2.7.
- **Views before everything that consumes them.** Prioritization rungs 1 and 3, the pulse, the
  lint findings, `show`'s lines, `stats`, and the honest inbox line all read `graph`, `facts`,
  `asks`, `mentions`, `hazard`. Landing the view layer once removes a second implementation from
  every one of those tasks.
- **The spec before the registration.** FORMAT.md is what third parties implement from; the
  binary is never the spec. The fixture is extended *before* any external reader (the weft's WF2,
  the UI's M0) ships against the single-store expectations.
- **The eruption fixes do not wait on the views.** Wave 2 is Rust over parsed tasks
  (`addressed.rs`, `prime.rs`, `set.rs`); its only gate is R-B. It can run beside Wave 1.
- **Rust passes only where a gate demands them.** `prime` and `ready` are gated at 100 ms;
  everything else reads SQL. Each Rust pass is pinned to its view by a differential test in the
  gate, so there is one definition.
- **Prioritization after the graph pass.** Bands change `ready`'s order; the order key needs
  `unlock` from the Rust pass, and the byte-identical-when-absent test needs the golden stores.
- **Verbs last among the ruled work.** `stats` is a presentation of numbers already reachable by
  `q` the day Wave 1 lands; the first reader's ordering argument (fixture before `stats`) holds.
- **Traceability last, and gated on a number.** Its cheapest falsifier is the weft's, not its own.

### 4.3 Critical path and parallelism

The longest chain is R-A → `a-views-spec` → `a-views-register` → `g-graph-pass` →
`p-prime-pulse` → `q-decay-wip`: six tasks, four of them a session each. Everything in Wave 0
and Wave 2 runs beside it. The ruling tasks are the only items where the owner is on the path;
they are filed as tasks so that `ready` and `why` say so.

### 4.4 Existing tasks this plan touches

| task | change |
|---|---|
| `mw-r6g9bhe` (asks age in the headline, seq 185) | absorbs the pulse's asks line; set its verify to `run cargo test prime_asks_age_line` and its handoff to point at the `asks` view's `unanswered`/`age_h` columns and `addressed::inbox`; it needs `a-views-register` for the union count but can land the age from `addressed.rs` alone |
| `mw-v4ej` (v1 acceptance, seq 150) | new `planned` TRACE rows from each ruling extend the `--strict` block it already carries; note it in the ruling commits, nothing else |
| `mw-cvw8` … `mw-a413` (mirror, 900–940) | untouched; C.6 asks the owner for the one-line ruling on the gate-behind-the-park contradiction |

---

## 5. The tasks

Conventions. Handles are local to the batch (`@handle` in `needs:`/`relates:`); ids are minted
at filing. Every verify is DSL and red today: `run cargo test <filter>` names a test that does
not exist; `contains` names text that is not there. Bodies are short — the detail is the plan
section above and the `docs:` refs. `seq` is proposed in wave order below `mw-r6g9bhe` (185),
gaps of 10; the owner may pull the five owed-ask fixes above it. Categories follow the store's
existing prefixes. Tasks answering an adopter ask carry `answers:`; under the current §15.12
rule that un-surfaces the ask from this repo's `prime` the moment the task is filed — the ruled
behaviour until R-B lands, and the reason those five sit at the top of the proposed order.

### 5.1 Wave 0 — owed, measured, unblocked

```markdown
---
handle: d-close-handoff
title: Make close strip a handoff block whole, and let lint --fix repair a task it cannot parse
category: core/lifecycle
answers: marasi-applied-r-and-d#ar-gfd8g38
seq: 190
verify: run cargo test close_strips_handoff_block_with_blank_line
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
---
`close`/`drop` remove the `handoff:` key but leave a block scalar's continuation lines when the
block contains an unindented blank line (hand-authored blocks, legal per SKILL.md); the file then
fails to parse and `lint --fix` refuses because the fixer parses first. Fix the scalar remover in
`src/edit.rs` to consume the whole block by indentation, and give `lint --fix` a pre-parse repair
for exactly this damage or a plain "cannot repair: <reason>" line. Reproduction is the study's §6
row; the lab's ask doc carries the incident.

---
handle: d-start-timeout
title: Give start's legacy-shell red-check the run timeout and a one-line notice
category: core/verify
relates: ["leras#le-3m0gpb1"]
seq: 200
verify: run cargo test start_redcheck_shell_timeout
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
---
`red_check` in `src/cli/transition.rs` runs an approved shell verify via `sh -c` with no timeout
and discards output; an unscoped `cargo test` compiles for minutes in silence and the agent kills
it and hand-flips `status: doing`. Route the shell path through `verify_exec::run_argv`'s timeout
(`RUN_TIMEOUT`), print `note: red-checking verify — may build` before it starts, and report a
timeout as a warning that still transitions. The leras hang report is answered from here.

---
handle: d-verify-path-missing
title: Warn verify-path-missing when a contains or grep verify names a path that does not exist
category: core/verify
answers: sazed#sa-b9y0pxe
seq: 210
verify: run cargo test lint::verify_path_missing
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
---
`doc-missing`'s twin on the field that decides closability: a live task whose `contains <path>`
or legacy `grep … <path>` names a file absent from the tree can never close and looks like
unfinished work (22 days in sazed after a doc rotation). Warning on live tasks only; `exists`
is excluded by definition (the artifact task's red state). The weft's W1 is the same check at a
ref across stores; this is the single-store form adopters asked for. The sazed ask closes on a
dated `verify-path-missing SHIPPED and running here` line in its own file.

---
handle: d-docs-crossrepo
title: "Give docs: a cross-repo form (repo#path#anchor) resolved through the registry"
category: core/format
answers: oreseur#or-j2qxf6j
seq: 220
verify: run cargo test docs_crossrepo_ref
docs:
  - FORMAT.md#§-task-file
  - docs/PROPOSAL-spec-traceability.md#§-b-a-covers-edge
---
`needs:`/`relates:` cross repos as `repo#id`; `docs:` has no such form, so every cross-repo doc
tie is a `../<repo>/…` path that `path-escape` now refuses — 66 pointers in the portfolio store
alone, plus oreseur's two. Add `repo#path[#anchor]` to FORMAT.md's link grammar, resolve it in
`src/docs.rs` through `MESHWORK_PORTFOLIO`'s `repos.toml` (unregistered repo = reported, never
read), keep refusing bare `../`, and let `lint --fix` rewrite `../<repo>/x` to `<repo>#x` when
the registry resolves the repo. The weft's resolver and `covers:` addressing both read this one
spelling.

---
handle: d-shim-session-var
title: Key the shim's author block on CLAUDE_CODE_SESSION_ID as well as the bridge variable
category: skill
answers: oreseur#or-9c7n6hp
seq: 230
verify: contains docs/meshwork/meshwork /CLAUDE_CODE_SESSION_ID/
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
---
CLI sessions export `CLAUDE_CODE_SESSION_ID` and not `CLAUDE_CODE_BRIDGE_SESSION_ID`, so the
shim falls back to `default_author` and agent work is stamped as the human in every sibling.
Update this repo's committed shim, the plugin's `references/install.md` shim text, and the
migrate ritual so adopters re-copy it; the author string keeps the `claude (<id>)` shape and the
no-`]` rule.

---
handle: m-waits
title: Measure what the owner costs the agents — add stop_reason waits to mine_sessions.py and re-cut
category: analysis
seq: 240
verify: contains docs/FIELD-STUDY-session-transcripts.md /owner-active minutes/
docs:
  - docs/ASKS-analytics-and-field-study.md#§-1-the-measurement
  - docs/ASKS-analytics-and-field-study.md#§-7-reproduction
---
The study's scoring is blind to a session that finished its turn and sat: no prompt, no call, no
error. Add `stop_reason` to the event stream in `scripts/mine_sessions.py`, implement the ASKS
§7 wait algorithm (main chain only, stop at the next assistant record, `isMeta` excluded,
queued prompts as zero waits), emit the wait table (count, p50/p75/p90/p99, total idle, share in
waits > 1 h, live-elsewhere and owner-active minutes) and turn-ends with no following prompt.
Land the table and the session-state primitive as a new §1 subsection of the field study.

---
handle: m-cost
title: Price the findings — tokens per session, per task by category, main chain vs subagents
category: analysis
seq: 250
verify: exists docs/cost-baseline.md
docs:
  - docs/ASKS-analytics-and-field-study.md#§-3-the-asks
  - docs/PROPOSAL-prioritization.md#§-7-build-ladder
---
One pass over records the scripts already parse: `message.usage` (input, output, cache read,
cache write) per session, split by `isSidechain`, attributed to tasks by the `start`/`close`
ids in the session's meshwork calls and to categories by the store. Outputs: `scripts/mine_cost.py
--check`; `docs/cost-baseline.md` — the deliverable, the setup-cost matrix's sibling, with every
denominator, which the weft's WF12 reads instead of mining twice; a priced column on the study's
Tier 1/Tier 2 findings; and the one number rung 0 owes the prioritization ladder — does cost
vary ≥ 3× across categories with n ≥ 7 — recorded in that proposal's §1 baseline table. Gated by
R-C item C.7 only for the *committed* data; the script itself is read-only and needs no ruling.

---
handle: m-envelope
title: Agree one event envelope between mine_sessions --events and the events view
category: analysis
seq: 260
verify: contains scripts/mine_sessions.py /event envelope/
docs:
  - docs/ASKS-analytics-and-field-study.md#§-3-the-asks
  - docs/PROPOSAL-analytics.md#§-4-3-clock
---
Two shapes for one idea. Make `--events` emit `{repo, gid?, kind, at, actor, note}` — the
`events` view's column subset — so a session's stream and the store's stream sit on one timeline
(what was this session doing in the ten minutes before it closed that task). Document the
envelope in the script header and as a note in FORMAT.md §Views once that section exists.

---
handle: t-renumber
title: Renumber the spec-traceability proposal's requirements from MW-S* to MW-T*
category: meta/docs
seq: 270
verify: contains docs/PROPOSAL-spec-traceability.md /MW-T1/
docs:
  - docs/PROPOSAL-spec-traceability.md#§-a-spec-clauses
  - docs/PROPOSAL-analytics.md#§-10-requirements
---
Both proposals mint `MW-S1`…; the analytics numbering is already cited by the ASKS document and
the UI proposal, the traceability numbering by the weft draft's §0 and §3.8. Renumber
traceability to `MW-T1`–`MW-T10` in its own file and update the weft draft's three citations in
the same commit.

---
handle: d-error-messages
title: Fix the five error messages that cost the most — foreign ids, inbox verbs, frontmatter-only flags, local addressed_to, dep add
category: core/cli
seq: 280
verify: run cargo test e2e::did_you_mean_inbox_verbs
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-4-discoverability-tax
  - docs/FIELD-STUDY-session-transcripts.md#§-1-3-errors
---
Five messages, each with a test: `show <foreign-id>` names the repo and prints the sibling
one-liner `(cd ../<repo> && ./docs/meshwork/meshwork show <id>)` when the registry resolves the
prefix; unknown verbs `next`/`addressed`/`inbox`/`help`/`list` point at `prime`, `ready`,
`--help` (never at `set`/`add`/`dep`); unknown `--to`/`--answers`/`--body` say
`frontmatter-only; see add --batch` instead of clap's `-- --to` tip (retired by the flags task
when R-B rules them in); a local `q` on `addressed_to = <me>` returning 0 rows prints
`note: local addressed_to is outbound; the inbox is portfolio q or prime`; `dep add A B` gets a
did-you-mean and the success line models `--needs`. No task ids in any message.

---
handle: d-lint-quiet
title: Make the lint channel readable — silence archived description-size, fold verify-shell, exempt the own-deliverable doc-missing, quiet this-clone verify edits
category: core/hygiene
seq: 290
verify: run cargo test lint::archived_description_size_silent
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
---
274 `verify-shell` lines on every run in the busiest store and `description-size` on immutable
archived tasks bury the real signals. Exclude `archive/` from `description-size`; print
`verify-shell` as one summary line (`N legacy shell verifies — lint --explain verify-shell`) with
the ids behind `--explain`; exempt `doc-missing` when the `docs:` target equals the task's own
`exists` deliverable; suppress `verify-changed-since-approval` when the current text was authored
on this clone, including by a hand-edit in an uncommitted tree (the close gate is untouched).

---
handle: d-add-validates
title: Validate edge targets and docs anchors at add, and warn on shell metacharacters in an inline body
category: core/authoring
seq: 300
verify: run cargo test add_refuses_dangling_edge
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
---
`add --from mw-mjwfxn` (typo) minted two dangling edges silently. Refuse a same-repo
`--from`/`--needs`/`--parent` target that does not exist; warn on a cross-repo target the
registry cannot resolve; warn `anchor-missing` for `--docs` at add rather than at the next lint;
warn when `--body` inline text contains a backtick or `$(` and point at `@file`/`-`. The flags
task reuses this validation for `--to`/`--answers`/`--relates`.

---
handle: d-show-archive-path
title: Print the archive path in show for terminal tasks
category: core/render
seq: 310
verify: run cargo test show_archived_file_path
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-4-discoverability-tax
---
`show` prints `file: docs/meshwork/<id>-….md` after `close` moved the file to `archive/`; the
agent fell back to `find` in a sibling store. Render the path the store actually holds.

---
handle: d-q-schema-help
title: List the table schemas in q --help and document the --json envelope shape
category: core/query
seq: 320
verify: run cargo test q_help_lists_schema
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-4-discoverability-tax
  - FORMAT.md#§-projection
---
Two sessions reverse-engineered the column list with `SELECT * FROM tasks LIMIT 2` (1.5 KB body
cells) and `pragma_table_info`. `q --help` prints every queryable table with its columns from the
same list the error path names (`tables::TABLES`, and the views once registered), and says that
`--json` nests rows under `data.rows`.

---
handle: d-verify-authoring
title: Fail malformed verifies at authoring time — add/set --verify refuse, start refuses, add --help stops saying sh -c
category: core/verify
seq: 330
verify: run cargo test add_refuses_malformed_dsl
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
---
`add --verify 'run cargo test -p leras topn'` mints; only `verify`/`close` refuse. Classify at
`add --verify`, `set --verify` and `add --batch` and refuse `Malformed` with the grammar line in
the message; make `start` refuse a verify `close` will refuse instead of warning and
transitioning; rewrite `add --help`'s `--verify` line (it still describes `sh -c`); put the
ten-line DSL grammar in `verify --help` and `close --help`.

---
handle: s-skill-a
title: Skill text the transcripts keep asking for — the four sentences, the inbox section, the DSL grammar, the authoring lead, the portfolio install ritual
category: skill
seq: 340
verify: contains .claude/skills/meshwork/SKILL.md /a store comment is not a ruling/
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-3-recommendations
  - docs/FIELD-STUDY-session-transcripts.md#§-2-7-behavioral-psychology
---
One commit, prose only, describing what exists today. The four sentences: a spoken instruction
becomes a task before the work starts, and the answer to "file it" is the id; never attribute a
request or ruling to the owner unless it is in this session's transcript — a store comment is
not a ruling; owner-scoped fields surface as a conflict, never resolved by editing the task;
memory files are yours, the store is the team's. Rewrite Sibling Stores around the inbox with
the verbatim `portfolio q` query and the sibling one-liner; the ten-line DSL grammar and that
`run` rejects flag tokens; `q --json`'s `data.rows`; lead authoring with `--body/--docs/--seq`
and `--batch --dry-run`; when hand-editing is not legal; `MESHWORK_ID_SEED`/`MESHWORK_TODAY`;
the portfolio store's shim + hook install in `references/install.md`. Skill rules: SKILL.md
stays daily-use; setup goes in the references.

---
handle: m-recut
title: Re-rank the field study's tiers with the wait and cost numbers attached
category: analysis
needs: [@m-waits, @m-cost]
seq: 350
verify: contains docs/FIELD-STUDY-session-transcripts.md /tokens per finding/
docs:
  - docs/ASKS-analytics-and-field-study.md#§-5-where-i-would-push-back
  - docs/FIELD-STUDY-session-transcripts.md#§-4-what-to-do-first
---
With a price on the 18-minute compile, the six-session repetition, the 723 hand-edits and the
61% of sessions that never load the skill, restate §3's Tier 1/Tier 2 split and §4's four
things with numbers beside them, in the study's own file. If the order changes, the seqs in
this plan's Wave 2 follow it; if it does not, say so in one line.
```

### 5.2 The rulings

```markdown
---
handle: r-analytics
title: Rule on analytics as contract — views in FORMAT.md, the pulse in prime, reads never prune, the three scope clauses, the handoff log line, the closure convention
category: meta/ruling
labels: [owner]
seq: 370
verify: contains docs/PLAN-proposals-implementation.md /R-A RULED 20[0-9]{2}-[0-9]{2}-[0-9]{2}/
docs:
  - docs/PLAN-proposals-implementation.md#§-3-rulings
  - docs/PROPOSAL-analytics.md#§-12-rulings-requested
---
Owner-gated. Six items, recommendations in the plan's R-A table. Ruling = the RULED marker on
the R-A header row in the plan, the accepted MUSTs appended to REQUIREMENTS under `MW-S*`, DESIGN
§15 gaining the decision, TRACE rows added `planned` in the same commit.

---
handle: r-session
title: Rule on the session ritual — write flags as local writes, asks visible until the answer is terminal, the full inbox, ask parity with next, outbound asks out of ready, the skill with prime
category: meta/ruling
labels: [owner]
seq: 380
verify: contains docs/PLAN-proposals-implementation.md /R-B RULED 20[0-9]{2}-[0-9]{2}-[0-9]{2}/
docs:
  - docs/PLAN-proposals-implementation.md#§-3-rulings
  - docs/FIELD-STUDY-session-transcripts.md#§-2-1-the-inbox
  - docs/DESIGN-meshwork.md#§-15-decisions
---
Owner-gated. Six items, recommendations in the plan's R-B table; B.1 and B.2 amend DESIGN
§15.12's reading, B.4 is the product-intent conflict the study names. Every eruption in field
study §2.1–§2.3 traces to one of these.

---
handle: r-rank
title: Rule on prioritization — bands, where they live, the default band, aging posture, unit value, inheritance opt-in, the token measurement, stats over lint --stats
category: meta/ruling
labels: [owner]
seq: 390
verify: contains docs/PLAN-proposals-implementation.md /R-C RULED 20[0-9]{2}-[0-9]{2}-[0-9]{2}/
docs:
  - docs/PLAN-proposals-implementation.md#§-3-rulings
  - docs/PROPOSAL-prioritization.md#§-10-rulings-requested
---
Owner-gated. Eight items, recommendations in the plan's R-C table. C.7 (transcripts as the
cost source) is one ruling with three consumers: rungs 5–6's gate, the field study's pricing,
the weft's WF12. C.6 also wants the one-line call on the mirror chain under the v1 gate.

---
handle: r-verbs
title: Rule on the §6 additions — stats, asks, portfolio search, show --comments N
category: meta/ruling
labels: [owner]
seq: 400
verify: contains docs/PLAN-proposals-implementation.md /R-D RULED 20[0-9]{2}-[0-9]{2}-[0-9]{2}/
docs:
  - docs/PLAN-proposals-implementation.md#§-3-rulings
  - docs/DESIGN-meshwork.md#§-6-cli-surface
  - docs/PROPOSAL-analytics.md#§-5-3-stats
---
Owner-gated. Four items. Each is a canned query or a flag over data the views already carry;
what is ruled is the surface, not the data. `e2e::cli_surface_frozen` is re-blessed once per
accepted verb with a reviewed diff.

---
handle: r-dsl
title: Rule on growing the verify grammar — package=/target= runner tokens, lacks, recursive contains and globs
category: meta/ruling
labels: [owner]
answers: leras#le-yppwtpq
seq: 410
verify: contains docs/PLAN-proposals-implementation.md /R-E RULED 20[0-9]{2}-[0-9]{2}-[0-9]{2}/
docs:
  - docs/PLAN-proposals-implementation.md#§-3-rulings
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
---
Owner-gated; a §12b trust-boundary decision, not a §6 one. Three items with the leras census
behind them (32 of 43 shell verifies fall into these shapes; the `-p` gap measured at 18
minutes of compile). The leras ask closes on a dated ANSWERED marker in its own file when this
rules, whichever way.

---
handle: r-spec
title: "Rule on spec traceability — the covers: key and hash, the cover and spec verbs, clause refs as docs refs, gestalt/ into a repo"
category: meta/ruling
labels: [owner]
seq: 420
verify: contains docs/PLAN-proposals-implementation.md /R-F RULED 20[0-9]{2}-[0-9]{2}-[0-9]{2}/
docs:
  - docs/PLAN-proposals-implementation.md#§-3-rulings
  - docs/PROPOSAL-spec-traceability.md#§-smallest-honest-slice
---
Owner-gated. Five items. The build behind it additionally waits on the weft's WF2 first-run
numbers (portfolio store, `po-add2zf4`): if pointer targets rarely move after assertion, exact
pins are not worth a format change.
```

### 5.3 Wave 1 — the view layer

```markdown
---
handle: a-views-spec
title: Publish FORMAT.md §Views with the multi-store conformance fixture blessed at two clock stamps
category: core/format
needs: [@r-analytics]
seq: 430
verify: 'all(contains FORMAT.md /^## Views/, exists fixtures/conformance/expected/views/pulse.json)'
docs:
  - docs/PROPOSAL-analytics.md#§-11-build-ladder
  - docs/PROPOSAL-analytics.md#§-7-budgets
  - docs/ASKS-analytics-and-field-study.md#§-3-the-asks
---
Spec first; the conformance test lands red. §Views: the thirteen names and columns; each
column's derivation in prose as the normative text (the weft's SQLite and the UI cannot run
DataFusion SQL); Appendix A as the reference implementation (`scripts/mine_views.py --sql` is
the source); the stamp guard; recursion bound 64 and the label-propagation stop; the closure
convention (done = current status, reopen ≠ closure, date-only = 00:00Z); the `sessions`/
`attention` sample-bias note (claims cover half of closed work); `clock` honouring
`MESHWORK_TODAY` in both conforming forms, loud on anything else, read-only idiom, and how it
differs from a read at a ref. Change `asks` before blessing: expose `answer_open`,
`answer_done`, `answered`, `unanswered` — state, not the suppression rule. Fixtures: the golden
store at two stamps; a two-store linked fixture (an ask A→B, an open answer, a done answer, a
cross-repo `needs`, a handoff naming a foreign id, a known survival curve with a censored tail)
with the union views blessed. README rows for `expected/views/`. Register nothing yet.

---
handle: a-views-register
title: Register clock and the thirteen views in the SQL session, list them in q's error path, read stats.window_days
category: core/query
needs: [@a-views-spec]
seq: 440
verify: 'all(run cargo test tables::views_registered, run cargo test conformance::views_golden)'
docs:
  - docs/PROPOSAL-analytics.md#§-5-1-q
  - docs/PROPOSAL-analytics.md#§-6-4-the-stats-table
  - FORMAT.md#§-config-toml
---
`clock` as a one-row MemTable; views created from the published SQL after the six tables and the
UDF, `[stats] window_days` (default 7) substituted into `p_win`. Register views only in sessions
that query them (`q`, `portfolio q`, later `stats` and the lint findings), never in the gated
`ready`/`prime` sessions — DataFusion plans a view body at registration and the cost is
unmeasured; add a perf row for `q` over `pulse` cold at 1K. The `q` error path's queryable-tables
line gains the view names. The conformance test goes green at both stamps and on the linked
fixture; `mine_views.py --check` stays as the cross-store probe.
```

### 5.4 Wave 2 — the eruptions, and the grammar

```markdown
---
handle: f-inbox-full
title: Make prime's inbox list every ask, or the count plus the oldest age plus the verb that lists them
category: capability/asks
relates: ["meshwork#mw-r6g9bhe"]
seq: 450
verify: run cargo test prime_inbox_lists_all_or_names_the_verb
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-1-the-inbox
  - docs/DESIGN-meshwork.md#§-7b-prime
---
`ADDRESSED_ROWS = 3`, oldest first, then `… and N more addressed` with no command to expand it;
no session in the corpus ever followed that line. Under the byte budget: print every inbound ask
while it fits; when it does not, print the count, the oldest age, and the exact `portfolio q`
statement (the `asks` verb once ruled). `ready`'s footnote gets the same treatment. The age
itself is `mw-r6g9bhe`.

---
handle: f-outbound-asks
title: Take outbound asks out of the sender's ready and next and list them under an asks-out line
category: capability/asks
needs: [@r-session]
seq: 460
verify: run cargo test ready_excludes_outbound_asks
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-1-the-inbox
  - docs/DESIGN-meshwork.md#§-5-canned-verbs
---
A `to:` task sits in its author's `ready` like work, so handoffs open "Nothing to build here —
this is an ask addressed to <repo>". With R-B item B.5: `ready`/`next`/`prime` exclude tasks
carrying `to:` from the worklist and render them as `asks out: N (oldest Dd)` with ids; `show`
on an ask prints `answered by <gid> (<status>)` when an answer exists. The frozen `ready` SQL in
DESIGN §5 gains the one clause and its golden is re-blessed.

---
handle: f-inbox-until-terminal
title: Keep an ask surfaced until its answering task is terminal, rendered answered-by with the answer's status
category: capability/asks
needs: [@r-session]
seq: 470
verify: run cargo test addressed_visible_until_answer_terminal
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-1-the-inbox
  - docs/PROPOSAL-analytics.md#§-4-6-asks
---
`src/addressed.rs` drops an ask when any non-dropped task carries `answers:`; an open
placeholder empties the inbox before a line of work exists. With R-B item B.2: an ask leaves the
inbox only when an answer is `done` (or the ask itself is terminal); an open answer renders
`answered-by <gid> (open)`. DESIGN §15.12 and FORMAT.md's reader semantics change in the same
commit; the `asks` view's `answer_open`/`answer_done` columns are the spec and the linked fixture
pins both states.

---
handle: f-flags-edges
title: Add --to, --answers and --relates to add and set
category: core/authoring
needs: [@r-session, @d-add-validates]
seq: 480
verify: run cargo test set_to_answers_relates_flags
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-3-the-fields
  - docs/ASKS-analytics-and-field-study.md#§-4-what-has-a-downstream
---
The two fields the cross-repo mechanism depends on have no CLI path; twelve of twelve writes in
the leras ask sessions went through `Write`, `perl -0pi` or a heredoc, one broke lint, one voided
a verify approval. With R-B item B.1: `--to <repo>` (scalar), `--answers <gid>` (scalar),
`--relates <gid>` (repeatable) on `add` and `set`, targets validated the way `d-add-validates`
does; `set` replaces `to:`/`answers:` and appends `relates:`. Retire the frontmatter-only error
text for these three; re-bless `e2e::cli_surface_frozen` with a reviewed diff.

---
handle: f-flags-set-fields
title: Add set --body, --parent, --from, and a docs replacement to set
category: core/authoring
needs: [@r-session]
seq: 490
verify: run cargo test set_body_parent_from_docs_replace
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-3-the-fields
---
Satisfying one `description-size` warning took six hand-edits; two ANSWER tasks were rewritten
whole seconds after `add`; `set --docs` appends and cannot fix a bad anchor. `set --body
"text"|@file|-` replaces the description above the tail sections; `--parent <id>` and `--from
<id>` set or replace the edge with validation; `--docs <old> <new>` replaces one link (keeping
`--docs <link>` as append). Same clone-approval rules as today; `cli_surface_frozen` re-blessed.

---
handle: f-handoff-provenance
title: Stamp a handoff with its author and age — set --handoff mints a log line and prime renders it
category: core/render
needs: [@r-analytics]
seq: 500
verify: run cargo test set_handoff_mints_log_line
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-2-the-previous
  - docs/PROPOSAL-analytics.md#§-4-7-mentions
  - FORMAT.md#§-tail-section-grammars
---
A handoff written by a previous session renders in the loudest slot on the page with no author
and no date, and reads to the next agent as an owner ruling. With R-A item A.5: `set --handoff`
appends `- <stamp> handoff by <author>` (author via the MW-K1 chain; the note form joins the log
grammar in FORMAT.md so `events` parses the actor); `prime`'s next block renders `[handoff by
<author>, Nd]` after the voice lines; a hand-written handoff with no line renders `[handoff:
unstamped]`. `moved_since_activity` in `mentions` gains an exact bound.

---
handle: f-prime-footer-skill
title: End prime with the one line that says to re-run it and to load the skill before filing
category: product/prime
seq: 510
verify: run cargo test prime_footer_names_skill
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-3-recommendations
---
The skill loaded in 39% of meshwork-using sessions; the sharpest behaviour change in the corpus
is an agent going from wrong-store filing to `add --batch` + `to:` + DSL verifies inside one
minute of loading it. One footer line inside the budget: `re-run meshwork prime when the
question changes; load the meshwork skill before filing`. No ruling; the hook change is
`f-hook-injects-skill`.

---
handle: f-hook-injects-skill
title: Have the SessionStart hook inject the skill body alongside prime
category: skill
needs: [@r-session]
seq: 520
verify: contains .claude/settings.json /skills/meshwork/SKILL.md/
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-3-recommendations
---
With R-B item B.6: the plugin's SessionStart hook and this repo's `.claude/settings.json` print
the skill body after `prime` (≈ 8 KB; the prime budget is untouched). Plugin-side change carried
through the install and migrate references so adopters pick it up on the next pin.

---
handle: f-dsl-package
title: Let run cargo test scope to a crate and a test target with dash-free package= and target= tokens
category: core/verify
needs: [@r-dsl]
seq: 530
verify: run cargo test verify_dsl::run_package_target_tokens
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
---
The runner grammar forbids a leading dash, correctly, so `-p <crate>` is inexpressible and
every `run cargo test <filter>` compiles the workspace: 18 minutes against a 5-minute timeout
in leras, 43 of 58 verifies kept on shell for this. With R-E item E.1: `package=<crate>` and
`target=<name>` (tight class, `=` already legal) are translated by meshwork into `-p`/`--test`;
no author text becomes a flag. Grammar comment in `src/verify_dsl.rs`, the `verify --help`
text, and the DSL fixture corpus.

---
handle: f-dsl-lacks
title: Add the lacks predicate — the inverse of contains, same reader, same confinement
category: core/verify
needs: [@r-dsl]
seq: 540
verify: run cargo test verify_dsl::lacks_predicate
docs:
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
---
"This text must be gone" is the most common close condition in a store full of defect fixes
(19 of leras's 43 shell verifies); `absent` is path-level. With R-E item E.2: `lacks <path>
<lit|/regex/>` passes when the file exists and does not match; a missing file fails (never
passes vacuously). Reader, class checks and confinement shared with `contains`.

---
handle: f-dsl-dir-glob
title: Let contains read a directory recursively and exists take a single-segment glob
category: core/verify
needs: [@r-dsl]
seq: 550
verify: run cargo test verify_dsl::dir_and_glob
docs:
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
---
With R-E item E.3, if ruled: `contains <dir>/ <pat>` walks the confined directory (byte cap,
no symlink following); `exists <path-with-one-*>` matches a dated artifact. Each widens the read
surface inside the repo only; each gets its own class check and its own fixture.
```

### 5.5 Wave 3 — the graph pass and the pulse

```markdown
---
handle: g-graph-pass
title: Compute the graph view's columns in Rust for the gated verbs, pinned to the SQL by a differential test; why prints lane, inherit and unlock
category: capability/rank
needs: [@a-views-register]
seq: 560
verify: 'all(run cargo test graph_rust_matches_view, run cargo test why_prints_placement)'
docs:
  - docs/PROPOSAL-analytics.md#§-4-5-graph
  - docs/PROPOSAL-prioritization.md#§-5-requirements
---
`ready` and `prime` are gated at 100 ms and from bands on will order by `unlock`; the SQL view
is the specification. One pass over live tasks and `needs` edges: `unlock`, `depth`, `inherit`
(same repo), `needs_behind`, `lane` over `needs` only, `lane_size`, `needs_open_foreign`,
`needed_by_foreign`. `e2e` differential test: `SELECT … FROM graph` on every fixture store equals
the Rust rows column by column under `MESHWORK_TODAY`. `why <id>` on a ready task prints its
lane, inherit and unlock; on an umbrella it says `hidden: N live children`. MW-R6/R7/R16.

---
handle: g-needs-behind
title: Warn needs-behind — a prerequisite placed later than something that needs it, naming both ids and both places
category: capability/rank
needs: [@g-graph-pass]
seq: 570
verify: run cargo test lint::needs_behind
docs:
  - docs/PROPOSAL-prioritization.md#§-5-requirements
  - docs/PROPOSAL-prioritization.md#§-1-what-the-stores-say
---
Nine contradictions portfolio-wide today, four of them findings a human wants (this repo's parked
mirror under the v1 gate; sazed's unranked prerequisite of a seq-72 task). Report only; the
owner's fix differs per case. Fixture: a parked prerequisite under a steered dependent. MW-R6b.

---
handle: p-prime-pulse
title: Render the repo's pulse row as five weather lines in prime, computed in Rust and equal to the pulse view
category: product/prime
needs: [@g-graph-pass]
seq: 580
verify: run cargo test e2e::prime_pulse_matches_view
docs:
  - docs/PROPOSAL-analytics.md#§-5-2-prime
  - docs/PROPOSAL-analytics.md#§-4-11-pulse
---
Flow, queue, graph, asks, friction — each ≤ 160 bytes, the block ≤ 800, every count with its
denominator, zero-lines omitted; the budget rule (also-ready shrinks to 3, then friction, then
graph, each cut loud). The asks line rides the inbox's union read and is `mw-r6g9bhe`'s
headline; the queue line's triage count uses 14 days until `q-decay-wip` makes it config. The
next block gains `cites N closed tasks (ids…)` from a mention pass over the handoff. The
differential test asserts `SELECT * FROM pulse` equals the rendered numbers on every fixture
store; goldens re-blessed with a reviewed diff; `check-perf.sh` green. MW-S6/S7.

---
handle: v-start-redcheck-native
title: Let start red-check native DSL predicates regardless of approval, and make a pre-emptive --approve echo the text
category: core/verify
seq: 590
verify: run cargo test start_redcheck_runs_native_dsl_unapproved
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
---
The cheap pre-work check is skipped for any verify another clone wrote and the expensive
post-work one is waved through: 43 of 44 closes carried `--approve` in four dogfooding sessions.
Native predicates are pure reads and already run ungated at close; run them at `start` too.
`close --approve` on a verify this clone has never seen refused prints the verify text before
proceeding.
```

### 5.6 Wave 4 — the floor, the findings, the lines

```markdown
---
handle: q-bands
title: Bands and placement — [bands] rules resolved by nearest ancestor, the composite key, placed_by in the projection
category: capability/rank
needs: [@g-graph-pass, @r-rank]
seq: 600
verify: run cargo test e2e::ready_bands_placement
docs:
  - docs/PROPOSAL-prioritization.md#§-4-the-shape
  - docs/PROPOSAL-prioritization.md#§-6-surfaces
  - FORMAT.md#§-config-toml
---
`[bands]` in `config.toml`: category-prefix → band, `default`, one-line `reason`, resolved by
whole-segment nearest ancestor (`category_matches`). `place(t)` = `seq` else band else
`default`; the key `(overlay, place, -unlock, created, id)` with `-unlock` entering only when
`[bands]` exists (MW-R2's byte-identical promise otherwise — a golden test on every fixture
store). `placed_by` (`seq` | `band:<rule>` | `floor`) in the tasks projection and `placement`
in every ordering verb's `--json` (schema bump). FORMAT.md rows. MW-R1–R3.

---
handle: q-inherit-placement
title: Opt-in inheritance as placement, the placed-by line in prime's next block
category: capability/rank
needs: [@q-bands]
seq: 610
verify: run cargo test ready_inherit_opt_in
docs:
  - docs/PROPOSAL-prioritization.md#§-5-requirements
---
`[bands] inherit = true` lets `inherit(t)` replace `place(t)` in the key when better, with
`placed_by = inherit:<dependent>`; off by default because it would have un-parked the mirror
against a ruling. `prime`'s next block prints `placed by: seq 180` / `band core/integrity=10 —
"<reason>"` / `floor (unlocks 3)`, sanitized like every text seam. MW-R6c, the prime half of
MW-R3.

---
handle: q-bands-lint
title: Warn band-dead, seq-shadowed, no-category (bands only) and sequence-stale
category: capability/rank
needs: [@q-bands]
seq: 620
verify: run cargo test lint::band_dead
docs:
  - docs/PROPOSAL-prioritization.md#§-5-requirements
---
A rule no task matches; a `seq` equal to the band it would have received; a live task with no
category once a band table makes categories load-bearing; an overlay entry blocked while a
later one is ready, or a tranche heading whose entries have all pruned away (registry-aware
pass). Warnings all. MW-R4/R5.

---
handle: q-decay-wip
title: Decay and WIP as config — triage_days and exempt_labels, the wip cap, open-decayed and wip-over-cap, the headline line
category: capability/rank
needs: [@p-prime-pulse, @r-rank]
seq: 630
verify: run cargo test lint::open_decayed
docs:
  - docs/PROPOSAL-prioritization.md#§-5-requirements
  - docs/PROPOSAL-analytics.md#§-4-10-flow
---
`[decay] triage_days` (14) and `exempt_labels` (`owner`); `[wip] cap`. The pulse's queue line
reads them; the headline's first line reads `doing 10 (cap 3 — OVER)` when capped; `lint` warns
`open-decayed` per task past the age and `wip-over-cap`. Nothing is blocked by any of it. The
hazard table is the `hazard` view; the fixture with the known survival curve pins the fit.
MW-R10, R12–R14.

---
handle: l-view-findings
title: Lint findings the views make cheap — handoff-cites-closed, implicit-edge, discovered-cycle, seq-collision, close-attempts
category: core/hygiene
needs: [@a-views-register]
seq: 640
verify: 'all(run cargo test lint::handoff_cites_closed, run cargo test lint::seq_collision)'
docs:
  - docs/PROPOSAL-analytics.md#§-5-4-lint
---
Each one query over `mentions`, `lineage`, `graph`, `facts`: 85 live handoffs naming a closed
task; 107 live→live mentions with no edge (report only; the tokenizer falsifier is in the
proposal's §13); `spawn_depth` at the bound; two live tasks on one `seq` in one repo (sazed 113);
≥ 2 failed closes on a live task. Warnings all. The lint session registers the views. MW-S9.

---
handle: l-verify-findings
title: Warn verify-self-satisfying and ruling-without-quote; refuse a --waive reason carrying a placeholder
category: core/verify
seq: 650
verify: run cargo test lint::verify_self_satisfying
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
  - docs/FIELD-STUDY-session-transcripts.md#§-2-7-behavioral-psychology
---
A `contains` whose target is the task's own file or a comment the CLI can write, without a
date-first marker, is satisfiable by the author; a body or comment carrying `owner ruled` /
`OWNER CALL` / `Owner ruling` with no quoted owner text is the authority-laundering shape. Both
are lexical heuristics and say so in `--explain`; the second is dropped if its first month is
mostly false. `close --waive` refuses a reason containing `<…>`.

---
handle: sh-show-lines
title: Add lineage and cites lines to show, from the views
category: core/render
needs: [@a-views-register]
seq: 660
verify: run cargo test show_lineage_line
docs:
  - docs/PROPOSAL-analytics.md#§-5-5-show
  - docs/PROPOSAL-analytics.md#§-4-8-lineage
---
After `commits:`: `lineage: spawned 25 (1 live, depth 4) · children 0 · mentioned by 7` and
`cites: 8 closed (…)`, both capped, both omitted when zero. `show` has no perf gate; read the
views. MW-S10's second half.
```

### 5.7 Wave 5 — verbs, prune, skill

```markdown
---
handle: st-reads-never-prune
title: Make portfolio q a pure read — only next and ready prune sequence.md
category: core/portfolio
needs: [@r-analytics]
seq: 670
verify: run cargo test portfolio_q_never_prunes
docs:
  - docs/PROPOSAL-analytics.md#§-6-3-reads-must-not-prune
  - docs/DESIGN-meshwork.md#§-9-portfolio
---
Every analysis in this body of work avoided the verb because it mutates the overlay on read; an
external reader that owns no state cannot prune the owner's overlay. With R-A item A.3: `q`
(and `stats`) never touch `sequence.md`; `next`/`ready` keep the ruled autoprune. Narrows
`mw-chcqk6g`; DESIGN §9 and §15 say so.

---
handle: st-stats
title: The stats verb — the pulse row, the weekly flow, the hazard table, spans, lanes, top-10s, the placement histogram; portfolio stats
category: capability/stats
needs: [@a-views-register, @r-verbs]
seq: 680
verify: 'all(run cargo test e2e::stats_tables, run cargo test perf::stats_1k_cold)'
docs:
  - docs/PROPOSAL-analytics.md#§-5-3-stats
  - docs/PROPOSAL-prioritization.md#§-7-build-ladder
---
With R-D item D.1: `stats [--window 7d|28d] [--json]` and `portfolio stats`, every table a
canned SELECT over a view, `--json` in the MW-C3 envelope, `check-perf.sh` gaining the ≤ 1 s
at 1K row. The placement histogram is `placed_by` once `q-bands` exists (a note until then).
`scripts/mine_rank.py` keeps only the Sidney experiment; its baseline tables come from `stats
--json`. DESIGN §6 row; `cli_surface_frozen` re-blessed.

---
handle: st-asks
title: The asks verb — inbound and outbound, unsuppressed, with answered-by state and age
category: capability/asks
needs: [@r-verbs, @f-inbox-until-terminal]
seq: 690
verify: run cargo test e2e::asks_verb
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-1-2-verbs
  - docs/PROPOSAL-analytics.md#§-4-6-asks
---
Typed as `addressed`/`inbox`/`portfolio show`/`list` 22 times and never existed. With R-D item
D.2: one canned query over `asks` at union scope (the same read the inbox performs), text and
`--json`, in and out sections, each row `gid · age · answered-by <gid> (<status>)`; the `prime`
inbox line names it as the expanding verb. `cli_surface_frozen` re-blessed.

---
handle: st-portfolio-search
title: portfolio search — the same literal substring search across every registered store
category: core/query
needs: [@r-verbs]
seq: 700
verify: run cargo test e2e::portfolio_search
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-4-discoverability-tax
---
Prior art in a sibling store is invisible to per-repo `search`; the lab replaced `prime` with a
hand-rolled cross-repo query for lack of it. With R-D item D.3: the same canned `strpos` SQL
over the union, hits grouped by repo, live before terminal, the MW-D2 cap + `--all`. Never
prunes. `cli_surface_frozen` re-blessed.

---
handle: s-skill-b
title: Skill ritual for what changed — the write flags, the DSL tokens, tiers as bands, the asks verb
category: skill
needs: [@f-flags-edges, @f-dsl-package, @q-bands]
seq: 710
verify: contains .claude/skills/meshwork/SKILL.md /express tiers as bands/
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-3-recommendations
  - docs/PROPOSAL-prioritization.md#§-7-build-ladder
---
One commit after the code it describes: "every field also has a CLI path" becomes true again
and says how; `package=`/`target=` and `lacks` in the grammar summary; *express tiers as bands,
keep `seq` for exceptions* with the migration note for stores that tiered by `seq`; the `asks`
verb replaces the `portfolio q` idiom in Sibling Stores. Skill rules as before.

---
handle: d-import-fixups
title: import keeps titles whole and multi-line verifies intact, and warns when it absorbs inter-checkbox prose
category: core/import
seq: 720
verify: run cargo test import_title_unwrapped
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
---
Every adoption session needed a manual repair pass: titles truncated across the frontmatter
wrap, verifies split into fragments, a 340-line section absorbed into one 17 KB body. Three
fixes with fixtures under `fixtures/import/`; low urgency now that the registered repos have
migrated.

---
handle: d-close-hygiene
title: close says what it did not check when the tree has uncommitted code and the verify is a run
category: core/lifecycle
seq: 730
verify: run cargo test close_uncommitted_notice
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
---
Two tasks were closed on a red gate through `tail <log> && close && commit`; the verify was a
`run` that passed while the gate had not. When the working tree carries uncommitted code and the
verify is a `run`, `close` prints one line naming the dirty paths it did not check. Advisory;
the close proceeds.
```

### 5.8 Wave 6 — spec traceability

```markdown
---
handle: t-covers
title: "The covers: key with a pinned content hash, the covers table, and the cover verb"
category: capability/spec
needs: [@r-spec, @d-docs-crossrepo]
seq: 740
verify: run cargo test e2e::cover_pins_hash
docs:
  - docs/PROPOSAL-spec-traceability.md#§-b-a-covers-edge
  - FORMAT.md#§-task-file
---
Waits additionally on the weft's WF2 first run (portfolio `po-add2zf4`'s tranche): add the
cross-repo `needs` when WF2 is minted. With R-F items F.1/F.2/F.4: `covers:` entries are
`docs:`-shaped refs plus `sha` over the anchored section (`src/docs.rs` resolves the section;
the hash is over its text), projected as a `covers` table (`gid, ref, sha, resolved`) — never an
`edges` row; `cover <task> <ref>` writes the pin, `cover --repin` re-reads; hand-written pins
without a matching hash are lint errors. FORMAT.md key + table, additive. MW-T4/T5.

---
handle: t-spec-drift
title: Warn spec-drift when a live task's covered clause no longer hashes to its pin
category: capability/spec
needs: [@t-covers]
seq: 750
verify: run cargo test lint::spec_drift
docs:
  - docs/PROPOSAL-spec-traceability.md#§-b-a-covers-edge
---
Fix is explicit re-pin, never silent. Done tasks with drifted pins are not reopened; they are
`spec audit`'s re-open candidates. The weft reads pinned edges as exact and computes the same
predicate, so the two never disagree on a pin. MW-T6.

---
handle: t-spec-audit
title: spec list and spec audit — unclaimed, orphaned, stale, re-open candidates, dangling; portfolio variant
category: capability/spec
needs: [@t-spec-drift]
seq: 760
verify: run cargo test e2e::spec_audit
docs:
  - docs/PROPOSAL-spec-traceability.md#§-c-coverage-and-rulings
---
With R-F item F.3, after `spec-drift` has fired on a real task at least once: `spec list <doc>`
enumerates a doc's anchored sections with hashes; `spec audit <doc>` answers the five questions
in one report; `portfolio spec audit` unions it. MW-J5's traceability matrix as a query.
`cli_surface_frozen` re-blessed. MW-T3/T8.

---
handle: t-spec-weather
title: Say in prime's weather how many live tasks the spec moved under
category: capability/spec
needs: [@t-spec-drift]
seq: 770
verify: run cargo test prime_spec_drift_line
docs:
  - docs/PROPOSAL-spec-traceability.md#§-c-coverage-and-rulings
---
One weather line, omitted at zero: `spec moved under 3 live tasks (ids…)`. In Rust over the
`covers` rows `prime` already parsed; byte-clamped like every line. MW-T10.
```

---

## 6. Deliberately not filed

| item | where it came from | why not now | what would change that |
|---|---|---|---|
| prioritization rung 4 (`lanes` section in prime) | prioritization MW-R15 | changes what a session sees first; ruling asked, not recommended yet | a session observed missing a lane the pulse's graph line names |
| prioritization rungs 5–7 (cost in the key, Sidney blocks, cohesion tie-break) | prioritization §7 | conditional on `m-cost` finding ≥ 3× variance, on a Sidney re-run after bands, on a setup-cost measurement | `m-cost`'s number; `mine_rank.py` re-run after `q-bands` |
| analytics rung 5 (git-joined per-task commits, "doing with zero commits since") | analytics §11 | the weft's WF3 runs the same join as an instrument first; single-store form waits on its evidence | WF3's first-run count and hand-check |
| analytics rung 6 (cached pulse) | analytics §11 | only if the Rust pulse misses the 100 ms gate | `check-perf.sh` after `p-prime-pulse` |
| `q --as-of <stamp>` flag | ASKS A1 | `MESHWORK_TODAY` already does it; a flag is a §6 change | R-D, if the owner wants the sugar |
| `no-seq` lint | field study Tier 2 #9 | contradicts the floor; the default is the fix | never |
| `move <id> --to <repo>`, `retract`/amend, a `ruling` record, `defer --until`, dictation ingest, `%SELF%`, bulk `set --seq -`, `init --alias`, a close-time successor prompt, a mechanical session-end handoff | field study Tier 4 | ideas with one or two sessions of evidence each; the `ruling-without-quote` lint is the cheap first step for the owner-attested channel | a second incident, or an adopter ask |
| an ask's `verify:` runnable by the answerer (`answer-verify:`) | field study #24a; this plan's own ruling tasks hit the shape | a format change; the dated-marker idiom carries it today | a third store inventing its own workaround |
| the owner-latency remedy (a session-state signal) | ASKS §1/§8 | not a store feature; nothing runs in the background | meshwork-ui's Rail (its M5), the harness |
| `discovered-from` targeting a clause; rulings as clauses | traceability MW-T7/T9 | doc practice over `covers:`; nothing to build until pins exist | `t-covers` |
| `gestalt/` as a repo | traceability MW-T2 | outside the tool | the owner |
| the weft tranche (WF1–WF14) | portfolio draft | lives in the portfolio store with its own apply procedure; this plan supplies what it consumes and consumes what it measures | the owner's word "dispatch" there |

---

## 7. Filing ritual

1. Read this file whole. Rulings are not needed to file — the ruling tasks *are* tasks.
2. Concatenate the eight batch blocks of §5 in order into one document (handles cross waves).
   From this repo: `./docs/meshwork/meshwork add --batch <file> --dry-run`, read the would-be
   files, then without `--dry-run`. All fifty-eight files or none.
3. `./docs/meshwork/meshwork lint` — every `docs:` anchor above was checked against the
   slug-prefix rule before this file was written; `anchor-missing` on any of them is a bug here.
4. Update `mw-r6g9bhe` per §4.4 (`set --verify`, `set --handoff`).
5. One commit for the store, by pathspec (`docs/meshwork/`), separate from any code:
   `chore(store): file the proposals plan — fifty-eight tasks in six waves`.
6. The five `answers:` edges un-surface their asks from `prime` at once (current §15.12); the
   answering tasks sit at the top of the proposed order so nothing is lost. Tell the askers by
   comment in their stores when each fix ships — the ask's own close condition names the marker.
7. When a ruling group is decided: edit its header row here, commit, `close` the ruling task
   (its verify is the marker), and the tasks behind it appear in `ready`.

---

## 8. Provenance

**Observed this session, on this machine.** All six documents read in full through the file
reader, not a summarizer. The store queried read-only per repo (`q`, `show`), never a `portfolio`
verb except one `portfolio q` restricted to the five inbound ask ids. Source read for every
claim about the tree: `prime.rs` (`ADDRESSED_ROWS`, budget, sections), `addressed.rs` (the
suppression filter), `tables.rs` (`session_for`, `TABLES`), `clock.rs` (`MESHWORK_TODAY`
verbatim), `lint.rs` (the warning inventory; no archive exclusion on `description-size`),
`docs.rs` (the slug-prefix anchor rule, used to write every `docs:` ref above), `verify_dsl.rs`
(the argument classes; `package=` is in class), `verify_exec.rs` (`RUN_TIMEOUT`),
`transition.rs` (the shell red-check), `add_batch.rs` and `parse.rs` (the batch key set;
`answers:` scalar, `relates:` list), `.claude/settings.json` (the hook), `fixtures/conformance/`
(single store, five keys), `scripts/mine_sessions.py` (no `stop_reason`, no `usage`),
`scripts/mine_views.py` and `mine_rank.py` (present, committed). The batch in §5 was dry-run
against the binary (`add --batch --dry-run`, exit 0, no warnings, fifty-eight would-be files)
and then filed for real in a throwaway store under the scratch directory: fifty-eight tasks,
thirty-nine `needs` edges all resolved, `lint` reporting no `verify-malformed`, `verify-shell`
or `unknown-key`, and `ready` listing exactly the twenty-eight ungated tasks (Wave 0 less
`m-recut`, the six rulings, and the six Wave 2–5 tasks that carry no `needs:`). The five
`answers:` and two `relates:` edges resolved as foreign-unresolved there only because the
throwaway had no registry. Every `docs:` anchor was checked against the slug-prefix rule
(fifty-five distinct refs, none missing) and every `contains`/`exists` verify was checked to be
red today.

**Read from the documents, not independently verified.** Every number quoted from the field
study, the analytics baseline, the prioritization baseline, the ASKS measurement and the weft's
§0 table.

**Argued.** §2 items 3, 7, 9, 10, 11 and 15 follow from the documents plus the source; none was
demonstrated by running code. Item 11 (view registration cost) is explicitly a risk to measure.

**Not done.** No gate run, no test run, no mining script executed, no DataFusion planning
measured, no sibling store written to, no file in this repo other than this one created.
