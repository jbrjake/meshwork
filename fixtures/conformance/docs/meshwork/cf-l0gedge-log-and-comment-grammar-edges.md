---
id: cf-l0gedge
title: Tail-grammar edge cases
status: open
created: 2026-08-06
---
Log: date-only free text, an empty entry (NULL date), a token that looks
transition-shaped but names no status (free text), a dateless entry whose
first token projects as the date anyway (mw-e60thg2). Comments: a
nonconforming line is skipped with a warning, never fatal.

## log
- 2026-08-06 created
- 2026-08-06
- 
- 2026-08-06 open→nowhere is free text because nowhere is not a status
- fixed the thing

## comments
- 2026-08-06T09:00Z [spec] A conforming comment.
- this line has no date or bracketed author and is skipped
