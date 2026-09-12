---
id: mw-4n00yte
title: Make the lint channel readable — silence archived description-size, fold verify-shell, exempt the own-deliverable doc-missing, quiet this-clone verify edits
category: core/hygiene
seq: 290
verify: run cargo test lint::archived_description_size_silent
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
status: done
created: 2026-09-07T16:32Z
---
274 `verify-shell` lines on every run in the busiest store and `description-size` on immutable
archived tasks bury the real signals. Exclude `archive/` from `description-size`; print
`verify-shell` as one summary line (`N legacy shell verifies — lint --explain verify-shell`) with
the ids behind `--explain`; exempt `doc-missing` when the `docs:` target equals the task's own
`exists` deliverable; suppress `verify-changed-since-approval` when the current text was authored
on this clone, including by a hand-edit in an uncommitted tree (the close gate is untouched).

## log
- 2026-09-07T16:32Z created
- 2026-09-12T16:46Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T16:51Z doing→done — verify exit 0 @ c143e34+10
