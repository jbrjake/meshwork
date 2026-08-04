# Engineering Baseline

Drop-in standards for a project's CLAUDE.md. Adopt whole; a project overrides a rule only explicitly, in its own CLAUDE.md, with a reason. Rust section at bottom; everything else language-agnostic (carry the spirit).

## Bootstrap

If missing, creating this scaffolding is your first TODO.md item, before feature work:

- `TODO.md` (root), `docs/HANDOFF.md`, `docs/archive/` + one-line-per-file `README.md` index.
- `.githooks/pre-commit` → `scripts/smoke.sh`; `.githooks/pre-push` → `scripts/regression.sh`. Enable per clone: `git config core.hooksPath .githooks`. Hooks version-controlled; skip loudly (not fail) without a toolchain.
- `scripts/check-file-length.sh` — fail >750 lines, warn >500 (generated files exempt).
- `scripts/check-todo.sh` — in smoke gate: open items have `verify:`, statuses legal, live file under cap.
- `scripts/check-perf.sh` + `bench-baseline.json` — with the FIRST perf-relevant code.

Gates announce whatever they skip or defer — no silent caps.

## Stewardship

Treat the project as yours; never excuse a defect as inherited. Fix bugs now or file them in TODO.md with context to act cold. Sand off rough edges as found. Leave every touched file cleaner. Fail closed: a check that can't run failed.

## Evidence

No claim without observed command output — an exit 0 you watched, this session, this code. Banned: "should work/pass", "likely fixed", "good to go" before gates ran, "CI in-flight" for "probably fine". "CI passed" = observed `conclusion=success`. Disclose any `--no-verify`. Assertion without evidence is the violation, not just wrong prediction.

## TODO.md — living worklist

Single source of truth for what's next. Markers: `[ ]` todo · `[~]` in progress · `[x]` done · `[!]` blocked (state blocker + unblock condition). `## Now` section on top. Item format:

```markdown
- [ ] **Imperative title** — context to act cold: why, where found.
      verify: `cargo test -p foo -- --exact parser::rejects_oversized_frame` exits 0
```

- Every item has a `verify:` command; its observed exit 0 is the only thing that closes the item. Can't write one → item isn't ready: split out a spike whose deliverable IS the verify. Decision items: verify = grep the recorded decision in its doc.
- Add items autonomously. Bug / rough edge / cut scope found mid-task → fix now if small, else file immediately with context. Never carry work only in your head or the conversation.
- Archive done/stale material to `docs/archive/TODO-YYYY-MM-DD-<slug>.md` (+ index line) when it accumulates or the file passes ~200 lines. Live = current, archive = history, never delete history.

## HANDOFF.md — session continuity

