---
id: mw-x5a8g9w
title: "import todo: stop [~] minting unclaimed doing rows"
status: open
category: core/import
discovered-from: mw-mrjhwws
relates:
  - mw-06j1wqe
verify: out=$(cargo test import_marker_doing 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 115
docs:
  - DESIGN-meshwork.md#§-10-migration
created: 2026-08-14T13:28Z
---
[~] maps to doing with no claimant, seeding instant doing-rot (the leras
import minted them straight into the stale-doing class). doing without a
claimant is a lie at import time — decide the mapping (open + a log note
is the honest floor) and encode it.

## log
- 2026-08-14T13:28Z created
