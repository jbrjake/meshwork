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
  docs/release-notes/RELEASE-NOTES-v0.5.0.md. It covers everything on main
  since v0.4.0 (git log v0.4.0..HEAD): the ask lifecycle and the asks
  verb, the derived projection and stats, the verify grammar's two new
  shapes, the view-backed lint findings, archive bundles, portfolio
  search, clause pins (cover, spec list, spec audit, spec-drift, prime's
  line), and the LIMIT fix; the DataFusion 54 bump is internal and left
  out on purpose. Read it against that log, land the prose, commit the
  notes, then `scripts/cut-release.sh v0.5.0` — it refuses a dirty tree,
  and README.md and docs/reveal-prep.md are uncommitted right now — then
  `git push origin main && git push origin v0.5.0`. This task's verify
  goes green when the script stamps Cargo.toml; the sazed ask retires
  itself when this closes. Everything else in ready waits on you:
  mw-ryd25rq is the prioritization ruling, mw-7cvse76 needs the flag's
  name and its §6 wording, mw-cvw8 is the mirror you deferred.
---

The v0.4.0 tag predates the verify grammar's `package=`/`target=` scoping tokens, the `lacks` predicate, the ask lifecycle flags, the derived projection, the `stats`, `asks` and `portfolio search` verbs, and the LIMIT fix; every pinned consumer is on 0.4.0 and sazed asked for the cut. The release is the owner's act: `scripts/cut-release.sh v0.5.0` stamps every stated version in lockstep, commits, and tags, and the tag push runs `release.yml`, which publishes the binaries, the skill tarball, and `docs/release-notes/RELEASE-NOTES-v0.5.0.md` as the release body.

Preconditions the script enforces: a clean working tree, no existing tag, and the notes file committed. The notes are owner-voiced (user impact first, plain English, no repo jargon); an agent drafts, the owner lands the prose.

## log
- 2026-09-22T14:06Z created
- 2026-09-22T14:06Z handoff by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
- 2026-09-22T14:48Z handoff by claude (25ed6f72-cf72-4c6f-9164-86d9af1be3d2)
