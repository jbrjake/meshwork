---
id: mw-f3zhd2j
title: "Cut v0.5.1 — the release answering sazed's exists-arm ask and carrying the two rough edges its first v0.5.0 session hit"
category: meta/release
seq: 140
needs: [mw-kzzhbpp]
answers: sazed#sa-evxy8rt
docs:
  - scripts/cut-release.sh
  - docs/release-notes/TEMPLATE.md
verify: contains Cargo.toml /^version = "0.5.1"/
status: open
created: 2026-09-23T18:32Z
handoff: |
  Everything but the cut is on main. The three fixes landed as one commit
  each
  with their tests and the DESIGN §6 rows (lint_verify.rs
  `missing_read_path` +
  `declares`; cli/lint.rs `explain_report`; cli/add_batch.rs
  `normalize_value` +
  `key_at_error`), the skill edits as one docs commit, and each closed on
  its
  verify after a green `./verify_meshwork.sh`.
  
  What is NOT committed, on purpose:
  `docs/release-notes/RELEASE-NOTES-v0.5.1.md`
  — drafted for the owner's review; the owner lands the prose. The order
  from
  here is the owner's: review and commit the notes (cut-release.sh refuses
  to
  tag without them), then `scripts/cut-release.sh v0.5.1` on a clean tree
  (stamps Cargo.toml, plugin.json and README in lockstep, re-blesses the
  golden
  version stamps, runs smoke, commits, tags), then
  `git push origin main && git push origin v0.5.1`. release.yml builds the
  binaries, packages the skill and publishes the notes file as the release
  body.
  Never hand-bump a version field.
  
  After the tag exists: sazed's ask (sazed#sa-evxy8rt) already shows this
  task as
  its answer in its prime; its own verify wants the pin past v0.5, so its
  next
  session bumps `.meshwork-version` and re-spells the 13 bare-`contains`
  deliverables with an `exists` arm — a comment on their task saying
  v0.5.1
  carries the rule is the courteous close. mw-59f0t1q (the reveal) was
  gated on
  the v0.5.0 cut and reads "the fixes the field study found in real use
  land
  before"; this release is those fixes.
---
The owner's act, once the notes read right: commit the notes, `scripts/cut-release.sh
v0.5.1` (clean tree; stamps Cargo.toml, plugin.json and README in lockstep, re-blesses
the golden version stamps, runs smoke, commits, tags), then push main and the tag.
`release.yml` builds the binaries, packages the skill and publishes the notes file as the
release body. sazed's ask closes on its own pin bump — its verify wants the pin past
v0.5, so the closing comment should say v0.5.1 carries the rule.

## log
- 2026-09-23T18:32Z created
- 2026-09-23T18:43Z handoff by claude (c86c9e3d-7fc8-4611-b368-47218aec1288)
