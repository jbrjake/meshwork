---
id: mw-04sbf3q
title: "adopt.md: hunt TODO/HANDOFF variants repo-wide, triage section prose, red-check verifies post-rotation"
status: open
category: skill
discovered-from: mw-9zrd
verify: grep -q 'red-check' .claude/skills/meshwork/references/adopt.md && grep -q 'between checkboxes' .claude/skills/meshwork/references/adopt.md
seq: 85
docs:
  - DESIGN-meshwork.md#§-10-migration
created: 2026-08-10T22:22Z
---
Three leras lessons, in ritual order. (1) Step 2 assumes a root TODO.md and
step 4 deletes HANDOFF.md — leras's handoff lived at docs/HANDOFF.md and the
migration prompt's premise missed it; open the ritual with a repo-wide hunt
(rg -l over TODO/HANDOFF/check-todo name variants) so the retirement list is
discovered, not assumed. (2) import absorbs everything between checkboxes
into the preceding task's body — leras's last checkbox preceded a 340-line
ledger that silently became one task's body; say plainly that
section-structured TODOs need a manual triage pass of every generated body.
(3) Verifies authored during the migration must be re-run against the staged
tree AFTER the archive rotation, and THROUGH sh -c — close's shell, where
agent-shell functions like rg do not exist — leras D03's acceptance grep of
docs/archive/ went green the moment the rotated finals (carrying the task's
own prose) landed, and 28 rg-based recasts exit 127 under close. The
red-check is the LAST step, not the first.

## log
- 2026-08-10T22:22Z created
