---
id: lf-hand001
title: In flight — the handoff names a closed task in right
status: doing
category: seam/right
seq: 50
claimed-by: claude (session_left)
handoff: |
  Start from right#rt-done001 (it shipped the fixture) and keep
  lf-ask0001 open until right answers.
created: 2026-08-03
---
`mentions` resolves `right#rt-done001` from the handoff to a terminal task
(`ref_status` done) and `lf-ask0001` to a live one; the closed reference
makes `pulse.handoff_stale_n` 1 for `left`. The comment below names a
foreign id with a stamp, so its `first_at` is set.

## log
- 2026-08-03 created
- 2026-08-05T08:00Z open→doing — claimed by claude (session_left)

## comments
- 2026-08-06T09:00Z [Jon Rubin] Blocked on right#rt-dep0001 in practice, not in the graph.
