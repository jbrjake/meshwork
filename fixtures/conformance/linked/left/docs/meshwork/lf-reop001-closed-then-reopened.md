---
id: lf-reop001
title: Closed, then reopened — not a closure
status: open
category: cohort
seq: 60
created: 2026-07-28
---
The closure convention: `done` means current status. This task's log
carries a `→done` line, but it is open now, so `facts.done_at`,
`terminal_at`, `cycle_h` and `service_h` are NULL, `reopens` is 1, and
`pulse.done_w` does not count it even though the close fell in the window.

## log
- 2026-07-28 created
- 2026-08-05T09:00Z open→doing — claimed by spec
- 2026-08-05T10:00Z doing→done — verify exit 0 @ abc1234
- 2026-08-06T11:00Z done→open — the fix regressed
