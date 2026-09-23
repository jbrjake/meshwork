---
id: mw-4ccvmff
title: "Make lint --explain <code> print that code's rows alone — a code with neither a fold nor a note printed the whole report unchanged"
category: lint/report
seq: 110
docs:
  - docs/DESIGN-meshwork.md#§-6-cli-surface
  - src/cli/lint.rs
verify: run cargo test e2e::lint_explain_one_code
status: open
created: 2026-09-23T18:32Z
---
Observed in sazed (v0.5.0): `lint --explain needs-behind | head -20` printed the first
twenty rows of the whole report — `archive-loose`, `dangling`, `description-size` — and
never a `needs-behind` row. `--explain` today only unfolds a folded code or prints a
heuristic's note above the unchanged report; any other code is silently ignored.

Shape: with `--explain <code>`, the text report is that code's rows only — the note first
when the code has one — followed by `N <code> finding(s) — E error(s), W warning(s) in
all`. No rows: `no <code> findings; codes in this report: a, b, c` (an unknown code reads
the same way). Exit status unchanged (errors still fail). `--json` is untouched; the
existing fold and note tests keep passing.

Docs in the same commit: DESIGN §6 `lint` row, SKILL.md session-ritual line on `--explain`.

## log
- 2026-09-23T18:32Z created
