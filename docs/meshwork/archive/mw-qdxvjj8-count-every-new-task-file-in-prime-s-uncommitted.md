---
id: mw-qdxvjj8
title: "Count every new task file in prime's uncommitted-edit total, not one per untracked directory"
status: done
category: core/render
relates: [mw-xvgs08n]
verify: run cargo test prime_counts_each_untracked_task_file
docs:
  - docs/DESIGN-meshwork.md#7-session-integration-where-the-savings-land
created: 2026-10-02T16:20Z
---

prime's store line counts `git status --porcelain -- docs/meshwork` lines (src/cli/prime.rs). Without `--untracked-files=all`, git collapses a wholly untracked directory into one line, so a store that has never been committed reads `1 uncommitted task edit` however many task files `add` wrote. A first close does the same for the new `docs/meshwork/archive/` directory.

Observed running `./scripts/demo.sh`: two `add`s, then prime printed `store @ f5d8a02 · 1 uncommitted task edit`. The README's quick-start shows the same state with one task, so it reads right there only by luck.

The same call without `-uall` sits in `dirty_ids` (src/lint_verify.rs), which then misses every task file in an uncommitted store, and in close's `@ sha+N` dirty count (src/cli/close.rs). Fix all three. The test pins that two task files added to a store with no commits under `docs/meshwork/` count as two.

## log
- 2026-10-02T16:20Z created
- 2026-10-04T14:41Z open→doing — claimed by claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)
- 2026-10-04T14:55Z doing→done — verify exit 0 @ b6709d9+4
