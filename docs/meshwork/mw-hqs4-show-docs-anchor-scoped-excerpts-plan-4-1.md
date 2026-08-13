---
id: mw-hqs4
title: show --docs anchor-scoped excerpts (PLAN 4.1)
status: open
category: plan/m4
needs: [mw-a413]
verify: out=$(cargo test e2e::show_docs_excerpts 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 130
docs:
  - REQUIREMENTS-meshwork.md#§-f-wiki-doc-drill-through   # MW-F1, MW-F2
  - DESIGN-meshwork.md#§-6-cli-surface   # show --docs
created: 2026-08-05
---

## log
- 2026-08-05 created
