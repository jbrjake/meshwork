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
---
The owner's act, once the notes read right: commit the notes, `scripts/cut-release.sh
v0.5.1` (clean tree; stamps Cargo.toml, plugin.json and README in lockstep, re-blesses
the golden version stamps, runs smoke, commits, tags), then push main and the tag.
`release.yml` builds the binaries, packages the skill and publishes the notes file as the
release body. sazed's ask closes on its own pin bump — its verify wants the pin past
v0.5, so the closing comment should say v0.5.1 carries the rule.

## log
- 2026-09-23T18:32Z created
