---
id: mw-6895bkg
title: "Verify execution security: retire raw sh -c for untrusted stores"
status: open
category: core/verify
verify: "run cargo test verify_dsl::"
seq: 15
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline # MW-E2 sh -c today
  - REQUIREMENTS-meshwork.md#§-3-non-goals # ruling recorded here when scope moves
created: 2026-08-06
---
Owner-ruled scope extension 2026-08-06: security warrants it. Today
`close` runs `verify:` via `sh -c` (MW-E2) — a task file arriving via
merge/PR from a third party is arbitrary code execution the moment an
agent closes it. North star: most minimal pragmatic steps that keep
validation non-vacuous without arbitrary shell on untrusted input.
Endgame: DSL verifies are safe by construction and run freely; legacy
shell verifies run only behind a per-clone trust gate. Children carry
the stages; this closes when the suite covers the DSL and this store
carries no untrusted-shell path. Key prior art (researched 2026-08):
direnv allow / mise trust / Helix workspace-trust (TOFU + content
hash); Cursor GHSA-hf2x-r83r-qw5q + Flowise bypass (argv[0] allowlists
defeated via argument injection — validate full argv, never a shell).

## log
- 2026-08-06 created

## comments
- 2026-08-14T15:36Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner directive 2026-08-14: run predicates never bypass the trust gate on verify-text approval alone. Attack: a merged PR carries the task AND the test it names (or just a build.rs/proc-macro edit) — cargo compiles and executes that code before any filter applies, so argv-safe run is still ride-along RCE. Same-merge detection was considered and rejected as the defense: the attacker controls commit topology (split PRs), and §12b already rules git metadata untrusted. Ruling: only native predicates (exists/absent/contains — pure reads) are gate-free; run stays gated like legacy shell until trust covers the tree (mw-hz1ezcg tree-hash bless is the candidate mechanism). DESIGN §12b + verify_exec.rs docs amended; mw-4aqmf0t handoff corrected before any wiring happened — close still routes nothing to the DSL, so no exposure window existed.
- 2026-08-14T15:40Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] SUPERSEDES the 2026-08-14 gate-routing comment above — owner overruled the run-stays-gated half: run verifies MUST execute approval-free (frictionless test-backed verifies are the DSL's point). The invariant is the ride-along guard instead: a run verify auto-trusts iff the task's git history is store-only (docs/meshwork/-only deltas, judged over each landing merge whole) — a task that arrived alongside code gets its run gated. Split-PR residual accepted with eyes open (two operator-accepted merges instead of one); tree-hash bless (mw-hz1ezcg) demoted to belt-and-braces escalation. Guard filed as mw-egksvhm, wired as a needs of mw-4aqmf0t so the close-integration cannot land without it. DESIGN §12b + verify_exec.rs amended in the same commit.
