---
id: mw-drrvpsg
title: "prime weather: import-provenance log lines are noise — surface substance or nothing"
status: open
category: core/render
verify: cargo test e2e::weather_skips_import_log
discovered-from: mw-ntt5
seq: 40
docs:
  - DESIGN-meshwork.md#§-7-session-integration
created: 2026-08-07T13:47Z
---
Pilot evidence (sazed): all 8 imported `doing` tasks render the identical
line "— 2026-08-07T04:20Z imported from TODO.md" in every prime — 8 of
prime's weather lines carry zero information, spent from a 6,144-byte
budget. Weather should prefer the newest substantive comment/log entry
and skip pure provenance stamps (or say nothing after the title). Same
event also suggests import should not stamp its provenance as the
newest-visible log line on doing tasks.

## log
- 2026-08-07T13:47Z created
