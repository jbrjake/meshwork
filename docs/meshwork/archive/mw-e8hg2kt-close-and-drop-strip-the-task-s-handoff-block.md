---
id: mw-e8hg2kt
title: close and drop strip the task's handoff block
status: done
category: core/lifecycle
verify: run cargo test strips_handoff
relates:
  - mw-4jgrjar
created: 2026-08-12T20:48Z
seq: 70
---
A task that closes while carrying `handoff:` leaves a lint warning
(handoff-stale) that `--fix` does not repair, on a file that has just
auto-archived — so the sanctioned fix is hand-editing an archived file,
exactly what the skill forbids. Observed end-to-end in leras (4e5b1f04
13:57: close → warning → hand-edit of docs/meshwork/archive/…), and
this store carries two live instances right now (mw-ncfg, mw-ntt5).
The voice belongs to whatever is up next; the terminal transition is
the natural place to drop it (into the log if it must survive).

## log
- 2026-08-12T20:48Z created
- 2026-08-22T00:16Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-22T00:28Z doing→done — verify exit 0 @ 4f402d9+2
