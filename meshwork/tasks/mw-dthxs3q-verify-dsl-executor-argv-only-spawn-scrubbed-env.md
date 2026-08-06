---
id: mw-dthxs3q
title: "Verify DSL: executor — argv-only spawn, scrubbed env, timeout"
status: open
category: core/verify
needs: [mw-sascrgs]
parent: mw-6895bkg
verify: cargo test verify_dsl::exec
seq: 170
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline # MW-E2 sh -c today
created: 2026-08-06
---
Executor for the mw-sascrgs grammar: argv-array spawn only (never a
shell), env scrubbed to a pinned minimal set, cwd = repo root,
wall-clock timeout, output cap. run limited to the per-runner grammars;
DSL verifies bypass the trust gate — safe by construction is the whole
payoff. SHOULD (follow-on hardening, not a blocker): OS sandbox layer —
sandbox-exec/Seatbelt on darwin, Landlock on linux — as defense in
depth for the run predicate.

## log
- 2026-08-06 created
