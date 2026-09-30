---
id: mw-nfbj0vh
title: "Draft the v0.5.3 release notes for the owner's review — the plugin upgrade is the whole upgrade, and every verb is granted"
category: meta/release
seq: 200
verify: exists docs/release-notes/RELEASE-NOTES-v0.5.3.md
docs: [docs/release-notes/RELEASE-NOTES-v0.5.2.md#getting-it, .claude/skills/meshwork/SKILL.md#session-ritual]
status: dropped
created: 2026-09-30T14:42Z
needs: [mw-x5yn4rg, mw-gh067xt]
---

What changes for a user in v0.5.3, in this order:

- Upgrading the plugin is the whole upgrade (mw-x5yn4rg): the plugin's
  SessionStart hook brings the session's project to the plugin's release,
  fetching the binary, writing the pin and the canonical shim, and prime's
  first line says what changed. No project is updated by hand, at user or
  project scope alike.
- Every meshwork verb runs without an approval prompt in a turn that loaded
  the skill (mw-nd480zh), and the build fails if a verb ships without its
  grant (mw-gh067xt). Loading the skill itself asks once in a default-mode
  session; "don't ask again" keeps it quiet for that project.
- The stats placement note no longer promises bands (mw-8m9kr6z).

Plain English, user impact only, no repo jargon. Draft in
docs/release-notes/RELEASE-NOTES-v0.5.3.md in the v0.5.2 notes' shape and
propose it; the owner revises and approves in the transcript. This task
waits on the upgrade and the grant check landing, so the notes describe what
ships, not what is planned.

## log
- 2026-09-30T14:42Z created
- 2026-09-30T14:47Z open→dropped — Owner ruling 2026-09-30, in session: no release is cut until the task queue is empty, and the owner calls the cut; an agent never files or proposes one. This task was filed off an agent-written line in a closed task, which is not an instruction.
