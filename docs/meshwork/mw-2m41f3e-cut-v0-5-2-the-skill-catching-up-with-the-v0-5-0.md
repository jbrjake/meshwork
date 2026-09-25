---
id: mw-2m41f3e
title: "Cut v0.5.2 — the skill catching up with the v0.5.0 surface, and the build check that keeps it there"
category: meta/release
needs: [mw-dva9cb7]
verify: contains Cargo.toml /^version = "0.5.2"/
docs:
  - docs/release-notes/RELEASE-NOTES-v0.5.2.md#getting-it
status: open
created: 2026-09-25T14:18Z
handoff: |
  The v0.5.2 draft is at docs/release-notes/RELEASE-NOTES-v0.5.2.md,
  uncommitted
  on purpose: the owner revises it. Everything else is committed and gated
  green:
  4d232d4 (SKILL.md, references/spec.md, references/verifies.md) and
  90acfe3
  (arch::skill_names_every_verb_and_flag, the CLAUDE.md hard boundary).
  Nothing
  is pushed. Once the owner approves the notes in the transcript: commit
  the
  notes file alone, scripts/cut-release.sh v0.5.2, push main and the tag
  (the
  pre-push gate runs ~4 min), watch release.yml, then check the skill
  tarball
  lists references/spec.md and references/verifies.md. The binary is
  unchanged,
  so no adopter needs to move its .meshwork-version pin; the plugin update
  is
  what delivers the skill.
---
Owner-gated: the owner revises and approves the notes in the transcript, then commit the notes and run scripts/cut-release.sh v0.5.2 (it refuses without the notes file), push main and the tag through the gate, watch the release workflow, and confirm the skill tarball carries references/spec.md and references/verifies.md.

## log
- 2026-09-25T14:18Z created
- 2026-09-25T14:18Z handoff by claude (192e60bd-9013-4a32-8fec-f25f918881a9)
