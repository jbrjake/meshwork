---
id: mw-6mqm4em
title: "import todo: warn when a title imports as one cryptic token"
status: done
category: core/import
discovered-from: mw-mrjhwws
verify: out=$(cargo test import_short_title 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 110
docs:
  - DESIGN-meshwork.md#§-10-migration
created: 2026-08-14T13:28Z
---
sazed imported tasks titled just R11, R8, R7 — unintelligible in every
listing three days later. The import summary should warn per single-token
title so the review pass retitles them as work orders, not codes.

## log
- 2026-08-14T13:28Z created
- 2026-08-14T14:28Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T14:37Z doing→done — verify exit 0 @ ecd5606+3
