---
id: cf-42
title: Terminal done task — legacy short suffix, date-only stamp
status: done
category: engine/spill
verify: "true"
waived: superseded by cf-a1b2c3d
created: 2026-07-30
---
Archive placement carries no semantics; readers load it identically.
Suffix length and alphabet are minting rules — `42` parses fine. The
done transition's trailing `@ <short-sha>[+N]` is a note convention,
not a parse rule.

## log
- 2026-07-30 created
- 2026-08-01T15:00Z open→done — waived: superseded by cf-a1b2c3d @ 1234abc+2
