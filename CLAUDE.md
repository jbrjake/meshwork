# meshwork

Task graph as markdown-with-frontmatter files in git, queried with DataFusion SQL, no database. Rust CLI, single binary. Built to replace TODO.md/HANDOFF.md friction across the portfolio.

## Doc map (read in this order when cold)

- `REQUIREMENTS-meshwork.md` — WHAT/WHY. `MW-*` IDs are normative; §3 non-goals is the anti-scope-creep fence (changing it requires an owner ruling).
- `DESIGN-meshwork.md` — HOW. File format §2, SQL contract §4–5, CLI surface §6 (frozen: anything not there is a non-goal), test architecture §13, gate §14, decisions §15.
- `PLAN-meshwork-build.md` — THE WORKLIST. Ordered items, each with a `verify:` command. The **Position** line at top marks the next item. Work top-to-bottom; no skipping ahead.
- `TRACE.md` — requirement → test map, machine-checked by the gate.

## Engineering baseline

`docs/CLAUDE-BASELINE.md` adopted whole (copied in; this repo must be self-contained). House numbers: 500-line target / 750 ceiling, 80% coverage floor, N≥7 bench reps, red-first TDD, evidence rule (no claim without observed exit 0, this session, this code).

**Explicit overrides, with reasons (baseline permits exactly this):**

1. **No TODO.md, no check-todo.sh.** `PLAN-meshwork-build.md` is the worklist until M1, when meshwork starts tracking itself (gate §8 self-host). Reason: this project exists to replace TODO.md; duplicating the plan into one would recreate the disease it cures. New work discovered mid-build: file it as a plan item with a `verify:` (pre-M1) or a meshwork task (post-M1) — never carry it in your head.
2. **Doc budgets are in bytes, not lines** (MW-D5 doctrine — line caps get gamed; this repo's own audit proved it). `docs/HANDOFF.md` ≤2KB. Code files still use the 500/750 line caps.
3. **check-perf.sh + bench-baseline.json arrive with the first perf-relevant code** (M2, gate §7) — the baseline's own rule, noted here so its absence isn't read as drift.

## Gates

- `./scripts/smoke.sh` — pre-commit (seconds): file caps, fmt, fast unit tests.
- `./verify_meshwork.sh` — THE gate (DESIGN §14, 8 sections, one exit 0). `scripts/regression.sh` (pre-push) delegates to it. `--strict` = v1 acceptance mode.
- Hooks are version-controlled in `.githooks/`; enable per clone: `git config core.hooksPath .githooks`.
- **Zero network in all tests and the whole gate** (MW-J6). Mirror tests use the stub `gh` in `tests/bin/`. A test that touches the network is a bug.

## Session ritual

1. Read the Position line in `PLAN-meshwork-build.md`; read that item and its MW/DESIGN refs.
2. Red first: the item's test precedes its code. Golden files change only via `--bless` + a reviewed diff.
3. An item closes only on its `verify:` exit 0 AND a green `./verify_meshwork.sh` — observed, not predicted.
4. Same commit: flip the item's TRACE.md rows `planned`→`done`, advance the Position line, update `docs/HANDOFF.md` (≤2KB).

## Hard boundaries

- The CLI surface is DESIGN §6, verbatim — `e2e::cli_surface_frozen` enforces it. Feature ideas go to REQUIREMENTS §3's rejection list by default.
- meshwork-the-tool never installs git hooks, never writes outside the repo (+ portfolio repo), never mutates GitHub beyond append (MW-A3/G1/H2). This repo's own dev hooks are unrelated to that rule.
- Never lower a threshold to pass a gate (baseline). Fix the code or the measurement.
