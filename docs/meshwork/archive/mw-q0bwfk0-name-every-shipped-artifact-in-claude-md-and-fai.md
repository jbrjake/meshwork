---
id: mw-q0bwfk0
title: "Name every shipped artifact in CLAUDE.md, and fail the build when one goes unnamed"
to: marasi
category: meta/claude-md
seq: 300
verify: grep -q claude_md_names_every_shipped_artifact ../marasi/CLAUDE.md
docs: [CLAUDE.md#the-rest-of-the-tree, CLAUDE.md#hard-boundaries]
status: dropped
created: 2026-09-30T13:45Z
---
meshwork's CLAUDE.md drifted from what the repo ships for months (the plugin
it vends went unnamed) because nothing tied the file to the tree.
`arch::claude_md_names_every_shipped_artifact` (meshwork, tests/suite/arch.rs)
now fails the build when a top-level tracked entry, the plugin manifest, the
skill directory or a workflow file is not named by path in CLAUDE.md, with a
short exemption list (`.gitignore`, `Cargo.lock`, `LICENSE`) justified in the
test's doc comment. The same drift is likely here.

The ask: name every top-level tracked entry, each CI workflow, and anything
this repo vends (plugin, skill, package) by path in CLAUDE.md, one line each
on what it is and how it is gated. Then add the equivalent check to this
repo's gate, a test or script named `claude_md_names_every_shipped_artifact`,
and name that check in CLAUDE.md so this ask's verify can see it. Pick the
exemptions deliberately and justify each: too narrow is the bug, too wide is a
file listing. Answer with a task carrying `answers: meshwork#<this id>`.

## log
- 2026-09-30T13:45Z created
- 2026-09-30T13:51Z open→dropped — Owner 2026-09-30, in session: the CLAUDE.md check is coded once in portfolio and applied per repo, not asked of each repo; one ask to portfolio replaces these eight
