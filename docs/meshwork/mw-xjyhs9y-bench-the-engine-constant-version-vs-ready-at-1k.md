---
id: mw-xjyhs9y
title: "Bench the engine constant: --version vs ready at 1K, commit the numbers"
category: core/perf
relates: [mw-ncfg]
verify: cargo bench --bench startup && test -f docs/bench-startup.md
seq: 220
docs:
  - DESIGN-meshwork.md#§-14-gate
status: open
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). hyperfine `meshwork --version` vs
`meshwork ready` at 1K tasks isolates DataFusion planning cost from
process start (N≥7 median, house rule). Either the constant is <20ms
and the complaint was about build times, or it isn't and there's a
number. Right now neither is known.

## log
- 2026-08-08T14:09Z created
