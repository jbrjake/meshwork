---
id: mw-q93qj46
title: "Teach the skill the asker's half of an ask — make the dependent work need the ask, and close the ask once its answer is done"
category: skill
docs: [.claude/skills/meshwork/SKILL.md#the-inbox-and-sibling-stores]
verify: "all(contains .claude/skills/meshwork/SKILL.md /--needs.*\\bask\\b/, contains .claude/skills/meshwork/SKILL.md /[Cc]lose.*\\bask\\b/)"
status: done
created: 2026-09-24T01:13Z
relates: [mw-tcb792a]
---
The skill's inbox section says how an ask surfaces in the other repo, but not what the asker does. An agent following it files the ask and keeps building the work that needed it, which is the failure an outside reviewer flagged on the README demo.

Two things are missing, and both already work in the binary:
- `dep add <dependent> --needs <ask>` takes the dependent out of `ready` until the ask is done; `why` then names the ask.
- The ask is a normal task in the asker's store. Its verify is the asker's own check, run in the asker's tree, and closing it is what unblocks the dependent. The answer's verify runs in the addressee's tree and only takes the ask out of their inbox.

Add this to the inbox section in a line or two, keeping the SKILL.md body lean. The README demo (mw-tcb792a) shows the full sequence.

## log
- 2026-09-24T01:13Z created
- 2026-09-25T14:06Z open→doing — claimed by claude (192e60bd-9013-4a32-8fec-f25f918881a9)
- 2026-09-25T14:18Z doing→done — verify exit 0 @ 90acfe3+4
