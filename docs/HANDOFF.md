# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 1.4 (2026-08-04). `comment <id> [--as] "text"`: author chain `--as` → `MESHWORK_AUTHOR` → config `default_author` → loud error; authors containing `]`/newline refused (they delimit the format); multi-line text becomes two-space continuations. `attach <id> <path>`: copies into `meshwork/attachments/<id>/`, records via new `edit::set_list` (replaces a key AND its indented block with a safe inline list — block-style corpora survive), `--force` gates overwrite, >1MB prints the excerpt-first note and lint warns (K3). TRACE: K1, K2 done (20 planned).

**Decisions:** attachments/docs lists are written inline-style by tools (block style still parses fine — set_list migrates on first edit).

**Open threads:** none new.

**Next concrete step:** PLAN 1.5 — `prime`: byte-budgeted ≤6KB digest measured on the kitchen-sink fixture (D3, D5): ready top-10 one-liners · in-progress with last log line · blocked with reasons · counts.
verify: `cargo test e2e::prime_budget` exits 0.
