---
id: mw-n7594g6
title: Cut v0.5.3 — the plugin grants every meshwork verb
category: meta/release
seq: 106
needs: [mw-nfbj0vh]
verify: contains Cargo.toml /^version = "0.5.3"/
docs: [docs/release-notes/RELEASE-NOTES-v0.5.2.md#getting-it, CLAUDE.md#hard-boundaries]
status: open
created: 2026-09-30T14:42Z
---
Owner-gated: the owner revises and approves the notes in the transcript,
then commit the notes and run scripts/cut-release.sh v0.5.3 (it refuses
without the notes file), push main and the tag through the gate, watch the
release workflow, and confirm the skill tarball carries the granted
SKILL.md. Installed copies of the plugin pick up the grant only at this tag.

## log
- 2026-09-30T14:42Z created
