---
id: mw-z578j81
title: "Batch ingest normalizes like add does: scalar docs, from → discovered-from"
status: done
category: core/authoring
verify: run cargo test batch_scalar_docs
docs:
  - FORMAT.md#task-file
created: 2026-08-12T20:48Z
seq: 40
handoff: |
  Fresh context from this session's batch work: split_documents and
  render_task live in src/cli/add_batch.rs; parse_entry already rewrites
  from→discovered-from (mw-16pyc5g) for TOP-LEVEL keys, so check what
  normalization is still missing vs add's write path (src/write.rs
  yaml_scalar, scalar docs handling in add.rs --docs). The splitter is now
  fence-aware via crate::parse::Fence — do not reintroduce a local scan.
  Red-first: the verify's test name must fail before code. Note parse.rs
  is at 629 lines (750 ceiling, split task mw-0y66mhb pending) — put
  nothing new there.
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
- 2026-08-21T20:18Z open→done — verify exit 0 @ 0dfe1fe+1
