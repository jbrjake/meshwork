---
id: mw-nhjns62
title: Fail the build when something the repo ships goes unnamed in CLAUDE.md
category: meta/claude-md
seq: 120
docs: [CLAUDE.md#hard-boundaries]
verify: run cargo test package=meshwork target=suite arch::claude_md_names_every_shipped_artifact
status: open
created: 2026-09-28T13:45Z
---
This repo vended the Claude Code plugin for months and CLAUDE.md never said
so, so a session asked about the plugin searched settings files and the
install cache instead of `.claude-plugin/` and `.claude/skills/meshwork/`.
CLAUDE.md drifts because nothing ties it to what the repo ships. The Hard
boundaries line "This file evolves with the project" states the rule; this
task makes it mechanical.

The work:

- `arch::claude_md_names_every_shipped_artifact` in tests/suite/arch.rs:
  every top-level tracked entry of the repo (from `git ls-files`, a local
  read; exclude the store `docs/meshwork/` only if CLAUDE.md names it some
  other way) plus `.claude-plugin/plugin.json`, the skill directory, and
  each `.github/workflows/*.yml` must be named by path in CLAUDE.md. Each
  unnamed one fails with its path.
- Decide the exact set with the owner's rule in mind: a new artifact,
  distribution channel, gate or hook that a cold session would need to know
  about must fail the test until CLAUDE.md names it. Too narrow is the bug
  being fixed. Too wide and CLAUDE.md turns into a file listing, so pick
  the set, then justify it in the test's doc comment.
- Red first against today's CLAUDE.md, then name what is missing. Watch it
  fail on a deliberate break (add a scratch top-level file, see it named,
  remove it).
- The same drift exists in every repo under the code root. When this
  closes, file the equivalent ask to each adopter repo's store, not a
  cross-repo edit.

## log
- 2026-09-28T13:45Z created
