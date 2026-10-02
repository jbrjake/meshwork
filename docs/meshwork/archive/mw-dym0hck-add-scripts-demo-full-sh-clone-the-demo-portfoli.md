---
id: mw-dym0hck
title: Add scripts/demo-full.sh — clone the demo portfolio over https and run its replay
category: meta/demo
seq: 250
needs: [mw-0q6m5b9]
verify: "all(exists scripts/demo-full.sh, contains scripts/demo-full.sh /story\\/replay\\.sh/, contains CLAUDE.md /scripts\\/demo-full\\.sh/)"
docs:
  - docs/PLAN-demo-notes.md#meshwork-integration
status: done
created: 2026-10-02T15:21Z
---
`scripts/demo-full.sh` is the full demo's wrapper:
- It `git clone`s https://github.com/jbrjake/meshwork-demo-notes-portfolio into a temp dir over https, with no `gh`.
- It runs `story/replay.sh`, passing `MESHWORK_BIN` through when it is set. The replay otherwise runs the release the demo repos pin.

CLAUDE.md gains its line in the same commit, saying it needs the network. The gate never runs it (zero network). Run it end to end once and read the full output before closing.

## log
- 2026-10-02T15:21Z created
- 2026-10-02T17:54Z open→doing — claimed by claude (602c381b-d7db-491e-8df6-85682e6152ed)
- 2026-10-02T17:57Z doing→done — verify exit 0 @ c4b5eef+2

## comments
- 2026-10-02T17:57Z [claude (602c381b-d7db-491e-8df6-85682e6152ed)] Done: scripts/demo-full.sh clones jbrjake/meshwork-demo-notes-portfolio over https into a temp dir and runs story/replay.sh. MESHWORK_BIN and the replay's flags pass through. CLAUDE.md names both demo scripts and says the full one needs the network and the gate never runs it. Ran end to end from GitHub with exit 0 and read all 529 lines: Beat 3 built the cli against the recorded notesync v0.2.0. That read caught the epilogue picking up the portfolio store's own rows, now fixed in the demo portfolio (752c6ce) and rerun green. ./verify_meshwork.sh: ALL SECTIONS PASS.
