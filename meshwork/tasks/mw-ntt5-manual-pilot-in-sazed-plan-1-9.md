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
  Everything you need is live: v0.1.2 released (README-spec refresh,
  binary unchanged), pinned install proven end to end (mw-der3's comment
  has the evidence), the meshwork skill carries the step-by-step. In
  sazed: pin v0.1.2, init, import TODO.md and READ the generated diff,
  wire the SessionStart hook, then delete TODO.md + check-todo.sh +
  HANDOFF.md in ONE commit — prime is the handoff now (DESIGN §7b). The
  two real work sessions are the acceptance evidence and cannot be
  delegated; checklist = REQUIREMENTS §4 clauses 1+5, recorded in the
  sazed commit message. Heads-up: mw-acgp will later move stores to
  docs/meshwork/ (a git mv, nothing else).
---

## log
- 2026-08-05 created
