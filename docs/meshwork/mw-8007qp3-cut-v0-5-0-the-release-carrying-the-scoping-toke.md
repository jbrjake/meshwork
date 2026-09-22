---
id: mw-8007qp3
title: "Cut v0.5.0 — the release carrying the scoping tokens, the ask lifecycle, the derived projection and the three new verbs"
status: open
category: meta/release
answers: sazed#sa-z0jc72t
verify: contains Cargo.toml /^version = "0.5.0"/
docs:
  - docs/release-notes/RELEASE-NOTES-v0.5.0.md
  - scripts/cut-release.sh
seq: 120
created: 2026-09-22T14:06Z
handoff: |
  The notes draft is on disk, uncommitted, for review:
  docs/release-notes/RELEASE-NOTES-v0.5.0.md — every section drawn from
  the feat/fix commits since v0.4.0 (git log v0.4.0..HEAD), user impact
  only, no task ids. Read it against that log; anything internal (analysis
  scripts, store chores, gate plumbing) was left out on purpose. Once the
  prose is yours, commit the notes, then `scripts/cut-release.sh v0.5.0`
  (it refuses a dirty tree — README.md and docs/reveal-prep.md are also
  uncommitted right now), then `git push origin main && git push origin
  v0.5.0`. The verify on this task goes green when the script stamps
  Cargo.toml. The sazed ask answers itself when this task closes; nothing
  to write in sazed's store.
---

The v0.4.0 tag predates the verify grammar's `package=`/`target=` scoping tokens, the `lacks` predicate, the ask lifecycle flags, the derived projection, the `stats`, `asks` and `portfolio search` verbs, and the LIMIT fix; every pinned consumer is on 0.4.0 and sazed asked for the cut. The release is the owner's act: `scripts/cut-release.sh v0.5.0` stamps every stated version in lockstep, commits, and tags, and the tag push runs `release.yml`, which publishes the binaries, the skill tarball, and `docs/release-notes/RELEASE-NOTES-v0.5.0.md` as the release body.

Preconditions the script enforces: a clean working tree, no existing tag, and the notes file committed. The notes are owner-voiced (user impact first, plain English, no repo jargon); an agent drafts, the owner lands the prose.

## log
- 2026-09-22T14:06Z created
- 2026-09-22T14:06Z handoff by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
