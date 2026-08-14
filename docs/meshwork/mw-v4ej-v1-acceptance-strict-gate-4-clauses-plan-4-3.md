---
id: mw-v4ej
title: "v1 acceptance: strict gate + §4 clauses (PLAN 4.3)"
status: open
category: plan/m4
needs: [mw-8r1a, mw-a413]
verify: ./verify_meshwork.sh --strict
seq: 150
docs:
  - REQUIREMENTS-meshwork.md#§-4-acceptance-gate-for-v1   # all 5 clauses
  - DESIGN-meshwork.md#§-14-gate   # --strict
created: 2026-08-05
---

## log
- 2026-08-05 created

## comments
- 2026-08-14T13:48Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Gained needs: mw-a413 2026-08-14 (moved from mw-hqs4): v1 acceptance runs the strict gate, which demands TRACE fully done including the MW-H mirror rows — v1 genuinely cannot close while M3 sits deferred, and this edge keeps that visible in why/blocked instead of a red verify surprise.
