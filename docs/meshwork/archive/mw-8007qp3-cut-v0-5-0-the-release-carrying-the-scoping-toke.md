---
id: mw-8007qp3
title: "Cut v0.5.0 — the release carrying the scoping tokens, the ask lifecycle, the derived projection and the three new verbs"
status: done
category: meta/release
answers: sazed#sa-z0jc72t
verify: contains Cargo.toml /^version = "0.5.0"/
docs:
  - docs/release-notes/RELEASE-NOTES-v0.5.0.md
  - scripts/cut-release.sh
seq: 120
created: 2026-09-22T14:06Z
---

The v0.4.0 tag predates the verify grammar's `package=`/`target=` scoping tokens, the `lacks` predicate, the ask lifecycle flags, the derived projection, the `stats`, `asks` and `portfolio search` verbs, and the LIMIT fix; every pinned consumer is on 0.4.0 and sazed asked for the cut. The release is the owner's act: `scripts/cut-release.sh v0.5.0` stamps every stated version in lockstep, commits, and tags, and the tag push runs `release.yml`, which publishes the binaries, the skill tarball, and `docs/release-notes/RELEASE-NOTES-v0.5.0.md` as the release body.

Preconditions the script enforces: a clean working tree, no existing tag, and the notes file committed. The notes are owner-voiced (user impact first, plain English, no repo jargon); an agent drafts, the owner lands the prose.

## log
- 2026-09-22T14:06Z created
- 2026-09-22T14:06Z handoff by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
- 2026-09-22T14:48Z handoff by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
- 2026-09-23T17:08Z open→done — verify exit 0 @ 3e3548c
