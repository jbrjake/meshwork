---
id: mw-8r1a
title: Lint anchor validation warn (PLAN 4.2)
status: open
category: plan/m4
needs: [mw-hqs4]
verify: out=$(cargo test lint::anchor_missing_warn 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 140
docs:
  - REQUIREMENTS-meshwork.md#§-f-wiki-doc-drill-through   # MW-F3
  - DESIGN-meshwork.md#§-6-cli-surface   # lint anchors
created: 2026-08-05
---

## log
- 2026-08-05 created
