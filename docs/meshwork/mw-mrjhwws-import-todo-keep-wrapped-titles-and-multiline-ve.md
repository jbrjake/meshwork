---
id: mw-mrjhwws
title: "import todo: keep wrapped titles and multiline verifies whole"
status: open
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
