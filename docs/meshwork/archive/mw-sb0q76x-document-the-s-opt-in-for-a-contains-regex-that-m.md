---
id: mw-sb0q76x
title: "Document the (?s) opt-in for a contains regex that must span a line wrap, and pin it with a test"
category: core/verify
answers: portfolio#po-5sv8hs3
seq: 365
verify: "all(contains docs/DESIGN-meshwork.md /\\(\\?s\\)/, run cargo test verify_dsl::exec_contains_dot_all_opt_in)"
docs:
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
status: done
created: 2026-09-21T15:28Z
---
A `contains` regex is grep-like on purpose: `^`/`$` anchor lines and `.` stops at a newline,
which is what every `/^## Heading/` and dated-marker verify means. A two-phrase `.*` pattern
against prose that wraps at 95 columns therefore misses a marker that is present — the 09-21
red team measured it on `po-0a9b6dy`.

Neither of the ask's two options is taken whole. Dot-all by default would weaken every existing
two-phrase predicate at once, and lint cannot know which `.*` an author meant to stay on one
line. The regex engine already carries the per-predicate answer: an inline `(?s)` at the front
of the pattern lets `.` cross the wrap for that predicate only, and the default stays grep-like.
This task pins both facts with an executor test and writes the idiom where authors read —
DESIGN §12b's executor paragraph and the skill's Verifies block — beside the single-line marker
idiom, which stays the recommended shape.

## log
- 2026-09-21T15:28Z created
- 2026-09-21T15:29Z open→doing — claimed by claude (b80d763e-77c7-4aa5-8ead-40c1daaa1288)
- 2026-09-21T15:34Z doing→done — verify exit 0 @ 7c49b77+3
