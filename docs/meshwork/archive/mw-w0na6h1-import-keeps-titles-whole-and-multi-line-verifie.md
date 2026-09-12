---
id: mw-w0na6h1
title: import keeps titles whole and multi-line verifies intact, and warns when it absorbs inter-checkbox prose
category: core/import
seq: 720
verify: run cargo test import_title_unwrapped
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
status: done
created: 2026-09-07T16:32Z
---
Every adoption session needed a manual repair pass: titles truncated across the frontmatter
wrap, verifies split into fragments, a 340-line section absorbed into one 17 KB body. Three
fixes with fixtures under `fixtures/import/`; low urgency now that the registered repos have
migrated.

## log
- 2026-09-07T16:32Z created
- 2026-09-12T17:18Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T17:22Z doing→done — verify exit 0 @ a2768e6+4
