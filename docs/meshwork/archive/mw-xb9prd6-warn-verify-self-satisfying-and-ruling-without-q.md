---
id: mw-xb9prd6
title: Warn verify-self-satisfying and ruling-without-quote; refuse a --waive reason carrying a placeholder
category: core/verify
seq: 650
verify: run cargo test lint::verify_self_satisfying
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
  - docs/FIELD-STUDY-session-transcripts.md#§-2-7-behavioral-psychology
status: done
created: 2026-09-07T16:32Z
---
A `contains` whose target is the task's own file or a comment the CLI can write, without a
date-first marker, is satisfiable by the author; a body or comment carrying `owner ruled` /
`OWNER CALL` / `Owner ruling` with no quoted owner text is the authority-laundering shape. Both
are lexical heuristics and say so in `--explain`; the second is dropped if its first month is
mostly false. `close --waive` refuses a reason containing `<…>`.

## log
- 2026-09-07T16:32Z created
- 2026-09-12T17:09Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T17:13Z doing→done — verify exit 0 @ e67c3d9+12
