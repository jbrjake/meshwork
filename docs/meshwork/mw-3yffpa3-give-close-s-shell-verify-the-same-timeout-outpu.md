---
id: mw-3yffpa3
title: "Give close's shell verify the same timeout, output cap and env scrub as the DSL run"
category: core/verify
docs: [docs/DESIGN-meshwork.md#12b-trust-boundary-verify-is-untrusted-input-ruled-via-mw-mjwfvxn-2026-08-07]
verify: run cargo test close_shell_verify_is_capped_and_timed
status: open
created: 2026-10-02T13:32Z
---
`run_shell` in src/cli/close.rs runs `sh -c <verify>` through plain `.output()`: no wall clock, no output cap, the caller's whole environment. `start`'s red-check of the same text goes through `verify_exec::spawn_capped` (300 s, 256 KiB, scrubbed env), so a shell verify that hangs or floods is bounded at `start` and unbounded at `close`.

## log
- 2026-10-02T13:32Z created
