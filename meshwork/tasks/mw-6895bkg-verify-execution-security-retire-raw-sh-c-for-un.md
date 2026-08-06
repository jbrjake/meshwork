---
id: mw-6895bkg
title: "Verify execution security: retire raw sh -c for untrusted stores"
status: open
category: core/verify
verify: "cargo test verify_dsl::"
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
