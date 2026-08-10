---
id: mw-rz4ey2h
title: "Long prose fields need a shell-safe path: --handoff/--comment from @file or stdin"
status: open
category: core/lifecycle
verify: cargo test e2e::handoff_from_file
discovered-from: mw-ntt5
seq: 60
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
created: 2026-08-07T13:47Z
---
Pilot evidence (sazed work session): a multi-paragraph --handoff payload
passed inline was corrupted by the shell — a backticked chunk executed
as command substitution ("(eval):18: command not found: pub"), the
stored body was mangled, and a python repair followed. Handoffs are the
fields agents write longest and most carefully; they need a path that
never transits shell quoting: `--handoff @file` / `--handoff -` (stdin),
same for comment. Surface delta — rides the §6 ruling alongside
[[set-fields]].

## log
- 2026-08-07T13:47Z created

## comments
- 2026-08-08T16:42Z [claude] Field evidence (sazed, 2026-08-08 review): handoffs routinely arrive as 300-700 char shell strings with embedded newlines and quotes; task bodies have no CLI path at all, so sessions fall back to cat >> (see mw-t01ek6s for the damage that causes) or python heredocs. @file/stdin would absorb all of it. Rule together with mw-s3905fv (add accepts a body).
- 2026-08-09T23:35Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Field evidence (sazed, 2026-08-09, session f1ee9642): handoffs reached ~1.4KB single-quoted shell strings with escaped backticks and emoji, rewritten twice in the one session. The pattern is holding and growing, not shrinking.
