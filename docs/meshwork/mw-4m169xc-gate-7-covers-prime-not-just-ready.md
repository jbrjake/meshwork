---
id: mw-4m169xc
title: "Gate §7 covers prime, not just ready"
category: core/perf
needs: [mw-ncfg]
verify: cargo test perf::prime_1k
seq: 230
docs:
  - DESIGN-meshwork.md#§-14-gate
  - DESIGN-meshwork.md#§-7-session-integration
status: open
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). `prime` is the one command in the
SessionStart hot path and the one growing without bound — read-time
output is capped, parse-time input isn't. 123 tasks is fine; 1K with
long comment tails is the case to measure before the
`.cache/tasks.jsonl` projection stops being deferrable. Builds on the
synthetic 1K harness that arrives with mw-ncfg (PLAN 2.5).

## log
- 2026-08-08T14:09Z created
