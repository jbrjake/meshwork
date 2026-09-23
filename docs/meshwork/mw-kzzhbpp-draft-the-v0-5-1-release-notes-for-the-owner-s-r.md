---
id: mw-kzzhbpp
title: "Draft the v0.5.1 release notes for the owner's review — the deliverable rule, the one-code explain, the batch values; plain English, user impact first"
category: meta/release
seq: 130
needs: [mw-x82yqq3, mw-4ccvmff, mw-qf0bsb6]
docs:
  - docs/release-notes/TEMPLATE.md
verify: all(exists docs/release-notes/RELEASE-NOTES-v0.5.1.md, contains docs/release-notes/RELEASE-NOTES-v0.5.1.md /^# meshwork v0.5.1/)
status: open
created: 2026-09-23T18:32Z
---
`docs/release-notes/RELEASE-NOTES-v0.5.1.md` on the TEMPLATE shape: a point release,
three fixes found by an adopter's first day on v0.5.0. Owner-voiced: draft and leave
uncommitted for review; the owner lands the prose and commits. No internal ids, no repo
jargon, one fact per sentence. This task's own verify is the deliverable shape the
release fixes — `exists` beside `contains` on one path — and lint goes quiet on it once
the first fix lands.

## log
- 2026-09-23T18:32Z created
