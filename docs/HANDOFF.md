# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Bootstrap complete, B0–B4 (2026-08-04). Corpus committed (DESIGN §13) + `fixtures::corpus_covers_features`. Stub `gh` at `tests/bin/gh`: records `$ argv` + `> stdin` lines to `$GH_STUB_CALLS` (required — exit 66 if a test didn't opt in), replays `tests/canned/<a>-<b>.json` (exit 64 if missing, `.exit` file overrides rc), and hard-refuses mutations — edit/close/delete/reopen/transfer/lock subcommands and non-GET/POST api methods exit 65 *after* recording (MW-H2 enforced at the boundary for the whole suite).

**Decisions:** DESIGN §15 + two corpus-derived: (1) `verify:` values ending in `::` must be YAML-quoted (trailing colon at EOL = mapping indicator); (2) doc anchors are heading slugs: `#§-budget-path` ↔ `## § budget path` (lowercase, spaces→dashes).

**Open threads:** TRACE MW-J4 `planned` until golden byte-compare + `--bless` land (0.8). `tests/suite/fixtures.rs` at 510 lines (warn) — split if corpus checks grow.

**Next concrete step:** PLAN 0.1 — task-file parser: strict serde model, frontmatter + `## log`/`## comments` tail sections, bullet+continuation comments; unknown keys warn; parse failure → `invalid` row with filename-recovered ID (A1, A6, I2, K1).
verify: `cargo test parse::` exits 0.
