---
id: mw-78nabpd
title: Reveal package prep — everything short of the flip (the flip stays an owner decision)
status: done
category: release
needs: [mw-5h2mpn7]
verify: exists docs/reveal-prep.md
seq: 40
created: 2026-08-17T03:16Z
handoff: |
  Step 4 of the owner's stated sequence (worklist → release →
  migrations → reveal) — carried by no task until now. Scope:
  everything short of the flip, so the reveal becomes a one-word decision
  instead of a project. README front door rewritten for a stranger (what
  it is, why, the 60-second demo, install path); a one-command demo (init
  → add → ready → close on a scratch repo); the install story (cargo
  install --git or a release binary); and docs/reveal-prep.md as the
  checklist of what the flip itself requires — the license call
  (portfolio decision-queue item 6), which docs stay private, crates.io
  yes/no. The setup-cost matrix (needs edge) supplies the headline number.
  THE FLIP ITSELF IS NOT THIS TASK: publication stays a separate
  intentional owner decision under the 08-04 private-by-default posture.
---

## log
- 2026-08-17T03:16Z created
- 2026-08-17T20:38Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-17T21:04Z doing→done — verify exit 0 @ f433ac7+2

## comments
- 2026-08-17T21:04Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Landed 93a8369 + f433ac7. Reality check first: the repo is ALREADY PUBLIC with v0.3.2 released — the flip is the announcement, not visibility. docs/reveal-prep.md states that and splits done (repo/releases/install/spec/demo/self-host/MIT) from the owner's open calls (portfolio license item 6 — meshwork itself is MIT and shipped; which portfolio-root docs stay private — none live in this repo, task docs: refs dangle harmlessly; crates.io; venue+text). scripts/demo.sh is the one-command zero-network loop, run green this session. README got a three-line surgical delta ONLY (demo pointer, matrix headline, cargo install --git) in its own commit f433ac7 for easy owner review/revert — the owner-passed prose was not rewritten. Headline wired: 9.1 min median ramp / cross-repo switch costs nothing extra, from docs/setup-cost-matrix.md.
