---
id: mw-ws31
title: mirror status drift report (PLAN 3.4)
status: open
category: plan/m3
needs: [mw-vmzg]
verify: out=$(cargo test e2e::mirror_status_reports_only 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 110
docs:
  - REQUIREMENTS-meshwork.md#§-h-github-push   # MW-H4
  - DESIGN-meshwork.md#§-8-github-push
created: 2026-08-05
---

## log
- 2026-08-05 created
