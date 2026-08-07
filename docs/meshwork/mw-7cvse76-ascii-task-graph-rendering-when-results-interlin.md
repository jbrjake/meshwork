---
id: mw-7cvse76
title: ASCII task-graph rendering when results interlink
status: open
category: core/render
verify: cargo test e2e::graph_render
seq: 260
docs:
  - DESIGN-meshwork.md#§-6-cli-surface # frozen surface — flags/behavior need the ruling
  - REQUIREMENTS-meshwork.md#§-d-context-discipline # MW-D5 byte budgets bound the art
created: 2026-08-06
handoff: |
  Format-hardening review is DONE: 8 tasks closed 2026-08-07 (log grammar
  + log SQL table, MW-E5 ruling + TOFU trust gate on close, FORMAT.md
  spec, registry rename aliases, mirror branch guard, comment identity
  hash, commit trace) — you're the last big one from it. Before you
  draw: prime's 6144B budget INCLUDES your art (MW-D5; clamp_bytes in
  write.rs), tree/why/blocked goldens are byte-pinned (re-bless via
  MESHWORK_BLESS=1 + reviewed diff), and show grew a commits: tail —
  don't collide section names. Owner addition: the art must also show
  category hierarchy (cluster boxes vs depth lanes — decide at impl).
  Session note: close now trust-gates shell verifies — meshwork close
  <id> --approve on first close per clone, or MESHWORK_TRUST=1. Also ready
  behind you: archive compaction (mw-bvxpeef, owner request — bundles
  sized for git, no LFS), §12b adjacent surfaces mw-2pz0zqc/mw-8fmsws3
  (terminal-escape task matters to YOUR renderer), parse.rs split
  (mw-0y66mhb, 502 lines).
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
