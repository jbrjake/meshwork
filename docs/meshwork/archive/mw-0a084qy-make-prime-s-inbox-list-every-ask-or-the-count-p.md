---
id: mw-0a084qy
title: Make prime's inbox list every ask, or the count plus the oldest age plus the verb that lists them
category: capability/asks
relates: ["meshwork#mw-r6g9bhe"]
seq: 450
verify: run cargo test prime_inbox_lists_all_or_names_the_verb
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-1-the-inbox
  - docs/DESIGN-meshwork.md#§-7b-prime
status: done
created: 2026-09-07T16:32Z
---
`ADDRESSED_ROWS = 3`, oldest first, then `… and N more addressed` with no command to expand it;
no session in the corpus ever followed that line. Under the byte budget: print every inbound ask
while it fits; when it does not, print the count, the oldest age, and the exact `portfolio q`
statement (the `asks` verb once ruled). `ready`'s footnote gets the same treatment. The age
itself is `mw-r6g9bhe`.

## log
- 2026-09-07T16:32Z created
- 2026-09-12T16:41Z open→doing — claimed by claude (a4b15620-cf77-4910-ac26-1c233b1b0c38)
- 2026-09-12T16:45Z doing→done — verify exit 0 @ 3df54d6+8
