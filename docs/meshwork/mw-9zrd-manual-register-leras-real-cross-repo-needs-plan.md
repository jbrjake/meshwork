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
  All 15 sweep prerequisites landed 2026-08-10 (HEAD, gates green):
  import/bulk fixes (--dry-run, unknown-key rejection + from: alias),
  store guards (alias charset, merge=union lint MUST + --fix), weather
  de-noising, did-you-mean + --category/--doc aliases (ruled),
  needs-verify start gate, set --cat/--verify/--title, @file/stdin prose,
  ./meshwork shim + automatic session authors, authoring doctrine +
  tail-section rule in the skill.
  
  BLOCKER FIRST: cut and publish a release (tag + binary + skill
  artifacts) — every fix above is unreleased and leras pins a release,
  never HEAD. Remember: re-pushing a tag flips its release to draft;
  undraft after.
  
  Then this task's manual list, owner present:
  1. ~/Documents/code/portfolio — git init, private remote, repos.toml
  naming sazed + leras + meshwork.
  2. leras migration per references/adopt.md: pin .meshwork-version to the
  new release, install binary + skill, drop the committed ./meshwork shim,
  init (the alias is baked forever at init — [a-z0-9]+ only, lint
  enforces), import todo, review EVERY generated file (imported doing
  tasks with no verify are born loud under the start gate), retire
  TODO/HANDOFF in the same commit.
  3. One real cross-repo needs between sazed and leras.
  4. From meshwork: portfolio ready must show the cross-repo edge
  (REQUIREMENTS §4 clause 3). Then flip PLAN 2.6 to ✓ and advance the
  Position line in the same commit as the close.
---

## log
- 2026-08-05 created

## comments
- 2026-08-10T04:47Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Pre-migration sweep (2026-08-10): 9 tasks re-ranked ahead of this one, chosen because the leras migration re-runs exactly the paths sazed's field evidence flagged. Bugs on the import/bulk path first: mw-0wvndqa (add --dry-run writes files, twice-observed) and mw-16pyc5g (batch accepted unknown 'from:' key, 6 of 13 tasks silently lost edges). Store-bootstrap guards that must exist BEFORE leras' store is minted: mw-a6jdf5s (alias charset — leras' alias gets chosen at init and is baked forever) and mw-mtn4hp8 (merge=union lint MUST — protects the new store's first merges). Session ergonomics the migration agent will hit: mw-drrvpsg (import provenance flooded 8 of sazed's weather lines — leras would repeat it) and mw-5hrb22q's landable half (did-you-mean errors; the alias half rides the ruling). mw-6wdpz1b (owner-requested start-gates-on-verify) ranked before the import so leras' verify-less backlog is born loud instead of rotting. Immediately ahead of this task sit the two §6 RULING items — mw-f1x71yg (set --cat/--verify/--title: sazed needed 3 python hand-edits in 65min for the exact post-import review this migration will do) and mw-rz4ey2h (@file/stdin prose: a handoff was shell-mangled via command substitution) — they need the owner's yes/no, ideally before migration day. mw-908n9k2 (portfolio seq stub) moved behind this task; it waits on a real exhausted gap.
- 2026-08-10T04:51Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Sweep amendment (owner direction, 2026-08-10): the four skill tasks are leras-relevant and now rank here too, ordered by dependency — mw-we7g0k3 (./meshwork shim: the adopt convention leras onboarding runs, and the landing spot for the next one), mw-b9d4qpr (automatic session-author for agent comments — must precede leras sessions; after M3 mirroring a mis-attributed comment is append-only, unfixable), mw-1dkhj8v (authoring doctrine: imperative titles, red verifies, console mirroring — guidance for this migration's own review), mw-2hcrzvz (tail-section rule + batch-for-bodies — in SKILL.md before adoption so leras gets it from day one). Owner also ruled the §6 unfreeze for mw-f1x71yg + mw-rz4ey2h, so those are code items now.
- 2026-08-10T16:14Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] v0.2.0 cut and published 2026-08-10: tag at 775a7fd, gates green, 4 platform binaries + skill tarball up, narrative notes applied from RELEASE-NOTES-v0.2.0.md. Version bump required re-blessing 6 goldens (JSON envelope embeds the crate version; diff was version-only). Release blocker cleared — remaining work is this task's manual list, owner present.
- 2026-08-10T16:33Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Pre-migration amendment (owner direction, 2026-08-10, README portfolio review): four more tasks ranked ahead — mw-bds8yq5 (seq 30, sazed to v0.2.0 so both live stores union on one engine), mw-2nmsys2 (40, dangling sequence.md entries become a lint finding), mw-chcqk6g (50, prune satisfied sequence.md entries), mw-kkvs8zq (60, drop warns on inbound cross-repo needs). Rationale unchanged from the sweep: the migration turns sequence.md and cross-repo edges into real hand-maintained state, so the guards exist before the state does. The README tasks from the same review (mw-h2qdr6q, mw-qe5y2fc, mw-5fekg2q) stay after — none block the migration.
