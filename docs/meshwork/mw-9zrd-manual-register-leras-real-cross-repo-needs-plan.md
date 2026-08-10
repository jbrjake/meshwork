---
id: mw-9zrd
title: "Manual: register leras + real cross-repo needs (PLAN 2.6)"
status: open
category: plan/m2
needs: [mw-ncfg]
verify: grep -q '| 2.6 ✓' PLAN-meshwork-build.md
seq: 70
docs:
  - REQUIREMENTS-meshwork.md#§-4-acceptance-gate-for-v1   # clause 3
  - REQUIREMENTS-meshwork.md#§-g-portfolio
created: 2026-08-05
handoff: |
  Everything you need landed 2026-08-09/10 (2.1-2.5 + mw-17hnhzk, all
  gates green): registry override semantics, portfolio ready/next/q live
  (discovery: MESHWORK_PORTFOLIO or ~/Documents/code/portfolio),
  cross-repo needs resolve in single-repo verbs, import todo now imports
  nested checkboxes as parent: children (the pilot's data-loss bug —
  leras can migrate safely). Manual steps: create the real portfolio repo
  (~/Documents/code/portfolio, git init + private remote, repos.toml with
  sazed+leras+meshwork), run leras' import migration session (MW-J3
  ritual), add one real cross-repo needs sazed<->leras, then verify:
  portfolio ready shows it (REQUIREMENTS §4 clause 3). Gate --strict
  still fails on 10 planned TRACE rows — M3/M4 work, expected.
---

## log
- 2026-08-05 created

## comments
- 2026-08-10T04:47Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Pre-migration sweep (2026-08-10): 9 tasks re-ranked ahead of this one, chosen because the leras migration re-runs exactly the paths sazed's field evidence flagged. Bugs on the import/bulk path first: mw-0wvndqa (add --dry-run writes files, twice-observed) and mw-16pyc5g (batch accepted unknown 'from:' key, 6 of 13 tasks silently lost edges). Store-bootstrap guards that must exist BEFORE leras' store is minted: mw-a6jdf5s (alias charset — leras' alias gets chosen at init and is baked forever) and mw-mtn4hp8 (merge=union lint MUST — protects the new store's first merges). Session ergonomics the migration agent will hit: mw-drrvpsg (import provenance flooded 8 of sazed's weather lines — leras would repeat it) and mw-5hrb22q's landable half (did-you-mean errors; the alias half rides the ruling). mw-6wdpz1b (owner-requested start-gates-on-verify) ranked before the import so leras' verify-less backlog is born loud instead of rotting. Immediately ahead of this task sit the two §6 RULING items — mw-f1x71yg (set --cat/--verify/--title: sazed needed 3 python hand-edits in 65min for the exact post-import review this migration will do) and mw-rz4ey2h (@file/stdin prose: a handoff was shell-mangled via command substitution) — they need the owner's yes/no, ideally before migration day. mw-908n9k2 (portfolio seq stub) moved behind this task; it waits on a real exhausted gap.
