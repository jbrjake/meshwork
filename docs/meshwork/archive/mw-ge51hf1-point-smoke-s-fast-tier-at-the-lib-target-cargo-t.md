---
id: mw-ge51hf1
title: Point smoke's fast tier at the lib target — cargo test --bins runs zero tests since the crate split
status: done
category: core/hygiene
verify: contains scripts/smoke.sh /cargo test --lib/
docs:
  - docs/DESIGN-meshwork.md#§-14-gate
seq: 570
created: 2026-09-21T15:35Z
---

scripts/smoke.sh calls its last section "fast unit tests" and runs `cargo test --bins`, which reports `smoke: OK (0 tests)` on every commit: the unit tests live in src/ modules of the library target (`cargo test --lib` runs 22 there), and the binary target carries none. The pre-commit gate has been vacuous on unit tests since the crate split into lib + bin. Fix: run `--lib` (and keep the bin harness if it ever gains tests), and make the OK line refuse a zero count — a gate that reports 0 tests as OK is the vacuity the verify DSL already refuses for `run cargo test`.

## log
- 2026-09-21T15:35Z created
- 2026-09-21T16:36Z open→doing — claimed by claude (b80d763e-77c7-4aa5-8ead-40c1daaa1288)
- 2026-09-21T16:39Z doing→done — verify exit 0 @ 0eed1b1+4
