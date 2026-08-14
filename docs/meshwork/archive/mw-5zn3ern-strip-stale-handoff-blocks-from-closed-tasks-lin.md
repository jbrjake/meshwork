---
id: mw-5zn3ern
title: "Strip stale handoff: blocks from closed tasks — lint handoff-stale to zero"
status: done
category: hygiene
verify: "! (./meshwork lint 2>&1 | grep -q handoff-stale)"
seq: 120
created: 2026-08-14T13:29Z
---

## log
- 2026-08-14T13:29Z created
- 2026-08-14T14:46Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T14:54Z doing→done — verify exit 0 @ 534e8f8+11
