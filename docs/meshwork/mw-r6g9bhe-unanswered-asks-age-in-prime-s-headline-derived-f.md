---
id: mw-r6g9bhe
title: "Unanswered asks age in prime's headline — derived from `created` plus the absence of `answers`"
category: capability/asks
labels: [asks]
seq: 185
status: open
created: 2026-08-31T18:39Z
handoff: |
  This task carries prime's asks line for the pulse. Two halves with
  different
  gates: the age — "N asks unanswered, oldest D days" — derives from
  `created`
  plus the absence of `answers` and lands from `addressed.rs` alone (the
  same
  filter `addressed::inbox` uses to build the inbox rows prime already
  renders).
  The union count across stores wants the `asks` view's state columns
  (`unanswered`, and the age column beside it), so that half waits on
  mw-p2q2fxd registering the views; mw-sg8phqk is where `asks` gains
  `answer_open`/`answer_done`/`answered`/`unanswered` as state rather than
  the
  suppression rule. Land the age first; the union count is additive.
  
  The verify names `prime_asks_age_line` — write it red before the code.
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
