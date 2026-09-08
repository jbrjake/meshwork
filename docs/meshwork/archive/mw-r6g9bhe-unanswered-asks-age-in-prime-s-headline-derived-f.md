---
id: mw-r6g9bhe
title: "Unanswered asks age in prime's headline — derived from `created` plus the absence of `answers`"
category: capability/asks
labels: [asks]
seq: 185
status: done
created: 2026-08-31T18:39Z
verify: run cargo test prime_asks_age_line
---
The recorded risk fired verbatim this period: *"If B never runs meshwork, the message sits
forever and A can't tell unread from ignored."* Nine asks sat undeliverable in the applied
lab's ledger and **a person noticed, not the tool** — the 08-30 fix commit says so. The same
node that recorded the risk recorded the remedy, and this task is that remedy: derive an ask's
age from `created` plus the absence of `answers`, and surface it in `prime`'s headline — "N
asks unanswered, oldest D days" — so silence becomes a number every session sees.

This is the second consecutive atlas pass to record the risk firing with the remedy unbuilt.
The `addressed_to` ledger is now the portfolio's real cross-repo traffic (30 asks, five
stores); the tool carrying it must be the thing that reports its stalls — same principle as
"if a defect is reachable by a command, fix the command."

While in there, the cheap half of the same failure class: if an ask's addressing cannot
resolve at file time, say so loudly then, not never.

## log
- 2026-08-31T18:39Z created
- 2026-09-08T00:04Z open→done — verify exit 0 @ 644b16f+13
