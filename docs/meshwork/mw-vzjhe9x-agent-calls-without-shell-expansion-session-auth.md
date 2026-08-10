---
id: mw-vzjhe9x
title: "Agent calls without shell expansion: session author resolves from the environment"
category: skill
relates: [mw-b9d4qpr]
verify: grep -q MESHWORK_AUTHOR .claude/settings.json
seq: 63
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
status: open
created: 2026-08-10T13:57Z
---
Owner report (2026-08-10, in-session): every agent-side
`start/comment/close --as "claude ($CLAUDE_CODE_BRIDGE_SESSION_ID)"`
trips a fresh Bash approval prompt — the env expansion inside the quoted
author string defeats approval generalization, so the human pays one
interrupt per verb, every session.

The MW-K1 chain already ends at $MESHWORK_AUTHOR before default_author
(no meshwork code change). What is missing is the once-per-session
export so agent calls carry no --as at all:

- this repo: SessionStart hook (or settings env) exports
  MESHWORK_AUTHOR="claude ($CLAUDE_CODE_BRIDGE_SESSION_ID)" before the
  first verb runs; verify greps for it in .claude/settings.json — move
  the verify if the export lands in a hook script instead.
- adopting repos: the ./meshwork shim (mw-we7g0k3) owns the same export;
  mw-b9d4qpr documents the rule in the skill.

Interim workaround (works today): expand the id once with echo and pass
the literal --as "claude (session_...)" — a literal command line lets
the approval generalize.

## log
- 2026-08-10T13:57Z created
