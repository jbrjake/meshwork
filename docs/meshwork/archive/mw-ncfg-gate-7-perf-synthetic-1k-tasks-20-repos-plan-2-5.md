---
id: mw-ncfg
title: "Gate §7 perf: synthetic 1K tasks / 20 repos (PLAN 2.5)"
status: done
category: plan/m2
needs: [mw-jpbv]
verify: ./verify_meshwork.sh
seq: 60
docs:
  - REQUIREMENTS-meshwork.md#§-c-query   # MW-C4
  - DESIGN-meshwork.md#§-14-gate   # §7 perf
created: 2026-08-05
---

## log
- 2026-08-05 created
- 2026-08-10T04:05Z open→doing — claimed by Jon Rubin
- 2026-08-10T04:29Z doing→done — verify exit 0 @ 7a68f75+9

## comments
- 2026-08-10T04:31Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Gate finding while landing 2.5: §7's original stanza (tail -1 | grep 'test result: ok') was unsatisfiable — cargo's doc-test phase ends the merged 2>&1 stream with a trailing blank line, so tail -1 always saw an empty line. Latent since the stanza was written; §7 SKIPped until today so it never fired. Diagnosed by teeing the pipeline inside a real gate run (standalone repros kept passing because I eyeballed tail -3). Fixed to exit-code + printed perf-median evidence, same pattern as §3. Lesson for gate authors: grep-the-last-line is not a measurement; exit codes are.
