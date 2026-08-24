---
id: mw-h4s4gka
title: Cut the release that ships the migration path
category: meta/distribution
discovered-from: mw-nx91erh
needs: [mw-4r7v8vj]
seq: 252
verify: t=$(git tag --sort=-v:refname | head -1); git ls-tree -r --name-only "$t" | grep -q references/migrate.md && git merge-base --is-ancestor "$t" origin/main
status: open
created: 2026-08-24T13:32Z
handoff: |
  Everything the release carries is on main as of 33abaf3: migrate.md
  ritual (verify-recast step 6), re-voiced README, batch approve-at-mint,
  and now gate §9 (mw-5fekg2q, e361089) machine-proving every README
  transcript against a real run — the README this tag publishes is
  gate-checked, not hand-promised. Cut with scripts/cut-release.sh vX.Y.Z
  — never hand-bump — then push main + the tag; release.yml builds
  binaries + skill tarball, marketplace serves the tag immediately.
  Suggest v0.4.0: the batch trust change is behavior, not a patch. After
  the push, the sazed/leras sweeps (mw-rtt16df, mw-mcx59sd) unblock onto
  this tag. NOTE: an owner-requested verify-DSL section is drafted
  UNCOMMITTED in README.md (2026-08-24) awaiting his review — land or
  drop it before cutting so the tag's README is deliberate; cut-release.sh
  adds by pathspec, so the draft can't slip into its commit.
---
Publishes the migration ritual landed on mw-nx91erh: marketplace installs
resolve the newest tag, so cutting this release IS the skill publish.
Mechanics are scripts/cut-release.sh vX.Y.Z — never hand-bump; the script
stamps Cargo.toml, plugin.json, and the goldens in lockstep, commits, and
tags — then git push origin main && git push origin <tag>; release.yml
builds the binaries and the skill tarball from the tag. If the tag ever
needs re-pushing, the GitHub release flips to draft: gh release edit
<tag> --draft=false.

The dep on mw-4r7v8vj is deliberate: the branch push publishes README@HEAD,
and the owner ruled 2026-08-24 that its trust-gate transcripts are
invalidated by approve-at-mint. Cutting before the re-voice publishes
transcripts known to be impossible. If the owner wants the release sooner,
dep rm and cut.

The two migration sweeps filed alongside need this tag: their pin bumps
must land on a release that carries references/migrate.md.

## log
- 2026-08-24T13:32Z created
