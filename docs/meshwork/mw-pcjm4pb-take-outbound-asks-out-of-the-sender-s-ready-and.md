---
id: mw-pcjm4pb
title: Take outbound asks out of the sender's ready and next and list them under an asks-out line
category: capability/asks
needs: [mw-vffwacx]
seq: 460
verify: run cargo test ready_excludes_outbound_asks
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-1-the-inbox
  - docs/DESIGN-meshwork.md#§-5-canned-verbs
status: open
created: 2026-09-07T16:32Z
---
A `to:` task sits in its author's `ready` like work, so handoffs open "Nothing to build here —
this is an ask addressed to <repo>". With R-B item B.5: `ready`/`next`/`prime` exclude tasks
carrying `to:` from the worklist and render them as `asks out: N (oldest Dd)` with ids; `show`
on an ask prints `answered by <gid> (<status>)` when an answer exists. The frozen `ready` SQL in
DESIGN §5 gains the one clause and its golden is re-blessed.

## log
- 2026-09-07T16:32Z created
