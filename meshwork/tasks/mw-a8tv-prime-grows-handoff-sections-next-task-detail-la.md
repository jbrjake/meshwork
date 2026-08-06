---
id: mw-a8tv
title: "prime grows handoff sections: next-task detail + last-N done; retire hand-written HANDOFF.md"
status: open
category: product/prime
verify: cargo test e2e::prime_handoff_sections
docs:
  - DESIGN-meshwork.md#§-7-session-integration   # MW-D3 budget
  - REQUIREMENTS-meshwork.md#§-d-context-discipline   # MW-D3, MW-D5
created: 2026-08-06
---
Owner rulings 2026-08-06 (three design rounds): HANDOFF must not be
hand-written — it duplicates graph state (the 08-06 session hand-edited
docs/HANDOFF.md twice restating what tasks already record). Current
conditions is a MATERIALIZATION, never a maintained field. The one authored
piece is color commentary, and it lives ONLY on up-next tasks.
prime becomes the full handoff view inside the same 6KB budget (MW-D3):
  1 headline: counts + category rollup CAPPED at top 5 groups (group by
    first two category segments; rank groups by min seq among their open
    tasks — where the soonest work lives; rest collapses to "… +N", the
    MW-D2 cap pattern). No priority field exists: seq IS the priority
    primitive (owner ruling 08-06, round four)
  2 weather (all derived): freshest comments across the active frontier
    (ready+doing+blocked, newest first, byte-capped) + blocked-with-reasons
  3 next: top ready task — its `handoff:` commentary FIRST (the voice), then
    category, blocks-line (what it unblocks), verify, docs: refs, body head
    verbatim, last-2 comment tail (MW-K4 cap)
  4 also-ready one-liners with blocks-lines
  5 recently done: last ~5 closed (id, title, done-date from log)
New frontmatter key `handoff:` (multi-line block, DESIGN §2 edit): the
outgoing session's color commentary to the incoming one. Rewritten freely —
history belongs in comments. Hand-edit only, NO verb — §6 stays untouched.
Lint: warn when `handoff:` is present on a done task (stale voice).
Session-end ritual (CLAUDE.md + skill edit lands here): refresh `handoff:`
on whatever task is up next; comment anything history-worthy.
One view, not two — a separate handoff verb splits session-start reading;
richer prime output is not a §6 change (re-bless prime golden via --bless).
Data hygiene, same commit: HANDOFF's decisions line moves to DESIGN §15;
its "2.3 re-bless ready-alpha.json expected" note moves into mw-k7r5's body.
Landing commit also: delete docs/HANDOFF.md; drop CLAUDE.md ritual step 4's
HANDOFF clause + baseline override 2; adoption skill step 4 becomes "delete
HANDOFF.md". Blocks mw-ntt5 — the pilot installs the post-HANDOFF world.

## log
- 2026-08-06 created
