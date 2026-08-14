---
id: mw-dthxs3q
title: "Verify DSL: executor — argv-only spawn, scrubbed env, timeout"
status: done
category: core/verify
needs: [mw-sascrgs]
parent: mw-6895bkg
verify: out=$(cargo test verify_dsl::exec 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
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
- 2026-08-14T15:18Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T15:23Z doing→done — verify exit 0 @ 7437912+8
