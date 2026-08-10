---
id: mw-5ckb
title: "Registry: repos.toml + local overrides (PLAN 2.1)"
status: done
category: plan/m2
needs: [mw-ntt5, mw-mrjccx2]
verify: cargo test e2e::registry_overrides
seq: 20
docs:
  - REQUIREMENTS-meshwork.md#§-g-portfolio   # MW-G2
  - DESIGN-meshwork.md#§-9-portfolio-master-sequencing
created: 2026-08-05
handoff: |
  M1 is COMPLETE — the sazed pilot closed mw-ntt5 (verdict + 10 filed
  findings in its comments; README now carries the measured before/after).
  You are the first M2 item: repos.toml + repos.local.toml overrides +
  default-path resolution (MW-G2), red-first via e2e::registry_overrides.
  Registry durability groundwork already landed (mw-mrjccx2: rename
  aliases + collision lint) — build on it, don't re-derive. One ordering
  constraint from the pilot: mw-17hnhzk (import drops nested checkboxes,
  silent data loss) MUST land before the leras migration event later in M2
  — it doesn't block registry code, but check it's done before any
  'leras joins' step.
---

## log
- 2026-08-05 created
- 2026-08-10T03:14Z open→done — verify exit 0 @ aadc7a5+5
