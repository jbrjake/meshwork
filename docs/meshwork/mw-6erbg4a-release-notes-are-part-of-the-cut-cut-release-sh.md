---
id: mw-6erbg4a
title: "Release notes are part of the cut — cut-release.sh requires the notes file, release.yml publishes it"
status: open
category: meta/distribution
discovered-from: mw-h4s4gka
verify: "all(contains scripts/cut-release.sh release-notes, contains .github/workflows/release.yml notes-file)"
created: 2026-08-25T21:28Z
---

cut-release.sh exists so a release needs no memory, but the release body is the one artifact still left to memory: release.yml hardcodes the install boilerplate as --notes, and the narrative notes (v0.2.x as docs/release-notes/ files, v0.3.x pasted straight into the GitHub release) happened only when someone remembered. v0.4.0 shipped with boilerplate only — owner flagged it 2026-08-25.

Fix, two halves:
1. cut-release.sh refuses to cut when docs/release-notes/RELEASE-NOTES-$TAG.md is absent — the notes draft becomes a precondition of the tag, same standing as the version stamps.
2. release.yml creates the release with --notes-file pointing at that file (the install/pin line moves into the notes template footer so nothing is lost).

Content standard (owner, 2026-08-25): notes lead with user impact — new features, UX-affecting fixes; not internal refactors or store chores; no internal ids. Notes are owner-voiced: an agent drafts and proposes before the cut, the owner lands the prose.

## log
- 2026-08-25T21:28Z created
