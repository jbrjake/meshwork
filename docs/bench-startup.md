# bench-startup — the engine constant

What `meshwork ready` at 1K tasks costs beyond bare process start
(mw-xjyhs9y). Refresh with `cargo bench --bench startup` and replace the
table; the bench is the method of record, this file is the numbers.

## Numbers (2026-08-19, Apple M5 Max, macOS 26.5.2, release build)

| measurement | median ms |
|---|---|
| `meshwork --version` (process start + clap) | 5 |
| `meshwork ready`, 1K-task store, cold | 24–26 |
| **engine constant** (delta: parse + DataFusion plan + query) | **18–21** |

Method: spawn the compiled binary end-to-end, N=9 reps per command plus
one unmeasured warm-up, median reported; three same-day bench runs
shown as the range. Corpus: the seeded 1K synthetic store shared with
`perf::ready_1k_cold` (`tests/suite/synth.rs`), so these line up with
gate §7's budgets (`bench-baseline.json` had `ready_1k_cold` at 30ms
when these were taken).

## Reading

The review question (2026-08-08) was whether the felt slowness was an
engine constant or build times. Answer: the engine constant exists but
is small — ~21ms for parse+plan+query over 1K files, ~4× the 5ms
process floor and well inside the 100ms MW-C4 budget. Anything slower
that an operator feels is build time (`cargo run` recompiles) or store
growth, not DataFusion planning. The `.cache/tasks.jsonl` projection
stays deferrable at this scale.
