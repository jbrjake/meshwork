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
handoff: |
  Ruled (R-A item A.3, MW-S14): q and portfolio q are pure reads;
  ready/next/seq keep the autoprune. The seam: src/cli/portfolio.rs
  load_portfolio() always calls registry_hygiene::autoprune_sequence —
  give it a prune: bool, pass false from q() (line ~345) and true from
  ready/next/seq. Keep the JSON 'pruned' key on q (always []) for MW-C3
  shape stability and say so in DESIGN §6's portfolio row (line ~136) and
  §9. Test: e2e::portfolio_q_never_prunes in tests/suite/e2e_portfolio.rs
  — copy portfolio_sequence_prune's fixture (a satisfied beta#bz-c0r3
  entry), run portfolio q '--json' over it, assert sequence.md is
  byte-identical, stderr carries no 'pruned', data.pruned == []; then
  portfolio ready still prunes. Flip TRACE MW-S14 to done with that test.
  After it, the lane is mw-zwgp6x7 (graph columns in Rust) then mw-bwwd75h
  (pulse in prime, MW-S6/S7) — both were gated on mw-sg8phqk, now done.
---
Every analysis in this body of work avoided the verb because it mutates the overlay on read; an
external reader that owns no state cannot prune the owner's overlay. With R-A item A.3: `q`
(and `stats`) never touch `sequence.md`; `next`/`ready` keep the ruled autoprune. Narrows
`mw-chcqk6g`; DESIGN §9 and §15 say so.

## log
- 2026-09-07T16:32Z created
- 2026-09-08T15:02Z handoff by claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)
