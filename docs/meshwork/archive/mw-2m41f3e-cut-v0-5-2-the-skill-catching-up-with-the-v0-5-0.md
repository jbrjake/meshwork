---
id: mw-2m41f3e
title: "Cut v0.5.2 — the skill catching up with the v0.5.0 surface, and the build check that keeps it there"
category: meta/release
needs: [mw-dva9cb7]
verify: contains Cargo.toml /^version = "0.5.2"/
docs:
  - docs/release-notes/RELEASE-NOTES-v0.5.2.md#getting-it
status: done
created: 2026-09-25T14:18Z
---
Owner-gated: the owner revises and approves the notes in the transcript, then commit the notes and run scripts/cut-release.sh v0.5.2 (it refuses without the notes file), push main and the tag through the gate, watch the release workflow, and confirm the skill tarball carries references/spec.md and references/verifies.md.

## log
- 2026-09-25T14:18Z created
- 2026-09-25T14:18Z handoff by claude (192e60bd-9013-4a32-8fec-f25f918881a9)
- 2026-09-25T14:52Z open→done — verify exit 0 @ fe1dcd1
