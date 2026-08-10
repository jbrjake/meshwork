---
id: mw-we7g0k3
title: "Install skill: adopt drops a repo-local ./meshwork shim"
category: skill
verify: grep -qi 'shim' .claude/skills/meshwork/references/install.md
docs:
  - FORMAT.md#store-layout
status: open
created: 2026-08-09T23:35Z
seq: 62
---
Field evidence (sazed, 2026-08-09). Every meshwork invocation across
every session re-derives the pinned path, usually as
`M=~/.meshwork/versions/$(cat .meshwork-version)/meshwork && $M …` —
boilerplate in every command of every transcript. Let adopt commit a
two-line `./meshwork` shim that execs the pinned version, so sessions
run `./meshwork <verb>`. Install-convention change in
references/install.md + adopt.md only; the frozen CLI surface is
untouched. The shim resolves `.meshwork-version` relative to itself, so
worktrees and subdirectory shells both work.

## log
- 2026-08-09T23:35Z created
