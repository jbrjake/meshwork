---
id: mw-9g064kv
title: "Code the CLAUDE.md shipped-artifact check once in portfolio, applied by every repo's gate"
to: portfolio
category: meta/claude-md
seq: 300
verify: grep -q claude_md_names_every_shipped_artifact ../portfolio/CLAUDE.md
docs: [CLAUDE.md#the-rest-of-the-tree, tests/suite/arch.rs]
status: open
created: 2026-09-30T13:51Z
---
Owner 2026-09-30, in session: not allowing things untracked in CLAUDE.md is
coded once in portfolio and then applied per repo, never re-solved per repo.

The check, as meshwork's `arch::claude_md_names_every_shipped_artifact`
(tests/suite/arch.rs) does it today: every top-level tracked entry from
`git ls-files`, each `.github/workflows/*.yml`, and anything the repo vends
(plugin manifest, skill directory) must appear in the repo's CLAUDE.md as a
whole path token; a short exemption list (`.gitignore`, `Cargo.lock`,
`LICENSE`) with a reason each; every unnamed path fails by name; a repo with
no CLAUDE.md fails outright. That test is the reference behavior and the
fixture for this ask.

The ask: one script in portfolio's `scripts/` (the `batch_edit.py` pattern:
canonical copy here, run from any repo root, no per-repo logic), named so
`claude_md_names_every_shipped_artifact` is its stable token, with the
exemption list as an optional per-repo argument or file rather than code. The
base CLAUDE.md gains the rule and names the script, so a cold session in any
repo under the code root finds it. Application is each repo's gate calling
the script; that lands per repo as its own commit, not from here.

## log
- 2026-09-30T13:51Z created
