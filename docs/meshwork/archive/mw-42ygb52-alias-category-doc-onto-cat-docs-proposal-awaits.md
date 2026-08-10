---
id: mw-42ygb52
title: "Alias --category/--doc onto --cat/--docs — proposal, awaits its own §6 nod"
category: core/lifecycle
relates: [mw-rz4ey2h]
verify: cargo test e2e::category_doc_aliases
seq: 57
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
status: done
created: 2026-08-10T14:40Z
---
Split out of mw-5hrb22q. Field evidence (3 sazed sessions): --category
rejected twice, --doc three times, each costing a whole-command retry;
clap's similarity tip names the real flag but agents' habitual
`| tail -3` truncation has eaten it in the field.

The 2026-08-10 §6 ruling covered set-fields (mw-f1x71yg) and prose
paths (mw-rz4ey2h) but explicitly NOT this alias half — "still awaits
its own nod" (ruling comment on mw-rz4ey2h). The aliases were briefly
shipped in fce5b9f's predecessor (9eccb56) under a misread of that
scope and reverted the same session once the carve-out was read
properly; the revert commit carries the working implementation, so a
nod makes this a one-commit re-land (git revert of the revert + §6
table line + surface-test update citing the nod).

OWNER RULING REQUIRED before any code. If rejected instead: keep the
did-you-mean (already sanctioned as error-text), and note the rejection
in §15 so it stops being re-proposed.

## log
- 2026-08-10T14:40Z created
- 2026-08-10T14:57Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-10T14:59Z doing→done — verify exit 0 @ 273405c+5

## comments
- 2026-08-10T14:57Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Owner ruling 2026-08-10 (in-session, via direct question): YES — --category/--doc land as hidden aliases of --cat/--docs on add and set. Re-land per this task's plan.
