---
id: mw-5xpt8ak
title: "Count a code fence that opens inside a list item, so the headings after it keep their anchors"
category: core/hygiene
verify: run cargo test list_item_fence_keeps_later_headings
docs:
  - FORMAT.md#clause-pins
status: done
created: 2026-10-02T15:21Z
---

`headings` in src/docs.rs (docs-ref anchors) and `headings` in src/spec.rs (spec clauses) each toggle a fence only on a line that starts with three backticks after leading spaces. CommonMark lets a list item open a fence on its marker line. The scanners miss that opener, read its indented closer as an opener, and from then on treat headings as code and code as headings, until the next unmatched fence flips them back.

Repro: a doc whose list item reads `7. ```` and opens a fence, an indented line of code, the indented closing fence, then `## Later`. `add --docs <doc>#later` warns that the anchor is missing, though GitHub renders the heading. The same scan finds `{#sp-…}` clauses, so a clause after such a list vanishes from `spec audit` and its pins read as dangling.

Recognize a fence after a list marker (`- `, `* `, `+ `, `N. `, `N) `), and have the two scanners share one implementation so they cannot diverge.

## log
- 2026-10-02T15:21Z created
- 2026-10-04T14:41Z open→doing — claimed by claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)
- 2026-10-04T14:55Z doing→done — verify exit 0 @ b6709d9+7
