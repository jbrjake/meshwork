---
id: mw-rwb77wp
title: Add the lacks predicate — the inverse of contains, same reader, same confinement
category: core/verify
needs: [mw-t41d6ze]
seq: 540
verify: run cargo test verify_dsl::lacks_predicate
docs:
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
status: done
created: 2026-09-07T16:32Z
---
"This text must be gone" is the most common close condition in a store full of defect fixes
(19 of leras's 43 shell verifies); `absent` is path-level. With R-E item E.2: `lacks <path>
<lit|/regex/>` passes when the file exists and does not match; a missing file fails (never
passes vacuously). Reader, class checks and confinement shared with `contains`.

## log
- 2026-09-07T16:32Z created
- 2026-09-21T16:30Z open→doing — claimed by claude (b80d763e-77c7-4aa5-8ead-40c1daaa1288)
- 2026-09-21T16:35Z doing→done — verify exit 0 @ 9c9cd95+5
