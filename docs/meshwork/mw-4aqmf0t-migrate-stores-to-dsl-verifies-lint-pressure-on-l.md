---
id: mw-4aqmf0t
title: Migrate stores to DSL verifies + lint pressure on legacy shell
status: open
category: core/verify
needs: [mw-dthxs3q, mw-9rc4vs6, mw-egksvhm]
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
  still sh -c's everything behind the trust gate. GATE ROUTING RULED
  2026-08-14 (owner, recorded on mw-6895bkg; DESIGN §12b is normative):
  native predicates (exists/absent/contains) → ungated always. run →
  approval-FREE, but only under the ride-along guard mw-egksvhm (now a
  needs of this task): task git history must be store-only —
  docs/meshwork/
  -only deltas judged over each landing merge whole; mixed provenance →
  run gated like legacy shell; uncommitted-here task → passes. Malformed
  →
  refuse loudly, never run. LegacyShell → existing sh -c gate path.
  Build
  mw-egksvhm FIRST (spec in its comment, verify red-checked). SECOND FORK
  still open: DSL `run cargo test F` inherits zero-match-exits-0 vacuity
  (this store's observed-pass idiom exists because of it) — recommend
  the
  executor's cargo-test runner demand /ok\. [1-9][0-9]* passed/ in output,
  documented in §12b, BEFORE flipping the store's verifies. verify-shell
  lint warn is trivial post-classify, but lint.rs is at 590/750 —
  consider
  splitting first. Docs owed per body: MW-E2, DESIGN §2/§6 rows, README,
  SKILL.md; lint-broken golden will gain verify-shell rows (re-bless +
  review). e2e::verify_migration not yet written — red-check via sh -c
  at
  authoring. cargo fmt BEFORE clippy (fn-cap re-wraps; bit three times).
---
Flip the stores and the docs: convert this repo's verifies to DSL (all
current shapes are covered by design), new lint warning verify-shell on
legacy shell verifies (pressure, not an error — the escape hatch stays,
behind the trust gate, for the genuinely unexpressible), amend MW-E2 +
DESIGN §2/§6 rows + README + skill docs, re-bless goldens. v1.x never
removes legacy shell; it just makes it loud, gated, and rare.

## log
- 2026-08-06 created
