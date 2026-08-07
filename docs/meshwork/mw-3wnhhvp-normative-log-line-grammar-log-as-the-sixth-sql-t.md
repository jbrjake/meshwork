---
id: mw-3wnhhvp
title: Normative log-line grammar + log as the sixth SQL table
status: open
category: core/format
needs: [mw-zp1h12d]
verify: cargo test e2e::log_table
docs:
  - DESIGN-meshwork.md#§-4-tables-the-sql-contract
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline
seq: 5
created: 2026-08-06
handoff: |
  Unblocked when minute stamps (mw-zp1h12d) landed 2026-08-07 — you're
  first in queue (seq 5), ahead of the threat model. Grammar heads-up:
  minted log lines now include minute-res stamps (2026-08-06T21:47Z) AND
  the claim suffix from mw-tb6gdr9 ('… — claimed by X'), plus close's
  'close attempt — verify exit N' — the normative grammar must cover
  all minted forms, and old date-only/free-text lines stay legal (parse
  never validates history). The sixth table joins DESIGN §4's five-table
  contract — update the doc table + re-bless json goldens (envelope now
  carries meshwork.version/schema). Tools from this session: file any
  sub-tasks via add --batch with @handles; start --as <you> to claim.
---
Owner-accepted 2026-08-06 (format-hardening review). Transitions are
written as prose (`- <date> open→doing — note`) and prime already
parses them back for recently-done — a de-facto grammar with no spec
and no SQL visibility. Freeze the line grammar the way §2 freezes
comments, then expose the sixth table: log (gid, ord, date,
from_status, to_status, note) — from/to NULL for free-text entries. No
new storage, no new verb; goldens re-bless. Unlocks blocked-duration/
cycle-time queries and activity feeds for any consumer. Drift risk is
the deadline: free-text log lines accumulating across adopted repos
make the retrofit parse ugly — land before the pilot (mw-ntt5 needs
this). The grammar embeds the mw-zp1h12d stamp format.

## log
- 2026-08-06 created
