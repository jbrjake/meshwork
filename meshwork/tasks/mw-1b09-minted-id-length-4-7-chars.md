---
id: mw-1b09
title: Minted ID length 4 → 7 chars
status: done
category: core/id
verify: "cargo test id::"
seq: 5
docs:
  - DESIGN-meshwork.md#§-2-task-file-format # §2 format decisions
  - REQUIREMENTS-meshwork.md#§-a-store # MW-A4
created: 2026-08-06
---
Owner ruling 2026-08-06: minted IDs grow from `<alias>-<4-char base32>`
(~1M combinations) to `<alias>-<7-char base32>` (32^7 ≈ 34.4B). Generation
only — parsing never validated length, so existing 4-char IDs stay legal
forever; stores mix lengths freely, no migration. Surface: id.rs mint loop
+ docs, id:: tests (red first), golden import-todo.md re-bless, DESIGN §2
line, MW-A4 example + combination count, README transcripts re-run against
the new binary. lint --fix re-slug and import inherit via IdGen for free.

## log
- 2026-08-06 created
- 2026-08-06 open→doing
- 2026-08-06 doing→done — verify exit 0

## comments
- 2026-08-06 [claude] v0.1.3 tag is on GitHub (f334986) but both tag pushes landed during a GitHub Actions major outage — no release run was created. When Actions recovers: check gh run list for a materialized run first; if none, delete + re-push the tag to re-fire. Release assets not yet published.
