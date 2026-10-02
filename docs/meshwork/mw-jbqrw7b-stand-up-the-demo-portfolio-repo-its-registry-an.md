---
id: mw-jbqrw7b
title: "Stand up the demo portfolio repo — its registry, and R1–R7 filed in its own store"
category: meta/demo
seq: 230
needs: [mw-y5zmkxb, mw-wyvgkt0]
verify: grep -q meshwork-demo-notes-cli ../meshwork-demo-notes-portfolio/repos.toml && grep -q meshwork-demo-notes-sync ../meshwork-demo-notes-portfolio/repos.toml && grep -q meshwork-demo-notes-portfolio ../meshwork-demo-notes-portfolio/repos.toml && grep -q repos.local.toml ../meshwork-demo-notes-portfolio/.gitignore && grep -rqs story/fixtures/gate-rewrite ../meshwork-demo-notes-portfolio/docs/meshwork && grep -rqs mw_refused ../meshwork-demo-notes-portfolio/docs/meshwork && grep -rqs story/recording.md ../meshwork-demo-notes-portfolio/docs/meshwork && git -C ../meshwork-demo-notes-portfolio merge-base --is-ancestor HEAD origin/main
docs:
  - docs/PLAN-demo-notes.md#repo-meshwork-demo-notes-portfolio
status: open
created: 2026-10-02T15:21Z
---
Work in ../meshwork-demo-notes-portfolio (store alias `pf`):
- `repos.toml` registers all three repos with https remotes. Registry names equal repo names.
- `.gitignore` carries `repos.local.toml`.
- File R1–R7 from the corrected plan's work breakdown in this store, each with the plan's verify and the needs between them. R1 closes here once `repos.toml` lands; R2–R7 stay open as that store's worklist.

This registry is the one every demo session resolves through. Builder sessions in any demo repo export `MESHWORK_PORTFOLIO=~/Documents/code/meshwork-demo-notes-portfolio`. Without it, cross-repo refs resolve against the real portfolio, which does not register the demo repos. Registering them there would put their prop backlogs in the real `portfolio ready`.

R6 also waits on P1 and P2 landing in meshwork's main. The demo store cannot write that need, because meshwork lives in the other registry. This store's recording task carries it instead.

## log
- 2026-10-02T15:21Z created
