---
id: mw-z578j81
title: "Batch ingest normalizes like add does: scalar docs, from → discovered-from"
status: open
category: core/authoring
verify: run cargo test batch_scalar_docs
docs:
  - FORMAT.md#task-file
created: 2026-08-12T20:48Z
---
Two authoring-parity gaps in `add --batch`, both observed in the wild:

1. A scalar `docs:` string is rejected ("invalid type: string … expected
   a sequence — nothing written", leras 6f063ba1 21:15) while the same
   scalar hand-written into a task file lints clean. One rule: accept a
   scalar as a one-element sequence, or reject it everywhere.
2. A `from:` key passes through raw, and lint then flags it
   `unknown-key` — 7 sazed files across 2 sessions, warning still live
   5 days later — while `add --from` correctly writes
   `discovered-from:`. Normalize at ingest.

The verify's test (`batch_scalar_docs`) should cover both: a batch doc
with scalar `docs:` and a `from:` line round-trips into a lint-clean
file carrying `discovered-from:`.

## log
- 2026-08-12T20:48Z created
