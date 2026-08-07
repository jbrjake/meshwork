---
id: mw-n6nvzpa
title: Store format version marker in config.toml
status: done
category: core/format
verify: cargo test e2e::format_marker
docs:
  - DESIGN-meshwork.md#§-1-on-disk-layout-per-repo
seq: 4
created: 2026-08-06
---
Owner-accepted 2026-08-06 (format-hardening review). A reader has no
way to know which format rules a store follows. init writes
`format = 1` to config.toml; absent means 1; the tool refuses, loudly,
a format newer than it knows. The minting-rule idiom (§15.8) covers
additive change; the first SEMANTIC change needs version detection,
which is unretrofittable archaeology across N adopted stores. Two
lines now. mw-ntt5 needs this so no store is ever minted unmarked.

## log
- 2026-08-06 created
- 2026-08-07T00:25Z open→doing — claimed by claude
- 2026-08-07T00:28Z doing→done — verify exit 0
