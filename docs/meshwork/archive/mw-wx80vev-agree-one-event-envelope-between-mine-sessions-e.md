---
id: mw-wx80vev
title: Agree one event envelope between mine_sessions --events and the events view
category: analysis
seq: 260
verify: contains scripts/mine_sessions.py /event envelope/
docs:
  - docs/ASKS-analytics-and-field-study.md#§-3-the-asks
  - docs/PROPOSAL-analytics.md#§-4-3-clock
status: done
created: 2026-09-07T16:32Z
---
Two shapes for one idea. Make `--events` emit `{repo, gid?, kind, at, actor, note}` — the
`events` view's column subset — so a session's stream and the store's stream sit on one timeline
(what was this session doing in the ten minutes before it closed that task). Document the
envelope in the script header and as a note in FORMAT.md §Views once that section exists.

## log
- 2026-09-07T16:32Z created
- 2026-09-08T13:35Z open→doing — claimed by claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)
- 2026-09-08T13:39Z doing→done — verify exit 0 @ fd2afa3+4

## comments
- 2026-09-08T13:39Z [claude (8c25d21d-f70c-4d87-a136-dac674b1ca92)] Landed: --events now emits one envelope per event — repo, gid (repo#id by prefix), kind (call | hand-edit | shell-read | prompt), at, actor (the author the shim stamps: claude (session_<bridge>) or claude (<uuid>)), note, session, detail. mine_telemetry.py reads it by kind; every corpus number it printed before still prints. Prompts are now in the stream too, so a session's timeline is complete. FORMAT.md has no §Views yet — the note is parked as a comment on mw-sg8phqk.
