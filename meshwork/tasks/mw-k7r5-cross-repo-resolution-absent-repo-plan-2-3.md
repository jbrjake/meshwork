---
id: mw-k7r5
title: Cross-repo resolution + absent repo (PLAN 2.3)
status: open
category: plan/m2
needs: [mw-9093]
verify: cargo test -- e2e::crossrepo_resolution e2e::absent_repo
seq: 40
docs:
  - REQUIREMENTS-meshwork.md#§-b-graph-model   # MW-B3
  - REQUIREMENTS-meshwork.md#§-g-portfolio   # MW-G5
  - DESIGN-meshwork.md#§-5-canned-verbs-frozen-sql
created: 2026-08-05
---
Expected diff when this lands: re-bless ready-alpha.json — az-x9b2 becomes
ready once cross-repo needs resolve. (Moved from hand-written HANDOFF at its
retirement, 2026-08-06.)

## log
- 2026-08-05 created
