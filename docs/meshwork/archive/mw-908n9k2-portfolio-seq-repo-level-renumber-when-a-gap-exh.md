---
id: mw-908n9k2
title: "portfolio seq: repo-level renumber when a gap exhausts (§15.2)"
status: done
category: plan/m2
verify: cargo test e2e::portfolio_seq_renumber
seq: 75
created: 2026-08-10T04:03Z
---

## log
- 2026-08-10T04:03Z created
- 2026-08-13T23:02Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-13T23:22Z doing→done — verify exit 0 @ d3f1afa+7

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] leras exhausted three seq neighborhoods within 48 hours of migration: 4/5/6 squeezed below 15, then 17 wedged between 15 and 20, and the audit block minted 84/85/86 plus 92/93 consecutively (faba7815, ea33cc32). Gaps-of-10 does not survive contact with a hot region; the renumber needs to exist before the portfolio inherits the problem.
- 2026-08-13T23:22Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] portfolio seq live (§15.2): trigger = two adjacent live weights with no integer between; action = that repo's live seq'd tasks renumber to 10,20,30… in (seq, created, id) order — the next-fallback order, so nothing observable reorders. Unseq'd and terminal tasks untouched; on-target weights not rewritten (minimal diffs); healthy repos byte-identical. e2e::portfolio_seq_renumber + portfolio_seq_terminal_and_text pin json/text modes, terminal exclusion, idempotency. Red observed (stub exit 1) before implementation; gate ALL SECTIONS PASS after.
