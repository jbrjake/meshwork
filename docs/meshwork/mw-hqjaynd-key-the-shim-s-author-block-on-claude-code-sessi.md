---
id: mw-hqjaynd
title: Key the shim's author block on CLAUDE_CODE_SESSION_ID as well as the bridge variable
category: skill
answers: oreseur#or-9c7n6hp
seq: 230
verify: contains docs/meshwork/meshwork /CLAUDE_CODE_SESSION_ID/
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-2-5-defects
status: open
created: 2026-09-07T16:32Z
---
CLI sessions export `CLAUDE_CODE_SESSION_ID` and not `CLAUDE_CODE_BRIDGE_SESSION_ID`, so the
shim falls back to `default_author` and agent work is stamped as the human in every sibling.
Update this repo's committed shim, the plugin's `references/install.md` shim text, and the
migrate ritual so adopters re-copy it; the author string keeps the `claude (<id>)` shape and the
no-`]` rule.

## log
- 2026-09-07T16:32Z created
