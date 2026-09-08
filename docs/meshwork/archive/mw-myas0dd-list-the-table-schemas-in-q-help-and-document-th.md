---
id: mw-myas0dd
title: List the table schemas in q --help and document the --json envelope shape
category: core/query
seq: 320
verify: run cargo test q_help_lists_schema
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-4-discoverability-tax
  - FORMAT.md#§-projection
status: done
created: 2026-09-07T16:32Z
---
Two sessions reverse-engineered the column list with `SELECT * FROM tasks LIMIT 2` (1.5 KB body
cells) and `pragma_table_info`. `q --help` prints every queryable table with its columns from the
same list the error path names (`tables::TABLES`, and the views once registered), and says that
`--json` nests rows under `data.rows`.

## log
- 2026-09-07T16:32Z created
- 2026-09-08T00:32Z open→done — verify exit 0 @ 1b8cda5+12
