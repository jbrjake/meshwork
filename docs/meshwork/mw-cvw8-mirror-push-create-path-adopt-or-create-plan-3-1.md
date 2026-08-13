---
id: mw-cvw8
title: "mirror push create path: adopt-or-create (PLAN 3.1)"
status: open
category: plan/m3
needs: [mw-9zrd, mw-pvfrpd4]
verify: out=$(cargo test e2e::mirror_create 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 80
docs:
  - REQUIREMENTS-meshwork.md#§-h-github-push   # MW-H1, MW-H3
  - DESIGN-meshwork.md#§-8-github-push
created: 2026-08-05
handoff: |
  M3 opens here; M2 closed 2026-08-13 (portfolio live:
  sazed+leras+meshwork
  registered at ~/Documents/code/portfolio, remote
  jbrjake/meshwork-portfolio,
  first real edge leras#le-qfg98a0 needs sazed#sa-87jpgw8). Read DESIGN
  §8
  whole before code: branch guard already landed (mw-pvfrpd4); create path
  =
  marker search first (adopt, never duplicate), then create with labels +
  task-ID marker + backlink, relationships only where gh supports
  creation.
  Zero network (MW-J6): everything drives the stub gh in tests/bin/ —
  extend
  the stub's .calls ledger, never hit GitHub. Verify is the re-armed
  observed-pass form (mw-yj2fq9x swept the whole store 2026-08-13): the
  test
  must exist AND pass ≥1 — it correctly exits 1 today. Expect
  --approve on
  close; the sweep re-armed the approval ledger.
---

## log
- 2026-08-05 created
