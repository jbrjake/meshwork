---
id: mw-2kk6303
title: "Start a CLAUDE.md that names every shipped artifact, and fail the build when one goes unnamed"
to: wyndam
category: meta/claude-md
seq: 300
verify: grep -q claude_md_names_every_shipped_artifact ../wyndam/CLAUDE.md
docs: [CLAUDE.md#the-rest-of-the-tree, CLAUDE.md#hard-boundaries]
status: open
created: 2026-09-30T13:45Z
---
meshwork's CLAUDE.md drifted from what the repo ships for months (the plugin
it vends went unnamed) because nothing tied the file to the tree.
`arch::claude_md_names_every_shipped_artifact` (meshwork, tests/suite/arch.rs)
now fails the build when a top-level tracked entry, the plugin manifest, the
skill directory or a workflow file is not named by path in CLAUDE.md, with a
short exemption list (`.gitignore`, `Cargo.lock`, `LICENSE`) justified in the
test's doc comment. wyndam has no CLAUDE.md at all, so a cold session here
has no map; the answer starts one.

The ask: write CLAUDE.md naming every top-level tracked entry, each CI
workflow, and anything this repo vends (plugin, skill, package) by path, one
line each on what it is and how it is gated. Then add the equivalent check to
this repo's gate, a test or script named
`claude_md_names_every_shipped_artifact`, and name that check in CLAUDE.md so
this ask's verify can see it. Pick the exemptions deliberately and justify
each: too narrow is the bug, too wide is a file listing. Answer with a task
carrying `answers: meshwork#<this id>`.

## log
- 2026-09-30T13:45Z created
