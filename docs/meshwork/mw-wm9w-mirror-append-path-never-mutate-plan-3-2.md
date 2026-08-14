---
id: mw-wm9w
title: mirror append path + never-mutate (PLAN 3.2)
status: open
category: plan/m3
needs: [mw-cvw8]
verify: "all(run cargo test e2e::mirror_append, run cargo test e2e::mirror_never_mutates)"
seq: 910
docs:
  - REQUIREMENTS-meshwork.md#§-h-github-push   # MW-H1, MW-H2
  - DESIGN-meshwork.md#§-8-github-push
created: 2026-08-05
---

## log
- 2026-08-05 created
