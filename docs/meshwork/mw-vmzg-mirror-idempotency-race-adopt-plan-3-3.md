---
id: mw-vmzg
title: mirror idempotency + race-adopt (PLAN 3.3)
status: open
category: plan/m3
needs: [mw-wm9w]
verify: out=$(cargo test e2e::mirror_idempotent 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 100
docs:
  - REQUIREMENTS-meshwork.md#§-h-github-push   # MW-H3
  - DESIGN-meshwork.md#§-8-github-push
created: 2026-08-05
---

## log
- 2026-08-05 created
