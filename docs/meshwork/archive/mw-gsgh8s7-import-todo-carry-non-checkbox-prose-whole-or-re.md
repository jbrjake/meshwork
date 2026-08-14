---
id: mw-gsgh8s7
title: "import todo: carry non-checkbox prose whole, or refuse loudly"
status: done
category: core/import
discovered-from: mw-mrjhwws
verify: out=$(cargo test import_prose 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 105
docs:
  - DESIGN-meshwork.md#§-10-migration
created: 2026-08-14T13:28Z
---
Column-0 prose outside any checkbox is dropped with exit 0 today; a whole
non-checkbox asks-section vanished in the leras migration and only the
human prompt caught it. Silent drops are the dangerous class — mangling
gets seen at review, dropping does not. Decide the carry: emit standalone
sections into a clearly-marked triage task, or refuse with a byte count.
Never exit 0 having dropped content.

## log
- 2026-08-14T13:28Z created
- 2026-08-14T13:50Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T13:56Z doing→done — verify exit 0 @ 76da28d+3
