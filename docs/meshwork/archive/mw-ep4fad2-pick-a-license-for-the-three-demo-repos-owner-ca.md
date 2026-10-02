---
id: mw-ep4fad2
title: License the three demo repos under MIT
status: done
category: meta/demo
labels: [owner-call]
verify: grep -q 'MIT License' ../meshwork-demo-notes-cli/LICENSE && grep -q 'MIT License' ../meshwork-demo-notes-sync/LICENSE && grep -q 'MIT License' ../meshwork-demo-notes-portfolio/LICENSE && grep -q '^license = "MIT"' ../meshwork-demo-notes-cli/Cargo.toml && grep -q '^license = "MIT"' ../meshwork-demo-notes-sync/Cargo.toml && grep -q '^license = "MIT"' ../meshwork-demo-notes-portfolio/story/fixture-gen/Cargo.toml && git -C ../meshwork-demo-notes-cli merge-base --is-ancestor HEAD origin/main && git -C ../meshwork-demo-notes-sync merge-base --is-ancestor HEAD origin/main && git -C ../meshwork-demo-notes-portfolio merge-base --is-ancestor HEAD origin/main
docs:
  - docs/PLAN-demo-notes.md#settled-decisions
seq: 260
created: 2026-10-02T17:57Z
---

The three public demo repos (jbrjake/meshwork-demo-notes-cli, -sync, -portfolio) are MIT, by owner ruling in session 2026-10-02.

Each repo gets a LICENSE with meshwork's MIT text and copyright line, and each crate's Cargo.toml gets `license = "MIT"`: the cli, notesync, and the portfolio's `story/fixture-gen`. Push main in each.

The license commits land on `main` after the story tags. A re-record resets `main` to `story/0-day0`, which would drop them unless they are re-applied after it.

## log
- 2026-10-02T17:57Z created
- 2026-10-02T18:13Z open→doing — claimed by claude (602c381b-d7db-491e-8df6-85682e6152ed)
- 2026-10-02T18:14Z doing→done — verify exit 0 @ e2026f5+1

## comments
- 2026-10-02T18:13Z [claude (602c381b-d7db-491e-8df6-85682e6152ed)] Owner ruling in session 2026-10-02: 'demo repos are mit'.
- 2026-10-02T18:14Z [claude (602c381b-d7db-491e-8df6-85682e6152ed)] Done: LICENSE (meshwork's MIT text) and license = "MIT" are pushed: cli 4eec8ef, sync 4f6592d, portfolio 02a705d (story/fixture-gen). Both crates' gates pass locally and CI is green on the new heads. GitHub's license API reports MIT for all three repos.
