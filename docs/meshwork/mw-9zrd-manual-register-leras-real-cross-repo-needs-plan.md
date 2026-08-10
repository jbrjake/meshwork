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
