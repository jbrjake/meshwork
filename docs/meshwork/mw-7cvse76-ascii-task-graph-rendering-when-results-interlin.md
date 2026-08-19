---
id: mw-7cvse76
title: ASCII task-graph rendering when results interlink
status: open
category: core/render
verify: run cargo test e2e::graph_render
seq: 950
docs:
  - DESIGN-meshwork.md#§-6-cli-surface # frozen surface — flags/behavior need the ruling
  - REQUIREMENTS-meshwork.md#§-d-context-discipline # MW-D5 byte budgets bound the art
created: 2026-08-06
handoff: |
  Up next after the 2026-08-19 sweep (startup bench, gate §7 prime
  coverage, log-tail status derivation, no-verify covers doing, SIGPIPE,
  FORMAT conformance corpus — see those archive files). You're the last
  big format-hardening item. Before drawing: read your own comment thread
  — the byte-budget ruling (art counts inside prime's 6144B; auto only
  when >1 edge AND it fits; a flag forces it) and the category-hierarchy
  owner addition (cluster boxes vs depth lanes, implementer's choice).
  Still needs the owner at impl time: ONLY the flag's name and its §6
  wording — tee both up as a one-word question like the mw-hz1ezcg
  proposal. Goldens for tree/why/blocked are byte-pinned (MESHWORK_BLESS=1
  + reviewed diff); layout is hand-rolled (dep posture). Separately
  pending the owner: mw-hz1ezcg sits blocked on the §6 bulk-bless ruling
  (proposal in its comments); a nod there also unblocks mw-yyf1bab.
---
Owner-requested 2026-08-06 (scope creep acknowledged and ruled worth
it): when a result set contains >=2 tasks interlinked by needs/parent/
relates/discovered-from, render the induced subgraph as terminal art —
box-drawing DAG, topological layers, edge glyphs distinguishing hard
needs from parent nesting from soft relates; cross-repo edges (repo#id)
marked. Owner addition 2026-08-06: the art also shows CATEGORY
HIERARCHY — the slash-path tree (engine/spill etc., MW-B4/B8) groups or
labels nodes so structure reads at a glance; exact treatment (cluster
boxes vs
depth-labeled lanes) decided at implementation with the same budget
rules. Applies to ready/q/tree/why output and the prime handoff view.
Constraints that make it non-trivial: prime's 6144B cap includes the
art (MW-D5). Owner ruling 2026-08-06 (refined): automatic when ALL of
— more than 1 edge to show, the art fits, and there's room in the
remaining budget; when the budget doesn't fit it does NOT show without
a flag. The budget is never raised for art and the flat format is
never squeezed by it. Trivial subgraphs (0-1 edges) get no art
regardless. tree today is indent-only
and lies about DAGs (a task with two inbound edges prints twice) — the
graph view renders shared nodes once. Must be cycle-safe (render can't
hang on what lint hasn't caught yet), monochrome, --json output
unchanged. Ruling still open at implementation: only the flag's name
and §6 wording. Layout stays hand-rolled — a graph-layout
dependency fails the pinned-dep posture (MW-J1) for a cosmetic feature.

## log
- 2026-08-06 created
