---
id: mw-2n6zkx9
title: Stamp a handoff with its author and age — set --handoff mints a log line and prime renders it
category: core/render
needs: [mw-w2920xb]
seq: 500
verify: run cargo test set_handoff_mints_log_line
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-2-the-previous
  - docs/PROPOSAL-analytics.md#§-4-7-mentions
  - FORMAT.md#§-tail-section-grammars
status: done
created: 2026-09-07T16:32Z
---
A handoff written by a previous session renders in the loudest slot on the page with no author
and no date, and reads to the next agent as an owner ruling. With R-A item A.5: `set --handoff`
appends `- <stamp> handoff by <author>` (author via the MW-K1 chain; the note form joins the log
grammar in FORMAT.md so `events` parses the actor); `prime`'s next block renders `[handoff by
<author>, Nd]` after the voice lines; a hand-written handoff with no line renders `[handoff:
unstamped]`. `moved_since_activity` in `mentions` gains an exact bound.

## log
- 2026-09-07T16:32Z created
- 2026-09-08T14:43Z open→doing — claimed by claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)
- 2026-09-08T15:01Z doing→done — verify exit 0 @ 122a177+18

## comments
- 2026-09-08T14:49Z [claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)] Landed: set --handoff mints '- <stamp> handoff by <author>' (author via the MW-K1 chain; 'handoff' alone when none resolves) beside the block it replaces; prime's next block renders '[handoff by <author>, Nd]' after the » lines, '[handoff: unstamped]' for a block with no line. FORMAT.md: the minted form joins the log grammar; the events view gains kind 'handoff' with the actor; mentions gains src_handoff_at and moved_since_activity uses it as the exact bound for handoff mentions. The linked fixture carries one such line and the corpus is re-blessed. e2e::set_handoff_mints_log_line; MW-S10 done.
