---
id: mw-jpbv
title: sequence.md + portfolio next ordering (PLAN 2.4)
status: done
category: plan/m2
needs: [mw-k7r5]
verify: cargo test e2e::portfolio_next_ordering
seq: 50
docs:
  - REQUIREMENTS-meshwork.md#§-g-portfolio   # MW-G4
  - DESIGN-meshwork.md#§-9-portfolio-master-sequencing
created: 2026-08-05
handoff: |
  2.3 landed: registry::{foreign_refs,resolve_foreign,quiet_load} +
  terminal-only injection via session_for(stores, foreign); why takes full
  statuses. For 2.4: parse sequence.md (fixtures/portfolio/sequence.md
  exists — repo#id bullets under tranche headings), portfolio next =
  first sequenced ready task, fallback repos.toml order then per-repo seq
  (MW-G4 total order); golden portfolio-next.txt per DESIGN §13. next/seq
  stubs live in cli/portfolio.rs run(). e2e harness pins HOME per-test
  (meshwork() helper) — registry tests set MESHWORK_PORTFOLIO
  explicitly. Red first via e2e::portfolio_next_ordering.
---

## log
- 2026-08-05 created
- 2026-08-10T03:55Z open→doing — claimed by Jon Rubin
- 2026-08-10T04:03Z doing→done — verify exit 0 @ 74117f2+7
