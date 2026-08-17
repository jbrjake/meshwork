---
id: mw-hfvtx0s
title: "Addressed tasks — to:/answers: as a read-time join across the portfolio union, no broker"
status: open
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
