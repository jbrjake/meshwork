---
id: mw-hfvtx0s
title: "Addressed tasks — to:/answers: as a read-time join across the portfolio union, no broker"
status: done
category: store
verify: run cargo test addressed
docs:
  - ../REVIEW-fresh-eyes-2026-08-14.md
seq: 30
created: 2026-08-17T03:16Z
handoff: |
  The pain is live: leras#le-nt3zbtt is a human-broker task ("relay the
  ask #48/#49 landing answers to sazed when the owner next writes") —
  the sazed↔leras ask-relay pattern is this feature wearing a workaround
  costume. Design per Gold III: addressed tasks as a READ-TIME JOIN, no
  broker — `to: repo#id` / `answers: repo#id` frontmatter keys, surfaced
  on the addressee's side by prime/ready via the portfolio union; no new
  daemon, no mirror, no write into the other repo's store (the ride-along
  guard stays intact — the merge is the trust unit). Ship with a
  conformance test proving a `to:`-addressed task appears in the
  addressee's prime/ready view and drops out when answered. Retire
  le-nt3zbtt as the first consumer.
---

## log
- 2026-08-17T03:16Z created
- 2026-08-17T18:09Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-17T20:35Z doing→done — verify exit 0 @ 419174c+2

## comments
- 2026-08-17T20:35Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Landed 419174c: to:/answers: frontmatter keys, read-time join in src/addressed.rs (the one sanctioned full-union read inside single-repo verbs — mw-k7r5 dep discipline untouched), surfaced in ready ('addressed to this repo') and prime (block 2b, cap 3), projection = tasks.addressed_to + 'answers' edge kind. Conformance e2e::addressed_* proves surface/drop-on-answer/un-answer-on-drop/terminal-ask/SQL-visibility, and add --batch accepts both keys (answers joins EDGE_KEYS for @handle resolution). First consumer RETIRED as broker: leras#le-nt3zbtt now carries 'to: sazed' (leras commit in flight) — verified live: sazed ready via the dev binary shows 'addressed to this repo (1): leras#le-nt3zbtt'. Sazed SEES it only after re-pinning to a release with this feature; leras's pinned v0.2.1 warns unknown-key until its own re-pin (MW-A6, non-fatal, by design). Gate green pre-close.
