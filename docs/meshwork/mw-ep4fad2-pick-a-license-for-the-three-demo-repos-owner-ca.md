---
id: mw-ep4fad2
title: Pick a license for the three demo repos — owner call
status: open
category: meta/demo
labels: [owner-call]
verify: contains docs/meshwork/mw-ep4fad2-pick-a-license-for-the-three-demo-repos-owner-ca.md /2026-[0-9-]+ owner picked the demo license/
docs:
  - docs/PLAN-demo-notes.md#settled-decisions
seq: 260
created: 2026-10-02T17:57Z
---

The three public demo repos (jbrjake/meshwork-demo-notes-cli, -sync, -portfolio) ship no LICENSE and no `license` field in their Cargo.toml files. A license is the owner's pick, so the build left it out.

The owner names the license in session and writes a line of the form `<YYYY-MM-DD> owner picked the demo license: <license>` into this file, which the verify reads. Then a session adds the LICENSE file and the Cargo.toml `license` field to each repo. Each repo's license commit lands on `main` after `story/0-day0`, so a re-record (which resets `main` to that tag) would drop it unless the tag moves first.

## log
- 2026-10-02T17:57Z created
