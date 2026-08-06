---
id: mw-ntt5
title: Manual pilot in sazed (PLAN 1.9)
status: open
category: plan/m1
verify: grep -q '| 1.9 ✓' PLAN-meshwork-build.md
seq: 10
docs:
  - REQUIREMENTS-meshwork.md#§-4-acceptance-gate-for-v1   # clauses 1+5
  - DESIGN-meshwork.md#§-7-session-integration
  - DESIGN-meshwork.md#§-10-migration   # MW-J3
created: 2026-08-05
needs: [mw-der3, mw-a8tv]
handoff: |
  Everything you need is live: v0.1.4 released (7-char ids, docs/meshwork
  store, add --seq/--docs, meshwork set), pinned install proven end to
  end, the skill carries the step-by-step. In sazed: pin v0.1.4, init
  (creates docs/meshwork), import TODO.md and READ the generated diff,
  wire the SessionStart hook, then delete TODO.md + check-todo.sh +
  HANDOFF.md in ONE commit — prime is the handoff (DESIGN §7b). The two
  real work sessions are the acceptance evidence and cannot be delegated;
  checklist = REQUIREMENTS §4 clauses 1+5, recorded in the sazed commit
  message.
---

## log
- 2026-08-05 created
