# HANDOFF (≤2KB — pointer, not narrative; durable state lives in PLAN Position + TRACE + commits)

**Done:** Through PLAN 0.4 (2026-08-04). CLI skeleton: `src/cli/mod.rs` (clap, global `--json`, envelope `{"v":1,"verb":…,"data":…}` via `emit_json` — MW-C3 shape fixed now, hardened at 0.8), main.rs is a 5-line shell. `init` (`src/cli/init.rs`): layout at git toplevel (found by `.git` walk-up, worktree-file tolerant), exact `.gitattributes`/`.cache/.gitignore` bytes, alias default = first 2 alnum chars of dir name (hand-edit invited in config comment), `default_author` seeded from `git config user.name`; refuses outside git and refuses re-init. TRACE MW-A3 → done.

**Decisions:** JSON envelope is `{"v":1,"verb","data"}`, one line, versioned with the binary. Gate §6 regex corrected to `[a-z_0-9]+::` (module names with digits — `e2e::` — didn't match; done-rows misread as citing no test).

**Open threads:** MW-J4 planned until --bless (0.8); fixtures.rs 510 (warn). e2e helpers (git_repo, meshwork cmd) live in e2e.rs — reuse for 0.5+.

**Next concrete step:** PLAN 0.5 — `add` (all flags incl. `--verify`) + `show` (last-3 comments, `… and N more`) (A5, D2, E4, K4).
verify: `cargo test e2e::add_show_roundtrip` exits 0.
