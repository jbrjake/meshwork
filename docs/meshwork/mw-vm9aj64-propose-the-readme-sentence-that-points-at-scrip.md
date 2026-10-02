---
id: mw-vm9aj64
title: Propose the README sentence that points at scripts/demo-full.sh — the owner words it
category: meta/demo
seq: 255
needs: [mw-dym0hck]
verify: contains README.md /scripts\/demo-full\.sh/
docs:
  - docs/PLAN-demo-notes.md#meshwork-integration
  - README.md#quick-start
status: open
created: 2026-10-02T15:21Z
handoff: |
  The draft sentence sits uncommitted in README.md, right after the
  `./scripts/demo.sh` line (line 76), in the same italic style. It names
  the three public repos (linking the demo portfolio), what the full demo
  shows beyond the small one (a refused close, a handoff, a cross-repo ask
  and its answer, a spec change caught by its pins), and that it needs the
  network. scripts/check-readme-transcripts.sh still replays clean with it
  in place.
  
  The owner rewords or accepts it in session. Then close this task (its
  verify passes on the working tree) and commit README.md, which also
  carries the owner's earlier uncommitted plugin-hook edits. Never commit
  the README before the owner says so.
  
  Done around it: scripts/demo-full.sh is committed (c4b5eef) and ran
  green from GitHub, and the notes story is recorded (demo portfolio
  story/recording.md).
---
The README needs one sentence pointing at the full demo, near the line about `./scripts/demo.sh`. It should say what the full demo shows beyond the small one and that it needs the network.

README.md is owner-voiced. Draft the sentence, propose it in session, and let the owner word and place it. Never edit the README unprompted.

## log
- 2026-10-02T15:21Z created
- 2026-10-02T17:57Z handoff by claude (602c381b-d7db-491e-8df6-85682e6152ed)
