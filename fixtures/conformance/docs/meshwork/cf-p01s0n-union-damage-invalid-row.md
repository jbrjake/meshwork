---
id: cf-p01s0n
title: Union-poisoned file — duplicate top-level key
status: doing
status: blocked
created: 2026-08-05
---
Duplicate `status:` makes this file invalid: it MUST surface as a row
with status `invalid`, the ID recovered from the filename (first two
dash-segments of the stem), and a non-empty error. The error text
itself is reader-defined and not part of this corpus.
