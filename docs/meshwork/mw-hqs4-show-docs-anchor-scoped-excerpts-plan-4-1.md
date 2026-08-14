---
id: mw-hqs4
title: show --docs anchor-scoped excerpts (PLAN 4.1)
status: open
category: plan/m4
verify: out=$(cargo test e2e::show_docs_excerpts 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 130
docs:
  - REQUIREMENTS-meshwork.md#§-f-wiki-doc-drill-through   # MW-F1, MW-F2
  - DESIGN-meshwork.md#§-6-cli-surface   # show --docs
created: 2026-08-05
---

## log
- 2026-08-05 created

## comments
- 2026-08-14T13:48Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Dep on mw-a413 removed 2026-08-14: it encoded PLAN M3-before-M4 ordering, which the owner's M3 deferral overturned — doc drill-through has no real mirror dependency. The a413 edge moved to mw-v4ej, where it is real.
