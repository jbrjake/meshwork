---
id: lf-need001
title: Needs an open task in right
status: open
category: seam/right
needs: [right#rt-dep0001]
seq: 40
created: 2026-08-04
---
A cross-repo `needs` whose target is open: `graph.needs_open_foreign` 1,
`ready` false, and `pulse.blocked_foreign` counts this task for `left`
while `pulse.owed_foreign` counts the target for `right`.

## log
- 2026-08-04 created
