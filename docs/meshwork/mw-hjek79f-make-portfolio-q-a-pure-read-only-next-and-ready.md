---
id: mw-hjek79f
title: Make portfolio q a pure read — only next and ready prune sequence.md
category: core/portfolio
needs: [mw-w2920xb]
seq: 670
verify: run cargo test portfolio_q_never_prunes
docs:
  - docs/PROPOSAL-analytics.md#§-6-3-reads-must-not-prune
  - docs/DESIGN-meshwork.md#§-9-portfolio
status: open
created: 2026-09-07T16:32Z
---
Every analysis in this body of work avoided the verb because it mutates the overlay on read; an
external reader that owns no state cannot prune the owner's overlay. With R-A item A.3: `q`
(and `stats`) never touch `sequence.md`; `next`/`ready` keep the ruled autoprune. Narrows
`mw-chcqk6g`; DESIGN §9 and §15 say so.

## log
- 2026-09-07T16:32Z created
