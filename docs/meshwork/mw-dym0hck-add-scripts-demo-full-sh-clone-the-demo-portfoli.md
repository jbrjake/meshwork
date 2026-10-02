---
id: mw-dym0hck
title: Add scripts/demo-full.sh — clone the demo portfolio over https and run its replay
category: meta/demo
seq: 250
needs: [mw-0q6m5b9]
verify: "all(exists scripts/demo-full.sh, contains scripts/demo-full.sh /story\\/replay\\.sh/, contains CLAUDE.md /scripts\\/demo-full\\.sh/)"
docs:
  - docs/PLAN-demo-notes.md#meshwork-integration
status: open
created: 2026-10-02T15:21Z
---
`scripts/demo-full.sh` is the full demo's wrapper:
- It resolves the binary the way `scripts/demo.sh` does.
- It `git clone`s https://github.com/jbrjake/meshwork-demo-notes-portfolio into a temp dir over https, with no `gh`.
- It runs `story/replay.sh` with `MESHWORK_BIN` set.

CLAUDE.md gains its line in the same commit, saying it needs the network. The gate never runs it (zero network). Run it end to end once and read the full output before closing.

## log
- 2026-10-02T15:21Z created
