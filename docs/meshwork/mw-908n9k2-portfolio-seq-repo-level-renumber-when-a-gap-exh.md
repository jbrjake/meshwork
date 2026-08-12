---
id: mw-908n9k2
title: "portfolio seq: repo-level renumber when a gap exhausts (§15.2)"
status: open
category: plan/m2
verify: cargo test e2e::portfolio_seq_renumber
seq: 75
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

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] leras exhausted three seq neighborhoods within 48 hours of migration: 4/5/6 squeezed below 15, then 17 wedged between 15 and 20, and the audit block minted 84/85/86 plus 92/93 consecutively (faba7815, ea33cc32). Gaps-of-10 does not survive contact with a hot region; the renumber needs to exist before the portfolio inherits the problem.
