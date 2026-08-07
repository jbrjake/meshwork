---
id: mw-rz4ey2h
title: "Long prose fields need a shell-safe path: --handoff/--comment from @file or stdin"
status: open
category: core/lifecycle
verify: cargo test e2e::handoff_from_file
discovered-from: mw-ntt5
seq: 255
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
