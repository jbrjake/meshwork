---
id: mw-6mqm4em
title: "import todo: warn when a title imports as one cryptic token"
status: open
category: core/import
discovered-from: mw-mrjhwws
verify: out=$(cargo test import_short_title 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 110
docs:
  - DESIGN-meshwork.md#§-10-migration
created: 2026-08-14T13:28Z
handoff: |
  Import pipeline shape (post mw-mrjhwws/mw-gsgh8s7): parse_todo returns
  (items, carried); title/context/verify extraction happens in the
  finalize loop at the bottom via split_headline/extract_command;
  user-facing counts print in todo()'s summary block
  (~src/cli/import.rs:110-130 — the carried_n println is the pattern to
  copy). For this task: after finalize, flag titles that are one short
  token (no whitespace; think R11) with a per-line stderr warning plus a
  summary count. Test import_short_title red-checked 2026-08-14 (observed
  exit 1); write it in tests/suite/e2e_import.rs —
  git_repo/init_store/meshwork/stdout_of idioms. Sibling mw-x5a8g9w
  touches parse_marker's '~' arm; decide open+log-note vs doing there, not
  here. cargo fmt BEFORE clippy — fmt re-wraps can push a fn over the
  100-line cap (bit this session twice).
---
sazed imported tasks titled just R11, R8, R7 — unintelligible in every
listing three days later. The import summary should warn per single-token
title so the review pass retitles them as work orders, not codes.

## log
- 2026-08-14T13:28Z created
