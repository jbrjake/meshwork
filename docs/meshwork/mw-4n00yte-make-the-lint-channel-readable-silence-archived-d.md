---
id: mw-4n00yte
title: Make the lint channel readable — silence archived description-size, fold verify-shell, exempt the own-deliverable doc-missing, quiet this-clone verify edits
category: core/hygiene
seq: 290
verify: run cargo test lint::archived_description_size_silent
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
  - docs/FIELD-STUDY-session-transcripts.md#§-2-6-the-verify-economy
status: open
created: 2026-09-07T16:32Z
handoff: |
  Fresh session, nothing started. Four independent pieces, one test each;
  the verify names the first.
  1. description-size: skip terminal tasks in lint::check_budgets
  (archived files are immutable) — test
  lint::archived_description_size_silent in tests/suite/lint.rs (675
  lines, room for ~60).
  2. verify-shell fold: in src/cli/lint.rs text mode, print one line 'N
  legacy shell verifies — lint --explain verify-shell' and hide the
  per-id rows; add --explain <code> to LintArgs (DESIGN §6 lint row gains
  [--explain code]); JSON keeps every finding.
  3. doc-missing exemption: in lint::check_docs skip a link whose path
  (before #) equals the task's own 'exists <path>' verify
  (verify_dsl::classify → Predicate::Exists).
  4. verify-changed-since-approval: lint_verify::changed_since_approval
  should take the RepoStore, not &[&Task], and skip entries whose file is
  dirty in 'git status --porcelain -- docs/meshwork' (an uncommitted
  hand-edit was made here); prime.rs calls it too. The close gate stays
  untouched.
  Gate takes ~4 min; do not edit src/tests while it runs.
---
274 `verify-shell` lines on every run in the busiest store and `description-size` on immutable
archived tasks bury the real signals. Exclude `archive/` from `description-size`; print
`verify-shell` as one summary line (`N legacy shell verifies — lint --explain verify-shell`) with
the ids behind `--explain`; exempt `doc-missing` when the `docs:` target equals the task's own
`exists` deliverable; suppress `verify-changed-since-approval` when the current text was authored
on this clone, including by a hand-edit in an uncommitted tree (the close gate is untouched).

## log
- 2026-09-07T16:32Z created
