---
id: mw-87gxe7q
title: Make close strip a handoff block whole, and let lint --fix repair a task it cannot parse
category: core/lifecycle
answers: marasi-applied-r-and-d#ar-gfd8g38
seq: 190
verify: run cargo test close_strips_handoff_block_with_blank_line
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
status: done
created: 2026-09-07T16:32Z
---
`close`/`drop` remove the `handoff:` key but leave a block scalar's continuation lines when the
block contains an unindented blank line (hand-authored blocks, legal per SKILL.md); the file then
fails to parse and `lint --fix` refuses because the fixer parses first. Fix the scalar remover in
`src/edit.rs` to consume the whole block by indentation, and give `lint --fix` a pre-parse repair
for exactly this damage or a plain "cannot repair: <reason>" line. Reproduction is the study's §6
row; the lab's ask doc carries the incident.

## log
- 2026-09-07T16:32Z created
- 2026-09-08T00:04Z open→done — verify exit 0 @ 644b16f+11
