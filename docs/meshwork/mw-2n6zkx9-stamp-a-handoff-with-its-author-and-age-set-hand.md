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
status: open
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
