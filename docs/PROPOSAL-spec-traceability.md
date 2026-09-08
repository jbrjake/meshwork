# PROPOSAL — spec traceability: the design corpus joins the graph

**Status: proposal, unruled.** Written 2026-08-14 out of a live exercise: an adversarial
review of the gameboy savestate demo as expressed across four repos' task stores and the
`gestalt/` design docs they sprang from. Every requirement below traces to a failure mode
that review actually hit. meshwork already proved the core idea on itself — REQUIREMENTS
gives every clause a stable `MW-*` ID and MW-J5 demands each MUST trace to a named
passing test. This proposal generalizes that discipline from *requirements → tests* to
*spec → tasks*, as graph edges instead of manual discipline.

## What the review observed (the motivating failures)

1. **Spec linkage is prose, one-directional, and unversioned.** Ten-plus tasks across
   three repos cite `../gestalt/19-usecase-savestate.md` as "seed material" in body
   text. When the ground truth changed (the demo's game version was ruled), finding the
   affected tasks was a grep across repos, and nothing would have flagged them had no
   one looked. The graph knew every task-to-task edge and none of the task-to-spec ones.
2. **`docs:` answers "where do I read", never "what does this satisfy".** MW-F3 keeps
   `docs:` links resolving, which is real; but there is no way to ask *which clauses of
   the spec have implementing tasks* or *which tasks implement nothing that exists in
   spec*. Coverage is unqueryable in a tool whose thesis is SQL over the graph.
3. **Rulings replicate as prose and drift.** One seam ruling ("`bits()` lands rows at
   the lens, never as an engine UDF") needed to appear in four places across three
   repos — and the spec doc it governs still said the opposite in its SQL sketches until
   the review reconciled them. Copy-paste is the only replication mechanism rulings
   have, so divergence is the steady state.
4. **The spec corpus itself is outside every guarantee.** `gestalt/` sits at the code
   root: no git repo, no history, no hashable identity. The most load-bearing documents
   in the lane are the only artifacts in it that meshwork cannot even point at through a
   registry, let alone detect drift in. (MW-G2 already refuses to let the *registry*
   live loose at the code root, for exactly this reason.)
5. **A task's `verify:` proves implementation, not conformance.** `run cargo test
   savestate` proves a test named savestate passes — it cannot notice that the spec
   section the task was transcribed from changed after the task closed. There is no
   red-check analog for "the spec moved".

## The shape: three additions, no new system

Same materials as everything else in meshwork: markdown, frontmatter, git, SQL. No spec
database, no new file format — spec docs stay ordinary markdown that humans read.

### A. Spec clauses become addressable (MW-T1..T3)

- **MW-T1 (MUST)** A spec doc opts in by carrying clause anchors: a stable ID on a
  heading or block, `{#sp-<slug>}` style (the same move as `TENSOON-ANCHOR` comments —
  the ID travels with the content through edits and renames, where a section number or
  line rots). Clause IDs are never renumbered or reused.
- **MW-T2 (MUST)** Spec docs live in a **registered repo** — their own, or the portfolio
  repo. An unregistered/loose spec path is lint-visible (`spec-untracked`), because an
  unversioned spec can drift with no diff to point at. (This makes `gestalt/` a repo or
  moves it into one; observation 4 is not fixable by tooling alone.)
- **MW-T3 (SHOULD)** `meshwork spec list <doc>` enumerates a doc's clause IDs with their
  content hashes — the spec-side mirror of `q` over tasks.

### B. A `covers:` edge with a pinned hash (MW-T4..T7)

- **MW-T4 (MUST)** New frontmatter edge on tasks: `covers: [repo#doc.md#sp-x, …]` — this
  task implements (part of) that clause. Cross-repo by the same `repo#` addressing as
  `needs` (MW-B3); resolution through the same registry; unresolved = reported, never
  corrupted (MW-G5's posture).
- **MW-T5 (MUST)** At link time the tool records the clause's **content hash** beside
  the edge (`covers: [{ref: …, sha: …}]`, written by `meshwork cover <task> <ref>`, not
  by hand). This is the fixture-pinning move (tensoon `docs/corpus.md`, oreseur
  `MANIFEST.toml`) applied to prose: the claim is not "I implement §2", it is "I
  implement §2 *as it read when I said so*".
- **MW-T6 (MUST)** `lint` gains `spec-drift`: a live task whose covered clause no longer
  hashes to the pinned value. Fix is explicit re-pin (`cover --repin`, the human
  re-reads the clause) — never silent. A **done** task with drifted coverage is surfaced
  by `spec audit` (below) as a re-open candidate, not auto-reopened.
- **MW-T7 (SHOULD)** `discovered-from` may target a clause ref, so work found while
  reading spec records where in the spec it came from — provenance for the moment a gap
  becomes a task, which this review did a dozen times with no way to say so.

### C. Coverage and rulings become queryable (MW-T8..T10)

- **MW-T8 (MUST)** `meshwork spec audit <doc>` answers, in one report: clauses with no
  covering task (*unclaimed*), clauses covered only by dropped/superseded tasks
  (*orphaned*), live tasks with drifted pins (*stale*), done tasks with drifted pins
  (*re-open candidates*), and `covers:` refs whose clause no longer exists (*dangling*).
  Portfolio variant unions it cross-repo. This is MW-J5's traceability matrix
  ("every MUST → a named passing test") turned from a release ritual into a query.
- **MW-T9 (SHOULD)** Rulings are clauses, not prose echoes: a ruling lands **once** as a
  small anchored block in the governing spec doc (or a dedicated `rulings.md` in the
  portfolio repo), and every task that obeys it says `covers:` (or `relates:`) to that
  one anchor. The bits() ruling would have been one clause with four inbound edges
  instead of four paraphrases; the drift *between the paraphrases* becomes structurally
  impossible, and drift between ruling and spec becomes a `spec-drift` finding.
- **MW-T10 (MAY)** `prime` names spec-drift counts in the weather line — a session
  should learn "the spec moved under 3 of your live tasks" before it starts, at the same
  moment it learns what's ready.

## What this is not

- **Not doc generation, not a DSL.** Spec docs remain prose for humans; the only added
  syntax is an anchor ID, which readers can ignore.
- **Not automatic conformance.** The hash detects *drift*, not *violation* — a human
  still judges whether the change invalidates the task. The tool's job is to make the
  question fire at the right moment, which is the part that failed silently this week.
- **Not mandatory.** Un-anchored docs and `docs:`-only tasks keep working; traceability
  is bought clause by clause, where the spec is load-bearing enough to earn it.

## Smallest honest slice

MW-T1 + MW-T4 + MW-T5 + the `spec-drift` lint (T6). That alone would have caught both
halves of this week's incident: the version ruling changing the ground truth under ten
linked tasks (drift fires on every pinned consumer), and the ruling/spec disagreement on
`bits()` (the reconciling edit to the spec doc fires drift on whichever task pinned the
old reading). Audit, ruling hygiene, and prime weather are follow-ons once the edge
exists.
