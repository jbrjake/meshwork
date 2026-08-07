---
id: mw-zp1h12d
title: Minute-resolution UTC stamps on minted log/comment lines
status: doing
category: core/format
verify: cargo test e2e::minute_stamps
docs:
  - DESIGN-meshwork.md#§-2-task-file-format
  - REQUIREMENTS-meshwork.md#§-i-concurrency-merge
seq: 3
created: 2026-08-06
claimed-by: claude
---
Owner-accepted 2026-08-06 (format-hardening review; DESIGN §2
amendment lands here). Every stamp today is a civil date. Three costs,
all worse the more clones/machines participate: union-merge interleaves
same-day comment appends from two clones with no recoverable order
(comments.ord is file position); the mirror comment hash
(date+author+text, §8) collides on same-day identical text from one
author — the second comment silently never pushes; prime's weather/
recently-done ordering is undefined within a day. Fix follows the
§15.8 idiom — a MINTING rule, never validation: new log lines, comment
lines, and created: stamp UTC minute resolution (2026-08-06T21:47Z);
the parser accepts date-only forever; display may keep date-only.
Derived last-activity = max stamp in file (a stored updated: field is
rejected — one shared frontmatter line is a union-merge hotspot).
Land before the pilot (mw-ntt5 needs this): cheap now, corpus-wide
guesswork after stores multiply.

## log
- 2026-08-06 created
- 2026-08-07 open→doing — claimed by claude
