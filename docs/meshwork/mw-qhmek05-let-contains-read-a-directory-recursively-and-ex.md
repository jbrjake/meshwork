---
id: mw-qhmek05
title: Let contains read a directory recursively and exists take a single-segment glob
category: core/verify
needs: [mw-t41d6ze]
seq: 550
verify: run cargo test verify_dsl::dir_and_glob
docs:
  - docs/DESIGN-meshwork.md#§-12b-trust-boundary
status: open
created: 2026-09-07T16:32Z
---
With R-E item E.3, if ruled: `contains <dir>/ <pat>` walks the confined directory (byte cap,
no symlink following); `exists <path-with-one-*>` matches a dated artifact. Each widens the read
surface inside the repo only; each gets its own class check and its own fixture.

## log
- 2026-09-07T16:32Z created
