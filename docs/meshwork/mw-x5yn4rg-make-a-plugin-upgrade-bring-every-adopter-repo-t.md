---
id: mw-x5yn4rg
title: "Make a plugin upgrade bring every adopter repo to the plugin's release at its next session — binary, pin and shim, with no per-repo step"
category: plugin/upgrade
seq: 130
relates: [mw-nd480zh]
docs: [.claude/skills/meshwork/references/install.md#the-shim-committed-what-sessions-actually-run, CLAUDE.md#hard-boundaries]
verify: "all(run cargo test package=meshwork target=suite e2e::plugin_upgrade_brings_adopter_current, contains .claude-plugin/plugin.json hooks, lacks .claude/skills/meshwork/references/install.md CLAUDE_CODE_BRIDGE_SESSION_ID)"
status: open
created: 2026-09-28T13:54Z
---

Owner ruling 2026-09-28, this session: upgrading the plugin is the whole
upgrade — "I upgrade the plugin and it magically works"; no project is
updated by hand, and no agent is told to rewrite anything.

The failure: each adopter's shim (`docs/meshwork/meshwork`) is text an
agent transcribed from install.md at adoption, and nothing rewrites it. A
pin bump moves `.meshwork-version` only: leras is on v0.5.2 and still runs
its August shim, which reads only the bridge session variable, so its CLI
sessions get no session author. Each release that changes the shim widens
the gap, because prose in a reference file is its only carrier.

The mechanism ships in the plugin, so the plugin upgrade delivers it:

- One canonical shim file in this repo, shipped with the plugin; install.md
  stops transcribing it, and the binary `include_str!`s the same file.
- A plugin SessionStart hook (`hooks` in plugin.json). In a repo with
  `docs/meshwork/` it fetches the plugin's release binary if absent,
  writes `.meshwork-version`, rewrites the shim byte-identical to
  canonical, and runs prime through it. prime's first line says what
  changed and tells the agent to commit it. A failed fetch leaves the repo
  as it was and says so loudly in that line.
- The plugin hook is the one prime injection: adopt.md drops the per-repo
  hook, and nothing double-injects where one remains.
- Confirm the hook contract (location, `${CLAUDE_PLUGIN_ROOT}`, output
  injection) against Claude Code's plugin docs first.

For the owner, not to decide here: "the plugin advances every pin" replaces
per-repo pin choice; a repo that must stay below the plugin needs an
opt-out the owner names.

The fetch lives in the hook script, not the binary. The test drives the
script with the stub `gh` in `tests/bin/` against fixture adopters and
asserts pin, shim bytes, fetched binary and prime's first line. Lands with
its CLAUDE.md line.

## log
- 2026-09-28T13:54Z created
