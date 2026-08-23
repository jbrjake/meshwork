---
id: mw-h2qdr6q
title: Disclose the ~30ms methodology under the README perf claim
category: meta/readme
verify: contains docs/portfolios.md check-perf
docs:
  - README.md
  - DESIGN-meshwork.md#§-14-gate
status: done
created: 2026-08-10T16:31Z
seq: 250
---
The token numbers name their script, state their sample, and disclose the
rule of thumb; the portfolio ~30ms is one tilde, and a skeptic doing the
arithmetic (20 repos at 1K tasks = 20K files in 30ms, ~1.5us each)
correctly concludes the fixtures must be smaller. They are:
tests/suite/perf.rs (gate §7) measures cold-process invocations — median
of N=7, release build, tempdir stores written moments earlier, so page
cache warm — over a 1K-task store (ready_1k_cold) and 20 repos of 50
tasks each (portfolio_20_repos); bench-baseline.json pins 30/31ms behind
scripts/check-perf.sh's 1.5x drift wall. Footnote the claim in the
token-numbers style: name the test and script, state N and median, define
cold as process-cold/cache-warm, and give both fixture shapes. This is
the one number in the doc that could get pulled on and take the credible
ones down with it.

## log
- 2026-08-10T16:31Z created
- 2026-08-23T19:17Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-23T19:17Z doing→done — verify exit 0 @ c0fc956+3

## comments
- 2026-08-23T19:17Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Verify retargeted README.md → docs/portfolios.md: the ~30ms claim moved there (portfolio section split out of README after this task was filed 2026-08-10). The footnote lands under the claim, not in a file that no longer makes it.
