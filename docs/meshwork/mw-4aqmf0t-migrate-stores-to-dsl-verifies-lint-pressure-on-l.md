---
id: mw-4aqmf0t
title: Migrate stores to DSL verifies + lint pressure on legacy shell
status: open
category: core/verify
needs: [mw-dthxs3q, mw-9rc4vs6]
parent: mw-6895bkg
verify: out=$(cargo test e2e::verify_migration 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 180
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline # MW-E2 sh -c today
created: 2026-08-06
handoff: |
  Parser + executor are BOTH landed (7437912, ca8accf):
  verify_dsl::classify
  → Dsl/Malformed/LegacyShell; verify_exec::execute + run_argv (5min
  wall,
  256KB cap, KEPT_ENV scrub). NOTHING is wired into close yet — close.rs
  still sh -c's everything behind the trust gate. This task's wiring:
  classify in close (and start's red-check): Dsl → verify_exec::execute,
  no gate; Malformed → refuse loudly, never run; LegacyShell →
  existing
  gate path. DESIGN FORK the body papers over: this store's dominant shape
  is the observed-pass idiom (out=$(cargo test F 2>&1) && grep 'ok\. N
  passed') BECAUSE bare cargo-test filters are vacuous on zero matches —
  DSL `run cargo test F` inherits that hole. Recommend: make the
  executor's
  cargo-test runner non-vacuous by construction (exit 0 AND /ok\. [1-9]
  [0-9]* passed/ in output), document in §12b, THEN flip the store;
  without
  it the migration downgrades every cargo verify. verify-shell lint warn
  is
  trivial post-classify, but lint.rs is at 590/750 — consider splitting
  before adding. Docs owed per body: MW-E2, DESIGN §2/§6 rows, README,
  SKILL.md; lint-broken golden will gain verify-shell rows (re-bless +
  review). e2e::verify_migration not yet written — red-check via sh -c
  at
  authoring. cargo fmt BEFORE clippy (fn-cap re-wraps; bit twice more this
  session).
---
Flip the stores and the docs: convert this repo's verifies to DSL (all
current shapes are covered by design), new lint warning verify-shell on
legacy shell verifies (pressure, not an error — the escape hatch stays,
behind the trust gate, for the genuinely unexpressible), amend MW-E2 +
DESIGN §2/§6 rows + README + skill docs, re-bless goldens. v1.x never
removes legacy shell; it just makes it loud, gated, and rare.

## log
- 2026-08-06 created
