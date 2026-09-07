---
id: mw-zhdypdz
title: Add set --body, --parent, --from, and a docs replacement to set
category: core/authoring
needs: [mw-vffwacx]
seq: 490
verify: run cargo test set_body_parent_from_docs_replace
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-3-the-fields
status: open
created: 2026-09-07T16:32Z
---
Satisfying one `description-size` warning took six hand-edits; two ANSWER tasks were rewritten
whole seconds after `add`; `set --docs` appends and cannot fix a bad anchor. `set --body
"text"|@file|-` replaces the description above the tail sections; `--parent <id>` and `--from
<id>` set or replace the edge with validation; `--docs <old> <new>` replaces one link (keeping
`--docs <link>` as append). Same clone-approval rules as today; `cli_surface_frozen` re-blessed.

## log
- 2026-09-07T16:32Z created
