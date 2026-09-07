---
id: mw-c3s9209
title: Warn verify-path-missing when a contains or grep verify names a path that does not exist
category: core/verify
answers: sazed#sa-b9y0pxe
seq: 210
verify: run cargo test lint::verify_path_missing
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
status: open
created: 2026-09-07T16:32Z
---
`doc-missing`'s twin on the field that decides closability: a live task whose `contains <path>`
or legacy `grep … <path>` names a file absent from the tree can never close and looks like
unfinished work (22 days in sazed after a doc rotation). Warning on live tasks only; `exists`
is excluded by definition (the artifact task's red state). The weft's W1 is the same check at a
ref across stores; this is the single-store form adopters asked for. The sazed ask closes on a
dated `verify-path-missing SHIPPED and running here` line in its own file.

## log
- 2026-09-07T16:32Z created
