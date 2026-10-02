---
id: mw-x2jxjqe
title: Check start's status before its red-check runs the verify
category: core/lifecycle
docs: [docs/DESIGN-meshwork.md#12b-trust-boundary-verify-is-untrusted-input-ruled-via-mw-mjwfvxn-2026-08-07]
verify: run cargo test start_refuses_status_before_red_check
status: open
created: 2026-10-02T13:39Z
---
`start` (src/cli/transition.rs) calls `red_check` before `transition()` checks the status, so `start` on a task already doing — or done, since `locate` reaches the archive — runs its verify first (a `run cargo test` verify means a build, up to 300 s) and only then refuses with `status is doing, needs one of [open]`. The status refusal should come first and run nothing.

## log
- 2026-10-02T13:39Z created
