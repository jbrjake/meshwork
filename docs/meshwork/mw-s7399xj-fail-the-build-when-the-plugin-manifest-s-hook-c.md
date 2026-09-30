---
id: mw-s7399xj
title: Fail the build when the plugin manifest's hook command names a file the plugin does not ship
status: open
category: plugin/upgrade
relates: [mw-gh067xt]
verify: "all(run cargo test package=meshwork target=suite arch::manifest_hook_paths_ship, contains CLAUDE.md /arch::manifest_hook_paths_ship/)"
docs:
  - CLAUDE.md#hard-boundaries
  - docs/DESIGN-meshwork.md#13-test-architecture--fixture-corpus-mw-j4j6
seq: 140
created: 2026-09-30T15:14Z
---

`.claude-plugin/plugin.json` registers the SessionStart hook as
`${CLAUDE_PLUGIN_ROOT}/hooks/session-start.sh` in exec form. Nothing checks
that the path resolves to an executable file the repo ships: `claude plugin
validate` checks component paths (`skills`, a `hooks` .json file) and never
a hook's `command`, and `arch::claude_md_names_every_shipped_artifact`
proves CLAUDE.md names `hooks/`, not that the manifest points into it. A
rename or a lost executable bit ships a dead hook — no upgrade, no prime in
any adopter — and the gate stays green.

The work:

- `arch::manifest_hook_paths_ship` in tests/suite/arch.rs: parse the
  manifest, walk every hook entry under `hooks` (every event, `command`
  and each `args` element), substitute `${CLAUDE_PLUGIN_ROOT}` with the
  repo root, and assert each path that names a file is a git-tracked
  executable. Zero hooks found is a failure, never a vacuous pass.
- Red first: watch it fail on a deliberate rename of the command in the
  manifest, then restore.
- Name the test in CLAUDE.md's "The plugin's hook is the upgrade"
  paragraph the way the other guards are named.

## log
- 2026-09-30T15:14Z created
