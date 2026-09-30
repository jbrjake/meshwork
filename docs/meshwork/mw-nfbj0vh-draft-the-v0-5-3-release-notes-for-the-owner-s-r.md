---
id: mw-nfbj0vh
title: Draft the v0.5.3 release notes for the owner's review — the plugin grants every meshwork verb
category: meta/release
seq: 105
verify: exists docs/release-notes/RELEASE-NOTES-v0.5.3.md
docs: [docs/release-notes/RELEASE-NOTES-v0.5.2.md#getting-it, .claude/skills/meshwork/SKILL.md#session-ritual]
status: open
created: 2026-09-30T14:42Z
---
What changed for a user since v0.5.2: the plugin's skill now carries a
permission grant for every meshwork verb, so an agent that has loaded the
skill runs `docs/meshwork/meshwork <verb>` without an approval prompt in
the turn that loaded it; loading the skill itself asks once in a default-mode
session, and "don't ask again" keeps it quiet for that project. The stats
placement note no longer promises bands. Plain English, user impact only, no
repo jargon; draft in docs/release-notes/RELEASE-NOTES-v0.5.3.md in the
v0.5.2 notes' shape and propose it; the owner revises and approves in the
transcript.

## log
- 2026-09-30T14:42Z created
