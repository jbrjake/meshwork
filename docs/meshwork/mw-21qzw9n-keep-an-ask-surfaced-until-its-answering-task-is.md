---
id: mw-21qzw9n
title: Keep an ask surfaced until its answering task is terminal, rendered answered-by with the answer's status
category: capability/asks
needs: [mw-vffwacx]
seq: 470
verify: run cargo test addressed_visible_until_answer_terminal
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-1-the-inbox
  - docs/PROPOSAL-analytics.md#§-4-6-asks
status: open
created: 2026-09-07T16:32Z
---
`src/addressed.rs` drops an ask when any non-dropped task carries `answers:`; an open
placeholder empties the inbox before a line of work exists. With R-B item B.2: an ask leaves the
inbox only when an answer is `done` (or the ask itself is terminal); an open answer renders
`answered-by <gid> (open)`. DESIGN §15.12 and FORMAT.md's reader semantics change in the same
commit; the `asks` view's `answer_open`/`answer_done` columns are the spec and the linked fixture
pins both states.

## log
- 2026-09-07T16:32Z created
