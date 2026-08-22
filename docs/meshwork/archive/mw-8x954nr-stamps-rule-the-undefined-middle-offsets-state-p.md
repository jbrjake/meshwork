---
id: mw-8x954nr
title: "Stamps: rule the undefined middle (offsets) + state prefix-ordering"
category: core/format
verify: run cargo test format::stamp_ordering
docs:
  - FORMAT.md#task-file
status: done
created: 2026-08-09T23:17Z
seq: 100
---
Review finding (2026-08-09). Minted stamps are `YYYY-MM-DDTHH:MMZ`,
date-only is legal forever, and hand-editing is expected — so someone
will eventually write `2026-08-06T21:47-04:00` and the spec says
nothing. Rule it: offset stamps are either nonconforming (warn, treat
as opaque text) or accepted — but say which, because "stamps sort
lexicographically" silently breaks under mixed offsets.

Also state the prefix-ordering property explicitly, since "last
activity = max stamp" leans on it: date-only sorts before any minute
stamp of the same day. That is the right answer and currently only
implied.

## log
- 2026-08-09T23:17Z created
- 2026-08-22T00:38Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-22T00:40Z doing→done — verify exit 0 @ bee4b6d+2
