---
id: mw-8fmsws3
title: Terminal escape sanitization in rendered task content
status: open
category: core/render
discovered-from: mw-mjwfvxn
verify: cargo test e2e::render_sanitized
docs:
  - DESIGN-meshwork.md#§-12b-trust-boundary
created: 2026-08-07T01:55Z
---
Named as an adjacent surface in DESIGN §12b (not covered by the MW-E5
ruling — different class): task titles, log lines, comments, and
blocked-reasons from untrusted files render straight to the operator's
terminal (show/prime/ready/why) and into the SessionStart hook's
injected context. Raw ESC/CSI/OSC sequences can spoof output or worse
in some emulators; for agent sessions, injected control text is prompt
surface. Strip or escape C0/C1 controls (except \n and the two-space
continuation shape) at render time, never at storage — files keep
bytes as written. e2e proves a hostile fixture renders inert.

## log
- 2026-08-07T01:55Z created
