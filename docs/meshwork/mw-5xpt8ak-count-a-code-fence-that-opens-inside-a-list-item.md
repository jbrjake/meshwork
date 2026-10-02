---
id: mw-5xpt8ak
title: "Count a code fence that opens inside a list item, so the headings after it keep their anchors"
category: core/hygiene
verify: run cargo test list_item_fence_keeps_later_headings
docs:
  - FORMAT.md#clause-pins
status: open
created: 2026-10-02T15:21Z
---
`headings` in src/docs.rs (docs-ref anchors) and `headings` in src/spec.rs (spec clauses) each toggle a fence only on a line that starts with three backticks after leading spaces. CommonMark lets a list item open a fence on its marker line, as in `7. ```` followed by an indented closer. The scanners miss that opener, read its closer as an opener, and from then on treat headings as code and code as headings, until the next unmatched fence flips them back.

Seen on docs/PLAN-demo-notes.md: `add` warns that `#the-replay`, `#meshwork-integration` and `#work-breakdown` are missing, though GitHub renders all three headings. The same scan finds `{#sp-…}` clauses, so a clause after such a list would vanish from `spec audit` and its pins would read as dangling.

Recognize a fence after a list marker (`- `, `* `, `+ `, `N. `, `N) `), and have the two scanners share one implementation so they cannot diverge.

## log
- 2026-10-02T15:21Z created
