---
id: mw-jbqrw7b
title: "Stand up the demo portfolio repo — its registry, and R1–R7 filed in its own store"
category: meta/demo
seq: 230
needs: [mw-y5zmkxb, mw-wyvgkt0]
verify: grep -q meshwork-demo-notes-cli ../meshwork-demo-notes-portfolio/repos.toml && grep -q meshwork-demo-notes-sync ../meshwork-demo-notes-portfolio/repos.toml && grep -q meshwork-demo-notes-portfolio ../meshwork-demo-notes-portfolio/repos.toml && grep -q repos.local.toml ../meshwork-demo-notes-portfolio/.gitignore && grep -rqs story/fixtures/gate-rewrite ../meshwork-demo-notes-portfolio/docs/meshwork && grep -rqs mw_refused ../meshwork-demo-notes-portfolio/docs/meshwork && grep -rqs story/recording.md ../meshwork-demo-notes-portfolio/docs/meshwork && git -C ../meshwork-demo-notes-portfolio merge-base --is-ancestor HEAD origin/main
docs:
  - docs/PLAN-demo-notes.md#repo-meshwork-demo-notes-portfolio
status: done
created: 2026-10-02T15:21Z
---
Work in ../meshwork-demo-notes-portfolio (store alias `pf`):
- `repos.toml` registers all three repos with https remotes. Registry names equal repo names.
- `.gitignore` carries `repos.local.toml`.
- File R1–R7 from the corrected plan's work breakdown in this store, each with the plan's verify and the needs between them. R1 closes here once `repos.toml` lands; R2–R7 stay open as that store's worklist.

This registry is the one every demo session resolves through. Builder sessions in any demo repo export `MESHWORK_PORTFOLIO=~/Documents/code/meshwork-demo-notes-portfolio`. Without it, cross-repo refs resolve against the real portfolio, which does not register the demo repos. Registering them there would put their prop backlogs in the real `portfolio ready`.

## log
- 2026-10-02T15:21Z created
- 2026-10-02T17:11Z open→doing — claimed by claude (602c381b-d7db-491e-8df6-85682e6152ed)
- 2026-10-02T17:12Z doing→done — verify exit 0 @ 5f9e2db+2

## comments
- 2026-10-02T17:12Z [claude (602c381b-d7db-491e-8df6-85682e6152ed)] Done: repos.toml registers the three repos with https remotes; .gitignore carries repos.local.toml; R1–R7 are filed in the pf store with the plan's verifies and the needs among them (R3 on R2, R5 on R3 and R4, R6 on R5, R7 on R6). R1 is closed. R5–R7 verifies gained an exists arm so lint stays quiet until the file appears. R2 and R3's needs on sync and cli day 0 are added once those tasks exist.
