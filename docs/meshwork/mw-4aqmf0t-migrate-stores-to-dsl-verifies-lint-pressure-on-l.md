---
id: mw-4aqmf0t
title: Migrate stores to DSL verifies + lint pressure on legacy shell
status: open
category: core/verify
needs: [mw-dthxs3q, mw-9rc4vs6]
parent: mw-6895bkg
verify: cargo test e2e::verify_migration
seq: 180
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline # MW-E2 sh -c today
created: 2026-08-06
---
Flip the stores and the docs: convert this repo's verifies to DSL (all
current shapes are covered by design), new lint warning verify-shell on
legacy shell verifies (pressure, not an error — the escape hatch stays,
behind the trust gate, for the genuinely unexpressible), amend MW-E2 +
DESIGN §2/§6 rows + README + skill docs, re-bless goldens. v1.x never
removes legacy shell; it just makes it loud, gated, and rare.

## log
- 2026-08-06 created
