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
  Pre-migration queue is FULLY COMPLETE and RELEASED as of 2026-08-10.
  v0.2.1 is the tag leras pins: cut at b82b4f4, gates green, 4 platform
  binaries + skill tarball up, narrative notes applied from
  RELEASE-NOTES-v0.2.1.md. It carries all four sequence-state guards
  (dangling-sequence lint, portfolio autoprune — owner ruled no flag,
  drop
  inbound cross-repo warning, plus the registry_hygiene split). The
  v0.2.0/HEAD delta concern from the previous handoff is closed.
  
  Optional, not blocking: sazed still pins v0.2.0 — fine to leave; bump
  to
  v0.2.1 whenever a sazed session is open anyway (edit .meshwork-version,
  reinstall binary + skill per the adoption skill).
  
  This task's manual list stands as written (owner present): portfolio
  repo
  init + repos.toml naming sazed + leras + meshwork, leras migration per
  references/adopt.md (pin .meshwork-version to v0.2.1), one real
  cross-repo needs, portfolio ready shows the edge, flip PLAN 2.6 ✓ +
  Position in the close commit.
---

## log
- 2026-08-05 created

## comments
- 2026-08-10T04:47Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Pre-migration sweep (2026-08-10): 9 tasks re-ranked ahead of this one, chosen because the leras migration re-runs exactly the paths sazed's field evidence flagged. Bugs on the import/bulk path first: mw-0wvndqa (add --dry-run writes files, twice-observed) and mw-16pyc5g (batch accepted unknown 'from:' key, 6 of 13 tasks silently lost edges). Store-bootstrap guards that must exist BEFORE leras' store is minted: mw-a6jdf5s (alias charset — leras' alias gets chosen at init and is baked forever) and mw-mtn4hp8 (merge=union lint MUST — protects the new store's first merges). Session ergonomics the migration agent will hit: mw-drrvpsg (import provenance flooded 8 of sazed's weather lines — leras would repeat it) and mw-5hrb22q's landable half (did-you-mean errors; the alias half rides the ruling). mw-6wdpz1b (owner-requested start-gates-on-verify) ranked before the import so leras' verify-less backlog is born loud instead of rotting. Immediately ahead of this task sit the two §6 RULING items — mw-f1x71yg (set --cat/--verify/--title: sazed needed 3 python hand-edits in 65min for the exact post-import review this migration will do) and mw-rz4ey2h (@file/stdin prose: a handoff was shell-mangled via command substitution) — they need the owner's yes/no, ideally before migration day. mw-908n9k2 (portfolio seq stub) moved behind this task; it waits on a real exhausted gap.
- 2026-08-10T04:51Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Sweep amendment (owner direction, 2026-08-10): the four skill tasks are leras-relevant and now rank here too, ordered by dependency — mw-we7g0k3 (./meshwork shim: the adopt convention leras onboarding runs, and the landing spot for the next one), mw-b9d4qpr (automatic session-author for agent comments — must precede leras sessions; after M3 mirroring a mis-attributed comment is append-only, unfixable), mw-1dkhj8v (authoring doctrine: imperative titles, red verifies, console mirroring — guidance for this migration's own review), mw-2hcrzvz (tail-section rule + batch-for-bodies — in SKILL.md before adoption so leras gets it from day one). Owner also ruled the §6 unfreeze for mw-f1x71yg + mw-rz4ey2h, so those are code items now.
- 2026-08-10T16:14Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] v0.2.0 cut and published 2026-08-10: tag at 775a7fd, gates green, 4 platform binaries + skill tarball up, narrative notes applied from RELEASE-NOTES-v0.2.0.md. Version bump required re-blessing 6 goldens (JSON envelope embeds the crate version; diff was version-only). Release blocker cleared — remaining work is this task's manual list, owner present.
- 2026-08-10T16:33Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Pre-migration amendment (owner direction, 2026-08-10, README portfolio review): four more tasks ranked ahead — mw-bds8yq5 (seq 30, sazed to v0.2.0 so both live stores union on one engine), mw-2nmsys2 (40, dangling sequence.md entries become a lint finding), mw-chcqk6g (50, prune satisfied sequence.md entries), mw-kkvs8zq (60, drop warns on inbound cross-repo needs). Rationale unchanged from the sweep: the migration turns sequence.md and cross-repo edges into real hand-maintained state, so the guards exist before the state does. The README tasks from the same review (mw-h2qdr6q, mw-qe5y2fc, mw-5fekg2q) stay after — none block the migration.
- 2026-08-10T22:29Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] leras migration reviewed (transcript + repo): solid — landed at leras 6f30ad7, 55 tasks, lint clean, gates green, archives rotated byte-identical. Filed from findings: le-b39mm50 (five cargo verifies close vacuously; D03 rescope), le-nbtzykd (28 rg verifies exit 127 under close's sh -c — rg is a Claude-shell function, no binary on PATH), le-xafm55c (15 prose verifies), le-5d9sxzx (stale TODO/HANDOFF pointers); mw side: skill mesh doctrine, adopt.md hardening, verify traps, import truncation fix, and re-arm of this store's own 39 vacuously-green cargo filters. Manual list unchanged: portfolio init + repos.toml, one real cross-repo needs, PLAN 2.6 flip.
