---
id: mw-5kp033j
title: "--json envelope: in-band binary + schema version stamp"
status: open
category: core/query
verify: cargo test e2e::json_envelope
docs:
  - REQUIREMENTS-meshwork.md#§-c-query
seq: 7
created: 2026-08-06
---
Owner-accepted 2026-08-06 (format-hardening review; amends MW-C3's
"versioned with the binary" to versioned in-band). Consumers
aggregating --json output across repos pinning different versions have
no in-band signal — and per-repo pinning makes that the NORMAL case.
Every --json output gains an envelope:
`{"meshwork": {"version": "0.1.4", "schema": 1}, ...}`. Breaking for
goldens (re-bless) and any early scripts — exactly why it lands before
external consumers exist, not the day after.

## log
- 2026-08-06 created
