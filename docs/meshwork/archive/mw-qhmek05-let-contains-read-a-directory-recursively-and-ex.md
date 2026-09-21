---
id: mw-qhmek05
title: "Let exists take a single-segment glob, and refuse a directory target for contains"
category: core/verify
needs: [mw-t41d6ze]
seq: 550
verify: run cargo test verify_dsl::exists_single_segment_glob
docs:
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
status: done
created: 2026-09-07T16:32Z
---
With R-E item E.3, if ruled: `contains <dir>/ <pat>` walks the confined directory (byte cap,
no symlink following); `exists <path-with-one-*>` matches a dated artifact. Each widens the read
surface inside the repo only; each gets its own class check and its own fixture.

## log
- 2026-09-07T16:32Z created
- 2026-09-21T16:30Z open→doing — claimed by claude (b80d763e-77c7-4aa5-8ead-40c1daaa1288)
- 2026-09-21T16:35Z doing→done — verify exit 0 @ 9c9cd95+6
