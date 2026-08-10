---
id: mw-9093
title: Portfolio union pipeline + portfolio q (PLAN 2.2)
status: open
category: plan/m2
needs: [mw-5ckb]
verify: cargo test e2e::portfolio_union_golden
seq: 30
docs:
  - REQUIREMENTS-meshwork.md#§-g-portfolio   # MW-G1, MW-G3
  - DESIGN-meshwork.md#§-9-portfolio-master-sequencing
created: 2026-08-05
handoff: |
  2.1 landed: registry::load now carries full override semantics —
  Registry.override_findings holds unknown-path-override/renamed-repo
  warnings + override-collision errors minted at load; expand_override
  does ~/, relative-to-portfolio, loud-on-unresolvable. Build the union on
  load()'s entries: absent path or no store at path → skip + report
  (MW-G5), never error. One code path: same Arrow tables per repo + a repo
  column, then union (DESIGN §9, MW-G1/G3). portfolio q is the frozen §6
  surface — golden test e2e::portfolio_union_golden (fixtures
  alpha/beta/gamma-absent exist under tests/canned or fixtures — check
  before generating new ones). Red first.
---

## log
- 2026-08-05 created
