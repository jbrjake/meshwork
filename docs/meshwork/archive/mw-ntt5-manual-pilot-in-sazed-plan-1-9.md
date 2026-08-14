---
id: mw-ntt5
title: Manual pilot in sazed (PLAN 1.9)
status: done
category: plan/m1
verify: grep -q '| 1.9 ✓' PLAN-meshwork-build.md
seq: 10
docs:
  - REQUIREMENTS-meshwork.md#§-4-acceptance-gate-for-v1   # clauses 1+5
  - DESIGN-meshwork.md#§-7-session-integration
  - DESIGN-meshwork.md#§-10-migration   # MW-J3
created: 2026-08-05
needs: [mw-der3, mw-a8tv, mw-0pj8qgv, mw-zp1h12d, mw-n6nvzpa, mw-3wnhhvp]
---

## log
- 2026-08-05 created
- 2026-08-07T13:50Z open→done — verify exit 0 @ 1d97825+15

## comments
- 2026-08-06 [claude] Release state 2026-08-06 evening: v0.1.4 is tagged at 74d65e2 (7-char ids + docs/meshwork flat store with auto-archive + add --seq/--docs + set verb — everything the pilot needs) but UNRELEASED. GitHub Actions major outage all afternoon: webhook triggers throttled, so tag pushes for v0.1.3 (twice) and v0.1.4 created no workflow runs; v0.1.3 was superseded before ever releasing, its tag is history. workflow_dispatch was added to release.yml (040768a) but the dispatch API also refused — workflow-definition indexing rides the same throttled pipeline. When Actions recovers: 'gh workflow run release --ref v0.1.4' (or delete+re-push the tag), verify BOTH assets download and the binary reports 0.1.4, then this pilot pins v0.1.4. README on main is already written against v0.1.4.
- 2026-08-07T13:48Z [claude] PILOT VERDICT (2026-08-07, both sazed sessions reviewed from transcripts). PASSED its clauses: prime (2,968B, ~700 tok) replaced 116,119B of TODO+HANDOFF session-start reading, 39x; hook injected it in 196ms; the work session oriented from prime alone — 4 store reads, zero doc sweeps, took the recommended task, closed it through the MW-E5 gate with zero friction (--approve on first try, pre-documented in sazed CLAUDE.md); HANDOFF.md deleted (83-file/841KB archive stopped growing), check-todo.sh never existed; checklist in sazed abe358b. FINDINGS: 10 tasks filed (mw-17hnhzk import drops nested checkboxes = worst, silent data loss; mw-16pyc5g batch from:-key trap; mw-0wvndqa --dry-run writes; mw-f1x71yg/mw-rz4ey2h/mw-5hrb22q set-fields+@file+forgiveness; mw-175bn4c red-check verify; mw-dkwf26w lint doing; mw-drrvpsg weather noise; mw-1dkhj8v skill doctrine). Cultural result: the session crisis (defect identified 3x, ranked behind measurement 3x) was RESOLVED through the store — seq 1, imperative retitle, implementation-brief handoff, vacuous verify swapped for a red ratchet — so the next session provably opens on the fix. Meshwork records faithfully; it cannot yet make a wrong ranking loud. That is mw-175bn4c + mw-1dkhj8v.
