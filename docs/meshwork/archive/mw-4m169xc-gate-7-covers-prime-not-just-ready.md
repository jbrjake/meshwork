---
id: mw-4m169xc
title: "Gate §7 covers prime, not just ready"
category: core/perf
needs: [mw-ncfg]
verify: out=$(cargo test --release -- --ignored --nocapture perf::prime_1k 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
seq: 230
docs:
  - DESIGN-meshwork.md#§-14-gate
  - DESIGN-meshwork.md#§-7-session-integration
status: done
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
- 2026-08-19T20:01Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-19T20:14Z close attempt — verify failed (dsl)
- 2026-08-19T20:15Z doing→done — verify exit 0 @ 977769e+7

## comments
- 2026-08-19T20:15Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Verify as filed (run cargo test perf::prime_1k) could never go green: the test is #[ignore]d because gate §7 runs perf:: release-only, and the DSL's no-leading-dashes grammar can't spell --ignored — the vacuous-pass guard refused the close exactly as designed. Rewritten to the observed-pass shell idiom on the release invocation; gate scripts and release-only runs are the documented legacy-shell holdout class.
