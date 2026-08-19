---
id: mw-7c6svyn
title: "FORMAT.md conformance corpus: golden store + expected parse"
category: core/format
relates: [mw-dg5j1sv]
verify: run cargo test format::conformance_corpus
seq: 260
docs:
  - FORMAT.md#task-file
status: done
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). A golden store + expected parse output a
third-party reader can self-check against. "Where the spec and the
binary disagree, the spec wins" is aspirational until there's a fixture
set that can prove one of them wrong. Spec hardening — do it before
anyone implements a reader from FORMAT.md.

## log
- 2026-08-08T14:09Z created
- 2026-08-19T21:31Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-19T21:39Z doing→done — verify exit 0 @ d886d4b+6
