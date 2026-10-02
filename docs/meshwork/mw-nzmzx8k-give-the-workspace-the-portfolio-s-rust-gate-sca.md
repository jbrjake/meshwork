---
id: mw-nzmzx8k
title: "Give the workspace the portfolio's Rust gate scaffold — profile.dev, lint flags in .cargo/config.toml, target outside ~/Documents"
category: core/perf
docs: [CLAUDE.md#gates]
verify: "all(contains Cargo.toml /^\\[profile\\.dev\\]/, exists .cargo/config.toml)"
status: open
created: 2026-10-02T13:39Z
---
The portfolio base rules require every Rust workspace to carry `[profile.dev] debug = "line-tables-only"` (+ `split-debuginfo = "unpacked"`, `[profile.dev.package."*"] debug = false`), `-D warnings` as `[build] rustflags`/`rustdocflags` in `.cargo/config.toml` rather than exported by scripts, one test target per crate, and `target/` outside the Spotlight/TCC-covered tree. This repo has no `[profile.dev]` and no `.cargo/config.toml`, and builds into `target/` under `~/Documents`; the debug binary is 368 MB. The test layout (`tests/suite/main.rs`) already complies. Check the gate scripts for exported RUSTFLAGS when moving the flags.

## log
- 2026-10-02T13:39Z created
