---
id: mw-1cmpywn
title: Have the SessionStart hook inject the skill body alongside prime
category: skill
needs: [mw-vffwacx]
seq: 560
verify: contains .claude/settings.json /skills/meshwork/SKILL.md/
docs:
  - docs/FIELD-STUDY-session-transcripts.md#§-3-recommendations
status: open
created: 2026-09-07T16:32Z
---
With R-B item B.6: the plugin's SessionStart hook and this repo's `.claude/settings.json` print
the skill body after `prime` (≈ 8 KB; the prime budget is untouched). Plugin-side change carried
through the install and migrate references so adopters pick it up on the next pin.

## log
- 2026-09-07T16:32Z created