Read on resume; update before stopping. Sections: **Done** (with observed evidence) · **Decisions** (with the why — the part the next session can't reconstruct) · **Open threads** · **Next concrete step** (startable cold, with verify). Archive like TODO.md. Pause deliberately: at a stable point, write the handoff and stop at the natural break.

## Documentation

- CLAUDE.md is the index, not the manual: each rule once, depth pointed down into `docs/`. A growing explanation moves to a doc + one-line pointer.
- Doc updates ride the SAME commit: API change → its index/table; new file → routing table; flow change → flow map; moved content → fix inbound links. Stale docs actively mislead.
- Mechanical indexes (module/function tables) are GENERATED from planted anchors by script, regenerated in the gate. Hand-write only what code can't say: flow maps, rationale, "change X → touch Y" routing.
- Cite stable anchors, never line numbers (`file.rs:123` rots): symbol path, planted `// ANCHOR: name` comment, or heading link. Plant the anchor in the same change; validate anchors in the gate.
- Code and docs disagree → one is wrong; find out which.

## TDD & test depth

- Red first, always. No production code without a failing test behind it; the failing test is the spec.
- Substantial tests only: no tautologies, no "didn't panic", no mocking away the layer under test. Anti-vacuity: assert non-emptiness/cardinality before equality — empty-vs-empty must fail. Assertion messages: expected vs received. Test names are behavior sentences.
- Use all that apply: unit (inline) · integration (`tests/`, real interface) · property (`proptest`: round-trip, idempotence, ordering) · fuzz (`cargo-fuzz`; anything parsing bytes/untrusted input) · concurrency (stress, `loom`) · bench (`criterion`).
- Fixture tiers: Minimal · Realistic · Adversarial (malformed/huge/empty/unicode/hostile — mandatory for external input).
- Coverage: 80% floor; necessary not sufficient (synthetic 100% < real-input 80%); never lower a threshold — fix the measurement or the code.
- Flaky test = bug: `#[ignore]` + TODO item; never delete or retry-until-green. Tests deterministic: no unseeded randomness or wall-clock dependence.
- Bug process: (1) replicate with a reliably failing test — mocked pass ≠ replication, no fix before this; (2) root-cause by reading the path, not guessing; (3) minimal fix; (4) that test passes + full suite clean; (5) reporter says still broken → your test is wrong; back to 1 with production-like replication; (6) same fix-class twice = pattern → escalate to root cause.

## Gates

The gate is the wall, not every scratch build — mid-TDD may warn; nothing lands warning.

- Smoke (pre-commit, seconds): file-length, format check, fast unit tests `-D warnings`. No slow lint — fast on purpose. Test output is quiet on success (a single OK line + pass count) and prints only the failing tests on failure — a passing gate must never dump the full per-test log (the hook's stdout lands in whoever runs `git commit`).
- Regression (pre-push, minutes): full lint at max strictness + build + all tests `--include-ignored` + micro-benches with perf guard.
- Large-scale benchmarks: on demand + CI only, never per-push.
- `#[ignore]` marks a bounded slow test belonging to the regression tier, not a disabled test.
- CI is the backstop: hooks skip on bare machines, so once a remote exists a minimal CI job runs `regression.sh` per push/PR and merges require it.

## Performance — CPU and memory, from the outset

- Criterion benches + committed `bench-baseline.json` from the first perf-relevant code. The baseline is data, reviewed like code.
- N≥7 runs with warmup; record median + spread (max−min), never a lone mean. DNF is a recorded first-class failure (with mode), never silent, never "correct".
- Regression wall: >1.5× recorded baseline fails `check-perf.sh`. Crossing requires explicit user approval; improvements reseed freely (`UPDATE_BASELINE=1`).
- Memory gated identically: every bench records peak RSS (`/usr/bin/time -l` macOS, `-v` Linux, or getrusage) into the same baseline, same 1.5× wall. Fast-by-eating-memory didn't get fast.
- Comparative claims report resource-seconds — CPU·s and RAM·GB·s — not wall clock alone.
- Baselines are per-machine: fingerprint (host, CPU, cores, RAM) on every entry; gate against same-fingerprint only; a new machine seeds loudly, doesn't fail. Cross-machine = analysis, never a gate.
- Correctness precedes any perf claim — assert results in or beside the bench.
- Profile before optimizing: `cargo flamegraph`, tracing → Perfetto, `EXPLAIN ANALYZE`; `dhat-rs`/`heaptrack` on demand when RSS moves. The data decides.

## Reproducibility

Lockfiles committed; toolchain pinned; load-bearing deps pinned `=x.y.z` with the pin asserted in a gate. Data/fixture generation deterministic — two runs hash-equal; generators pinned by SHA; unstable upstream output snapshotted as fixtures. No baseline numbers without environment provenance.

## Commits & hygiene

Conventional commits, atomic — tests and doc updates ride along. No dead or commented-out code. No `TODO` comment without a linked TODO.md item. Comments explain why, not what; preserve design-rationale comments through refactors.

## Security floor

Secrets from environment only — never in code, config, logs, or errors. Parameterized queries only; shell-escape anything templated into a command, and test the escaping. Bound untrusted input (sizes/counts/depths) — anything attacker-influenced declares a ceiling. Pin CI actions by full SHA, never mutable tags.

## Rust — hard mode

- Lints: root `[workspace.lints]`, crates inherit `[lints] workspace = true`. Clippy `all` deny, `pedantic` warn; manifest keeps most lints `warn` so mid-TDD builds pass. Warnings-as-errors at the gates via `RUSTFLAGS=-D warnings`, `RUSTDOCFLAGS=-D warnings`, `cargo clippy -- -D warnings` — never `#![deny(warnings)]` in source. To silence: `#[allow]` at narrowest scope with written reason; never loosen workspace lints.
- `unsafe` allowed, never silent: deny `unsafe_op_in_unsafe_fn`, `missing_safety_doc`, `undocumented_unsafe_blocks`; every block carries `// SAFETY:` stating the invariant upheld.
- Errors: `thiserror` in libraries, `anyhow` at binary entry only. Variants say what, why, expected vs received. Propagate `?`/`.context()`; no `.unwrap()` in lib code; `.expect("reason")` for true invariants; raw unwraps in tests only.
- Types: newtypes for domain IDs; `pub(crate)` by default; `///` on all public items.
- Async: Tokio for all I/O; never block the runtime — `spawn_blocking` for CPU-heavy work; hot inner loops stay sync and allocation-free; bound concurrency (`Semaphore`); `Send + Sync` from the start; cancel-safe shutdown.
- Zero-copy where perf is the point: no needless memcpy or serialize-then-deserialize; borrow over clone; `&str` over `String` in signatures; every hot-path copy justifies its existence.
- Observability: `tracing` spans with structured fields, two tiers (always-on cheap; on-demand detailed); spans export to Perfetto as the profiling substrate.
- File size: 500-line target (repo median at/under), 750 hard ceiling, enforced pre-commit; over the line → split into a submodule dir.

```bash
./scripts/smoke.sh                # pre-commit gate
./scripts/regression.sh           # pre-push gate
cargo test -- --include-ignored   # incl. bounded-slow tier
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo bench                       # guarded by check-perf.sh
```
