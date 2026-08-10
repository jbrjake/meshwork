---
id: mw-0wvndqa
title: "add --dry-run writes task files to disk — §6 promises it writes nothing"
status: done
category: core/lifecycle
verify: cargo test e2e::dry_run_writes_nothing
discovered-from: mw-ntt5
seq: 20
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-07T13:47Z
---
Pilot evidence (sazed): `add "SPEC PROBE" --dry-run` (with and without
--json) printed id+path and left real 247-byte task files on disk that
had to be rm'd — observed twice. §6: "--dry-run prints the would-be
files, writes nothing." Two defects: it writes, and it prints only
id+path instead of the promised file content. Fix both, test both, for
bare `add` and `add --batch`.

## log
- 2026-08-07T13:47Z created
- 2026-08-10T13:37Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-10T13:43Z doing→done — verify exit 0 @ adc0d7b+5
