---
id: mw-svbdkvd
title: "Fence-aware body scan: a quoted `## log` inside a fenced code block must not start the tail sections"
category: core/format
labels: [bug]
verify: run cargo test fenced_heading_stays_body
discovered-from: mw-3gpdbbh
seq: 40
status: done
created: 2026-08-21T19:23Z
---
Same damage class as the batch splitter, different boundary scanners. A
task body that quotes the tail grammar in a fence — a doc task showing
the file format, a bug report with a repro — hits every scanner that
recognizes headings without tracking fence state:

- `parse.rs` `parse_body`: the first un-indented `## log` line switches
  to tail mode even inside a fence, so everything after it is read as
  log entries and dropped from the description.
- `edit.rs` `append_section_entry`: a fenced `## log` is found by the
  heading position scan, so a real log append can land inside the fence.
- `lint_tail.rs` stray-content relocation and `cli/lint.rs` tail checks
  scan `## ` lines the same way and can misclassify fenced content.

The splitter fix left a `fence_run` helper in `cli/add_batch.rs`
(CommonMark: up to 3 leading spaces, 3+ backticks or tildes, longer runs
close only on runs at least as long). Lift it somewhere shared and thread
fence state through the body scanners; the batch test
`batch_ignores_separators_in_fenced_code` is the shape to mirror.

## log
- 2026-08-21T19:40Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-21T19:52Z doing→done — verify exit 0 @ 179c096+2

## comments
- 2026-08-21T19:52Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Landed as one tracker in parse.rs (Fence, fence_run, fenced_lines) threaded through parse_body, append_section_entry, relocate_stray, status_from_log, and the batch splitter. Deliberate deviation from CommonMark: fences recognized only at indent 0-1, because 2-space indent is entry-continuation space — a comment quoting code can never mask the real headings below it.
