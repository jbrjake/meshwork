---
id: mw-908n9k2
title: "portfolio seq: repo-level renumber when a gap exhausts (§15.2)"
status: open
category: plan/m2
verify: cargo test e2e::portfolio_seq_renumber
seq: 65
created: 2026-08-10T04:03Z
handoff: |
  Filed at 2.4 close: portfolio seq was left an honest stub (message
  points at §15.2) because no plan item owned it. Semantics per §15.2:
  renumber a repo's seq weights to gaps of 10 in current order when a gap
  exhausts; the frozen §6 surface already has the verb (cli/portfolio.rs
  run() Seq arm). Not urgent until a real store exhausts a gap.
---

## log
- 2026-08-10T04:03Z created
