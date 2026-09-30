---
id: mw-x5yn4rg
title: "Make a plugin upgrade bring the session's project — binary, pin and shim — to the loaded plugin's release, at user or project scope"
category: plugin/upgrade
seq: 130
relates: [mw-nd480zh]
docs: [.claude/skills/meshwork/references/install.md#the-shim-committed-what-sessions-actually-run, CLAUDE.md#hard-boundaries]
verify: "all(run cargo test package=meshwork target=suite e2e::plugin_upgrade_brings_adopter_current, contains .claude-plugin/plugin.json hooks, lacks .claude/skills/meshwork/references/install.md CLAUDE_CODE_BRIDGE_SESSION_ID)"
status: open
created: 2026-09-28T13:54Z
handoff: |
  Owner ruling 2026-09-30, in session: no release is cut until the task
  queue is empty, and the owner calls the cut in the transcript. Do not
  file, propose or sequence a cut or release notes; the two filed today
  were dropped. Work the queue.
  
  Hook contract, confirmed 2026-09-30 against the raw docs
  (code.claude.com/docs/en/plugins/manifest-reference.md and hooks.md), so
  this task can start at the design:
  
  - `hooks` in `.claude-plugin/plugin.json` takes a `.json` file path, an
  inline hooks object in the same shape as settings-file hooks, or an
  array mixing both; whatever it declares merges with `hooks/hooks.json`
  at the plugin root when that file exists. Every component path is
  relative to the plugin root; `${CLAUDE_PLUGIN_ROOT}` is substituted in
  hook commands (use exec form with `args`, or double-quote it in a
  shell-form command; `claude plugin validate` warns otherwise).
  `${CLAUDE_PLUGIN_DATA}` is the plugin's persistent data dir.
  - Plugin hooks register at session start, before any skill loads, in
  every install scope. The SessionStart hook's stdout lands in context
  like this repo's `.claude/settings.json` hook does today (the one prime
  injection this task moves into the plugin).
  - A hook command runs with the project as cwd and `CLAUDE_PROJECT_DIR`
  set; exit 2 blocks on blocking events; JSON on stdout is honored per
  event.
  
  Where the pieces live now: the shim text is transcribed in
  `.claude/skills/meshwork/references/install.md` under "The shim,
  committed"; the per-repo SessionStart hook is `.claude/settings.json`
  here and in adopt.md's install step; `.meshwork-version` is the pin; the
  stub `gh` is `tests/bin/gh`; `scripts/cut-release.sh` stamps versions.
  The test the verify names, `e2e::plugin_upgrade_brings_adopter_current`,
  does not exist yet; red-check will fail on it and on `contains
  .claude-plugin/plugin.json hooks`.
  
  Proven: the full gate is green at HEAD;
  `arch::claude_md_names_every_shipped_artifact` requires a CLAUDE.md line
  for anything new at the top level, so a `hooks/` directory or a
  canonical shim file lands with its CLAUDE.md line in the same commit.
  Grants are verified in an owner-run interactive session, never with
  `claude -p`.
---

Owner ruling 2026-09-28, this session: upgrading the plugin is the whole
upgrade — "I upgrade the plugin and it magically works"; no project is
updated by hand, and no agent is told to rewrite anything. The plugin may
be installed at user or project scope, so projects can run different
meshwork versions; the design assumes neither scope. The plugin version a
session loads is that project's release.

The failure: each adopter's shim (`docs/meshwork/meshwork`) is text an
agent transcribed from install.md at adoption, and nothing rewrites it. A
pin bump moves `.meshwork-version` only: leras is on v0.5.2 with its
August shim, so its CLI sessions get no session author.

The mechanism ships in the plugin and acts only on the session's project:

- One canonical shim file in this repo, shipped with the plugin; install.md
  stops transcribing it, and the binary `include_str!`s the same file.
- A plugin SessionStart hook (`hooks` in plugin.json). It reads the
  loaded plugin's version from `${CLAUDE_PLUGIN_ROOT}`; when the session's
  project has `docs/meshwork/` and is behind that version, it fetches the
  binary if absent, writes `.meshwork-version`, rewrites the shim
  byte-identical to canonical, and runs prime through it. prime's first
  line says what changed and tells the agent to commit it. A failed fetch
  leaves the project as it was and says so loudly in that line.
- The plugin hook is the one prime injection: adopt.md drops the per-repo
  hook, and nothing double-injects where one remains.
- install.md and adopt.md teach both install scopes (the loaded plugin's
  version is the project's pin); install.md stops presenting user scope as
  the one way.
- Confirm the hook contract against Claude Code's plugin docs first.

The binary stays network-free. The test drives the hook with the stub `gh` in `tests/bin/` against fixture adopters and
asserts pin, shim bytes, fetched binary and prime's first line. Lands with
its CLAUDE.md line.

## log
- 2026-09-28T13:54Z created
- 2026-09-30T13:49Z handoff by claude (05b075de-ed31-494c-b04a-af9f0dacd709)
- 2026-09-30T14:48Z handoff by claude (05b075de-ed31-494c-b04a-af9f0dacd709)

## comments
- 2026-09-30T14:47Z [claude (05b075de-ed31-494c-b04a-af9f0dacd709)] Owner ruling 2026-09-30, in session: no release is cut until the task queue is empty, and the owner calls the cut. An agent never files or proposes a cut or release notes; the two filed today were dropped.
