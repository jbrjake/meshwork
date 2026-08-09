---
id: mw-n3xgfs0
title: "lint --fix relocates stray body content above the tail sections"
category: core/store
verify: cargo test lint::fix_stray_body_relocation
docs:
  - FORMAT.md#task-file
  - FORMAT.md#tail-section-grammars
status: open
created: 2026-08-09T23:35Z
---
Field evidence (sazed, 2026-08-09). Nine task files were hand-written
with `##`-headed bodies below `## log`; the parser ignored the content
per spec, the session read that as data loss and filed a defect against
meshwork, and the eventual hand repair nearly swallowed a log entry —
one file had its body sitting *between* two entries.

The repair is mechanical and belongs in `--fix`: within the tail
sections, `- ` bullets, two-space continuations, and blanks are legal;
anything else — including `##` headings — is stray body. Relocate it
above `## log` preserving order, and assert the multiset of non-blank
lines is unchanged (the invariant the agent had to hand-roll). One
command instead of a nine-file structural rewrite.
