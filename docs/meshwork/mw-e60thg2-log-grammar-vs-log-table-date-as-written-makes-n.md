---
id: mw-e60thg2
title: "log grammar vs log table: date-as-written makes NULL unreachable"
category: core/format
relates: [mw-7c6svyn]
verify: out=$(cargo test format::log_date_nullability 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
docs:
  - FORMAT.md#tail-section-grammars
  - FORMAT.md#projection
status: open
created: 2026-08-09T23:17Z
---
Review finding (2026-08-09). The `## log` grammar says `date = first
whitespace-delimited token, as written` — unconditionally. A hand-written
`- fixed the thing` therefore yields `date=fixed`. But the projection's
`log` table defines `date` as "NULL if the entry has none," which under
this grammar can never happen. An implementer has to guess. Pick one:

- a date-shape predicate decides whether token one is a date, and
  non-matching entries get `date=NULL` (token stays in the text), or
- token one is always the date column, and the NULL clause is deleted.

Either way the ruling lands in both sections and the conformance corpus
(mw-7c6svyn) carries the deciding fixture — this contradiction is
precisely what that corpus exists to surface.
