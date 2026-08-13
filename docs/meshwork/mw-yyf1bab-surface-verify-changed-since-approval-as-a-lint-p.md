---
id: mw-yyf1bab
title: "Surface verify-changed-since-approval as a lint + prime finding"
category: core/verify
needs: [mw-hz1ezcg]
relates: [mw-9rc4vs6]
verify: out=$(cargo test lint::verify_changed_since_approval 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 200
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
status: open
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). Approval already blocks execution on a
changed verify; this makes the *diff* visible as a lint/prime finding
instead of just a prompt at close time. The silent weakening is the
attack; the prompt is only a speed bump if the operator is clicking
through.

## log
- 2026-08-08T14:09Z created
