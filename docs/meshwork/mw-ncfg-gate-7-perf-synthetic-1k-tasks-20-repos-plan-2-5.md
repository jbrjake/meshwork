---
id: mw-ncfg
title: "Gate §7 perf: synthetic 1K tasks / 20 repos (PLAN 2.5)"
status: open
category: plan/m2
needs: [mw-jpbv]
verify: ./verify_meshwork.sh
seq: 60
docs:
  - REQUIREMENTS-meshwork.md#§-c-query   # MW-C4
  - DESIGN-meshwork.md#§-14-gate   # §7 perf
created: 2026-08-05
handoff: |
  2.4 done: sequence.md + portfolio next live (golden portfolio-next.txt);
  union + resolution all landed. 2.5 = gate §7: seeded synthetic
  generators (1K tasks single repo, 20 repos portfolio), ready <100ms cold
  / portfolio <1s, N>=7 reps median (MW-C4), release build, wire into
  verify_meshwork.sh §7 (currently SKIPs) + check-perf.sh &
  bench-baseline.json per baseline rule. MESHWORK_ID_SEED exists for
  determinism. Perf tests likely a perf:: prefix per §7's grep. Note
  mw-xjyhs9y (bench engine constant at 1K) overlaps — check it before
  writing new harness.
---

## log
- 2026-08-05 created
