---
id: mw-7ywrxf1
title: Print the archive path in show for terminal tasks
category: core/render
seq: 310
verify: run cargo test show_archived_file_path
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-4-discoverability-tax
status: done
created: 2026-09-07T16:32Z
---
`show` prints `file: docs/meshwork/<id>-….md` after `close` moved the file to `archive/`; the
agent fell back to `find` in a sibling store. Render the path the store actually holds.

## log
- 2026-09-07T16:32Z created
- 2026-09-08T00:14Z open→done — verify exit 0 @ f88597e+10
