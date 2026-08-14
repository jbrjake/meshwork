---
id: mw-egksvhm
title: "Ride-along guard: run verifies auto-trust only store-only task provenance"
status: done
category: core/verify
parent: mw-6895bkg
verify: out=$(cargo test ride_along 2>&1) && echo "$out" | grep -qE "ok\. [1-9][0-9]* passed"
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
seq: 175
created: 2026-08-14T15:39Z
---

## log
- 2026-08-14T15:39Z created
- 2026-08-14T16:02Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T16:08Z doing→done — verify exit 0 @ 62969c2+8
- 2026-08-14T16:13Z done→open
- 2026-08-14T16:13Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T16:16Z doing→done — verify exit 0 @ d494bc9+6

## comments
- 2026-08-14T15:39Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Spec (owner ruling 2026-08-14, recorded on mw-6895bkg): given a task id, walk git log --follow over its file (root and archive/ paths). For each commit, judge the FULL delta of the merge that landed it — first-parent walk to find the landing merge; squash/fast-forward = the commit itself (git diff-tree covers it; for a true merge commit judge diff M^1..M so a PR split across inner commits is still seen whole). If any touched path falls outside docs/meshwork/, provenance is mixed → the task's run predicates go behind the trust gate; store-only history → run executes approval-free. Uncommitted task file = authored by this clone's operator → passes. Conservative by design: ANY commit touching the task file counts, even seq-only edits. Zero network, local refs only (MW-J6); degrade toward gating, never toward trust, when git is unreadable. Red-checked verify 2026-08-14 (observed exit 1).
- 2026-08-14T16:05Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] SPEC CORRECTED (owner, mid-review 2026-08-14) — supersedes the first comment's algorithm. The unit of judgment is THE MERGE, never the commit; the confirmed threat is a single PR carrying a task plus the test its run verify names, so the task self-verifies against attacker code once merged. Algorithm: commits = git log --follow --format=%H -- <task file>. Empty → Trusted (uncommitted = operator's own). For each commit C: if C is on rev-list --first-parent HEAD and is NOT a merge commit → Trusted without content judgment (direct commits are the operator's own; this repo's task+code close-commits must stay frictionless). If C IS a first-parent merge, or C is reachable only through some first-parent merge M (C ancestor of M, not of M^1; earliest such M is the landing merge) → judge git diff --name-only M^1 M — the WHOLE merge delta, so a PR splitting task and test across inner commits is still seen as one arrival. Any path outside docs/meshwork/ → RodeAlong{merge, path} → run gates like legacy shell. Git unreadable / no landing merge found → Unknown → gate (degrade toward gating, never trust). Operational requirement documented in §12b: third-party work lands as true merge commits; squash/rebase integration erases the boundary the guard reads.
- 2026-08-14T16:15Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] SEMANTICS CORRECTED AGAIN (owner, 2026-08-14, third ruling): the merge basis was meant to WIDEN judgment to the delivered group, never to exempt single commits — and no merge style may be forced on contributors. Final rule, a conservative union: (1) every commit touching the task file is judged by its OWN full delta — squash-merges are the whole PR in one linear commit and must gate, as does any single-commit task+code combo; (2) merge-landed commits are additionally judged by the whole landing-merge delta (M^1..M). Consequence accepted with eyes open: operator task+code close-commits read as mixed, so a reopened task re-running a run verify after its close-commit landed needs one --approve; store-only and uncommitted history stays frictionless. The earlier 'direct commits pass without content judgment' and 'true merge commits required' clauses are dead — §12b rewritten, tests flipped red-first (squash case observed failing before the fix).
