---
id: mw-yyf1bab
title: "Surface verify-changed-since-approval as a lint + prime finding"
category: core/verify
needs: [mw-hz1ezcg]
relates: [mw-9rc4vs6]
verify: run cargo test lint::verify_changed_since_approval
seq: 200
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
status: done
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). Approval already blocks execution on a
changed verify; this makes the *diff* visible as a lint/prime finding
instead of just a prompt at close time. The silent weakening is the
attack; the prompt is only a speed bump if the operator is clicking
through.

## log
- 2026-08-08T14:09Z created
- 2026-08-20T11:18Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-20T11:32Z doing→done — verify exit 0 @ e9e1978+9
