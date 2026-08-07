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
  Owner-driven pilot (PLAN 1.9, MANUAL — no session can do this for
  you). Stale-text correction: v0.1.4 DID ship 2026-08-07; the mw-0pj8qgv
  release blocker is resolved and the chapter is closed. What changed
  since that tag: main is now 10 commits ahead — the whole
  format-hardening review closed 2026-08-07 (log grammar + log as sixth
  SQL table, MW-E5 TOFU trust gate, FORMAT.md spec, registry rename
  aliases, mirror default-branch guard, comment identity hash,
  commit-trace anchors) plus the close ritual itself changed: first close
  of a task per clone now wants close --approve (or MESHWORK_TRUST=1 on
  reviewed checkouts). A pilot pinned to v0.1.4 exercises none of that —
  if the pilot should validate the hardened format, cut a fresh tag first
  (gotcha from mw-0pj8qgv: deleting a GitHub tag drafts its release; gh
  release edit --draft=false after re-push). Filed this session, ready
  behind you: mw-bvxpeef archive compaction (owner request), mw-2pz0zqc
  path confinement, mw-8fmsws3 terminal escapes, mw-0y66mhb parse.rs
  split; mw-7cvse76 (ASCII graphs) carries its own handoff.
---

## log
- 2026-08-05 created

## comments
- 2026-08-06 [claude] Release state 2026-08-06 evening: v0.1.4 is tagged at 74d65e2 (7-char ids + docs/meshwork flat store with auto-archive + add --seq/--docs + set verb — everything the pilot needs) but UNRELEASED. GitHub Actions major outage all afternoon: webhook triggers throttled, so tag pushes for v0.1.3 (twice) and v0.1.4 created no workflow runs; v0.1.3 was superseded before ever releasing, its tag is history. workflow_dispatch was added to release.yml (040768a) but the dispatch API also refused — workflow-definition indexing rides the same throttled pipeline. When Actions recovers: 'gh workflow run release --ref v0.1.4' (or delete+re-push the tag), verify BOTH assets download and the binary reports 0.1.4, then this pilot pins v0.1.4. README on main is already written against v0.1.4.
