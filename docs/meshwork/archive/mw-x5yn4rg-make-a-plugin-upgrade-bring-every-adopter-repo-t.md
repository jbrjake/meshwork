---
id: mw-x5yn4rg
title: "Make a plugin upgrade bring the session's project — binary, pin and shim — to the loaded plugin's release, at user or project scope"
category: plugin/upgrade
seq: 130
relates: [mw-nd480zh]
docs: [.claude/skills/meshwork/references/install.md#the-shim-committed-what-sessions-actually-run, CLAUDE.md#hard-boundaries]
verify: "all(run cargo test package=meshwork target=suite e2e::plugin_upgrade_brings_adopter_current, contains .claude-plugin/plugin.json hooks, lacks .claude/skills/meshwork/references/install.md CLAUDE_CODE_BRIDGE_SESSION_ID)"
status: done
created: 2026-09-28T13:54Z
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
- 2026-09-30T14:56Z open→doing — claimed by claude (c25add50-6c7d-4332-bb50-42705a33eb64)
- 2026-09-30T15:13Z doing→done — verify exit 0 @ 7b3309b+19

## comments
- 2026-09-30T14:47Z [claude (05b075de-ed31-494c-b04a-af9f0dacd709)] Owner ruling 2026-09-30, in session: no release is cut until the task queue is empty, and the owner calls the cut. An agent never files or proposes a cut or release notes; the two filed today were dropped.
- 2026-09-30T15:02Z [claude (c25add50-6c7d-4332-bb50-42705a33eb64)] Owner ruling 2026-09-30, in session: the upgrade hook cannot rely on gh as a dependency. The binary fetch goes through curl against the release download URL; gh is not required anywhere in the mechanism.
