---
id: mw-mrjhwws
title: "import todo: keep wrapped titles and multiline verifies whole"
status: done
category: core/import
discovered-from: mw-9zrd
relates: [mw-17hnhzk]
verify: out=$(cargo test import_wrapped 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 100
docs:
  - DESIGN-meshwork.md#§-10-migration
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-10T22:22Z
---
leras import (2026-08-10): 12 titles truncated at hard-wrapped line breaks
("Spillway Phase-3 — the ENGINE LANDED 2026-08-03 late (owner-") and 8
multiline verifies cut at the first line. Markdown continuation lines
indented under a checkbox item belong to that item and must be joined before
field extraction. Fixture: a wrapped-checkbox TODO corpus. Sibling of the
nested-checkbox fix (mw-17hnhzk), same pipeline stage.

## log
- 2026-08-10T22:22Z created
- 2026-08-14T13:12Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T13:25Z doing→done — verify exit 0 @ b2c1e6c+4

## comments
- 2026-08-12T20:50Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Both migrations widen this task's damage list beyond wrapped titles/verifies: (1) single-token titles — sazed shipped tasks titled just R11, R8, R7, unintelligible in every listing 3 days later; (2) open checkboxes nested under done entries got folded into the bodies of CLOSED tasks — open work trapped inside archived records, hand-rescued three times (sa-e2z806m, sa-w6ffah7, sa-q36y7jb); (3) trailing non-checkbox prose silently dropped or grafted onto neighboring tasks (leras 6f063ba1: "the import had swallowed the whole non-checkbox sazed-asks section"); (4) [~] maps to doing with no claimant, seeding instant doing-rot (le-s3k2v7b). The silent drops are the dangerous class — mangling gets seen at review, dropping does not.
