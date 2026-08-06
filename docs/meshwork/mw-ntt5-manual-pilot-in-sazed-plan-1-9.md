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
needs: [mw-der3, mw-a8tv, mw-0pj8qgv, mw-zp1h12d, mw-n6nvzpa, mw-3wnhhvp]
handoff: |
  v0.1.4 is TAGGED (74d65e2) but NOT RELEASED — the pilot now needs
  mw-0pj8qgv, whose blocked-reason carries the exact unblock commands.
  Everything the pilot wants is in that tag: 7-char ids, docs/meshwork
  flat store with auto-archive, add --seq/--docs, set verb. Once the
  release ships: pin v0.1.4, init, import TODO.md and READ the diff, wire
  the SessionStart hook, delete TODO.md + check-todo.sh + HANDOFF.md in
  ONE commit. Two real work sessions are the acceptance evidence;
  checklist = REQUIREMENTS §4 clauses 1+5 in the sazed commit message.
---

## log
- 2026-08-05 created

## comments
- 2026-08-06 [claude] Release state 2026-08-06 evening: v0.1.4 is tagged at 74d65e2 (7-char ids + docs/meshwork flat store with auto-archive + add --seq/--docs + set verb — everything the pilot needs) but UNRELEASED. GitHub Actions major outage all afternoon: webhook triggers throttled, so tag pushes for v0.1.3 (twice) and v0.1.4 created no workflow runs; v0.1.3 was superseded before ever releasing, its tag is history. workflow_dispatch was added to release.yml (040768a) but the dispatch API also refused — workflow-definition indexing rides the same throttled pipeline. When Actions recovers: 'gh workflow run release --ref v0.1.4' (or delete+re-push the tag), verify BOTH assets download and the binary reports 0.1.4, then this pilot pins v0.1.4. README on main is already written against v0.1.4.
