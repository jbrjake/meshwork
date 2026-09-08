---
id: rt-lane002
title: Prerequisite placed after the task that needs it
status: open
category: engine
needs: [rt-lane003]
seq: 300
created: 2026-08-02
---
`graph.place` 300, `inherit` 10 (from rt-lane001), `needs_behind` true,
`unlock` 1, `depth` 1; with rt-lane003 (20, also behind the 10),
`pulse.needs_behind_n` is 2 for `right`.

## log
- 2026-08-02 created
