---
id: mw-0vw7nj0
title: "portfolio ready's order ignores the sequence.md overlay — its top row disagrees with portfolio next (observed 2026-08-21: ready led with te-f2tgymv while next was le-yhrz3kv); present ready in the total ordering"
status: done
category: cli/portfolio
labels: [bug]
verify: run cargo test portfolio_ready_total_order
created: 2026-08-21T18:21Z
seq: 30
---

## log
- 2026-08-21T18:21Z created
- 2026-08-21T19:24Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-21T19:34Z doing→done — verify exit 0 @ ce7f6d7+2

## comments
- 2026-08-21T19:34Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Fixed by refactor: next and ready share sort_total (sequence.md position, repos.toml rank, seq/created/id), so disagreement is structurally impossible. The observed te-f2tgymv/le-yhrz3kv divergence was this — ready ignored the overlay entirely.
