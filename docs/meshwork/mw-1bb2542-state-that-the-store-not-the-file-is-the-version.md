---
id: mw-1bb2542
title: "State that the store, not the file, is the versioning unit"
category: core/format
verify: grep -qi 'versioning unit' FORMAT.md
docs:
  - FORMAT.md#configtoml
status: open
created: 2026-08-09T23:17Z
---
Review finding (2026-08-09). `format = 1` lives in config.toml, so a
single task file pasted into an issue or emailed carries no version —
awkward for a project whose slogan is "files are the API." Not worth a
per-file key; worth one sentence: the store is the versioning unit, and
a bare task file encountered outside a store is read at the reader's
current format version.

## log
- 2026-08-09T23:17Z created
