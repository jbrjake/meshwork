---
id: mw-tdv6h30
title: Run portfolio's shared CLAUDE.md check from the gate and retire the Rust copy
category: meta/claude-md
seq: 310
needs: [mw-9g064kv]
verify: "all(lacks tests/suite/arch.rs claude_md_names_every_shipped_artifact, contains verify_meshwork.sh claude_md_names_every_shipped_artifact)"
docs: [CLAUDE.md#the-rest-of-the-tree, verify_meshwork.sh]
status: open
created: 2026-09-30T13:51Z
---
Once portfolio ships the check (the ask this task needs), meshwork's gate
calls that script and `arch::claude_md_names_every_shipped_artifact` in
tests/suite/arch.rs goes, so the behavior has one implementation. The
CLAUDE.md Hard boundaries line that names the Rust test moves to naming the
script; same commit.

## log
- 2026-09-30T13:51Z created
