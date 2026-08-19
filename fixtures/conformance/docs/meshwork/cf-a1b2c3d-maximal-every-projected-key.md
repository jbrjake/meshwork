---
id: cf-a1b2c3d
title: Maximal task — every projected frontmatter key
status: open
category: engine/spill/budget
labels: [perf, spec]
needs: [cf-42, other#ox-1]
parent: cf-42
discovered-from: cf-42
relates: [cf-h4nd0ff]
verify: exists docs/meshwork/config.toml
docs:
  - FORMAT.md#task-file
attachments:
  - attachments/cf-a1b2c3d/note.txt
seq: 10
github: 7
created: 2026-08-01T09:30Z
---
The kitchen-sink row: exercises every frontmatter key that projects into
the `tasks`, `edges`, and `labels` tables. The description is arbitrary
markdown and projects nowhere.

## log
- 2026-08-01T09:30Z created
- 2026-08-02T10:00Z open→doing — claimed by spec
- 2026-08-02T18:12Z close attempt — verify exit 1
- 2026-08-03T08:00Z doing→open — released, note spans lines
  and this continuation joins with a newline
- 2026-08-03T09:00Z open→doing claimed without the minted em-dash

## comments
- 2026-08-02T11:00Z [spec] A comment whose text continues
  onto a second line, joined by a newline for the identity hash.
- 2026-08-02T12:00Z [reviewer] Evidence convention in text: [evt:ledger:abcd1234] cites an external event.
