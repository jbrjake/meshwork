---
id: mw-4aqmf0t
title: Migrate stores to DSL verifies + lint pressure on legacy shell
status: done
category: core/verify
needs: [mw-dthxs3q, mw-9rc4vs6, mw-egksvhm]
parent: mw-6895bkg
verify: run cargo test e2e::verify_migration
seq: 180
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline # MW-E2 sh -c today
created: 2026-08-06
handoff: |
  Everything this wiring needs is landed: verify_dsl::classify →
  Dsl/Malformed/LegacyShell (7437912); verify_exec::execute + run_argv
  (ca8accf; 5min wall, 256KB cap, KEPT_ENV scrub); provenance::
  task_provenance → Trusted/RodeAlong/Unknown (d494bc9 + 38a71a0, final
  semantics per the THIRD 2026-08-14 ruling: union — every task-file
  commit's own delta judged (squash gates) AND landing-merge M^1..M for
  merge-landed commits; no merge style forced). close.rs still sh -c's
  everything — the wiring IS this task: native predicates →
  verify_exec
  ungated; run → task_provenance(root, task file rel) first, Trusted →
  ungated verify_exec, RodeAlong/Unknown → MW-E5 gate (refusal must name
  the arrival hash + offending path from RodeAlong); Malformed → refuse
  loudly, never run, never gate-prompt; LegacyShell → existing gate
  path.
  Start's red-check routes identically. STILL-OPEN FORK before flipping
  the store's verifies: DSL `run cargo test F` inherits zero-match-exits-0
  vacuity (the store's observed-pass idiom exists because of it) —
  recommend the executor's cargo-test runner demand /ok\. [1-9][0-9]*
  passed/ in output, documented in §12b, BEFORE migration. verify-shell
  lint warn is trivial post-classify, but lint.rs is at 590/750 —
  consider
  splitting first. Docs owed per body: MW-E2, DESIGN §2/§6 rows, README,
  SKILL.md; lint-broken golden gains verify-shell rows (re-bless, review
  the diff). e2e::verify_migration not yet written — red-check via sh -c
  at authoring. cargo fmt BEFORE clippy (fn-cap re-wraps; bit three
  times).
---
Flip the stores and the docs: convert this repo's verifies to DSL (all
current shapes are covered by design), new lint warning verify-shell on
legacy shell verifies (pressure, not an error — the escape hatch stays,
behind the trust gate, for the genuinely unexpressible), amend MW-E2 +
DESIGN §2/§6 rows + README + skill docs, re-bless goldens. v1.x never
removes legacy shell; it just makes it loud, gated, and rare.

## log
- 2026-08-06 created
- 2026-08-14T18:33Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T19:04Z doing→done — verify exit 0 @ 6ecbd3f+1
