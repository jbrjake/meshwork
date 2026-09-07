---
id: mw-wx80vev
title: Agree one event envelope between mine_sessions --events and the events view
category: analysis
seq: 260
verify: contains scripts/mine_sessions.py /event envelope/
docs:
  - docs/ASKS-analytics-and-field-study.md#§-3-the-asks
  - docs/PROPOSAL-analytics.md#§-4-3-clock
status: open
created: 2026-09-07T16:32Z
---
Two shapes for one idea. Make `--events` emit `{repo, gid?, kind, at, actor, note}` — the
`events` view's column subset — so a session's stream and the store's stream sit on one timeline
(what was this session doing in the ten minutes before it closed that task). Document the
envelope in the script header and as a note in FORMAT.md §Views once that section exists.

## log
- 2026-09-07T16:32Z created
