---
id: mw-b9d4qpr
title: "Agent sessions comment under a distinct author with the session tag, automatically"
category: skill
relates: [mw-we7g0k3]
verify: grep -q 'MESHWORK_AUTHOR' .claude/skills/meshwork/references/install.md
docs:
  - DESIGN-meshwork.md#§-6-cli-surface
status: open
created: 2026-08-09T23:52Z
seq: 64
---
Owner ruling (2026-08-09): comments minted by an agent session must not
be attributed to the human default_author — four field-evidence comments
landed as [Jon Rubin] before this was caught (re-authored same day; safe
because nothing is mirrored yet — re-authoring changes the comment's
identity hash, so after M3 this kind of fix is append-only, not edit).

No meshwork code change: the MW-K1 chain already ends at
$MESHWORK_AUTHOR before default_author. Two landing spots:

- the ./meshwork shim (mw-we7g0k3) exports the fallback when the var is
  unset and a session is present — Claude Code exposes
  CLAUDE_CODE_BRIDGE_SESSION_ID=session_… in the Bash env, the same tag
  the Claude-Session commit trailer carries. Author string:
  claude (session_…). Never a ] in an author — the comment grammar
  closes on it.
- the skill states the rule for non-shim contexts: pass
  --as "claude (session_…)" explicitly; default_author is the human and
  belongs only to comments the human writes.

## log
- 2026-08-09T23:52Z created
