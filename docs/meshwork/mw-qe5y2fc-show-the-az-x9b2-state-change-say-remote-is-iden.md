---
id: mw-qe5y2fc
title: Show the az-x9b2 state change; say remote is identity, not a fetch target
category: meta/readme
verify: grep -q 'not a fetch target' README.md
docs:
  - README.md
status: open
created: 2026-08-10T16:31Z
---
Two portfolio-section fixes. (1) `meshwork why az-x9b2` appears twice
with opposite outputs and no visible state change between the blocks —
the prose narrates with-checkout vs without, but under the
every-transcript-is-real banner it skims as an error. Show the state
change in the transcript (the beta checkout going away, or
repos.local.toml losing its path) so the pair replays honestly in
sequence; the transcript-replay guard executes fences in order and
diverges on this pair as written, which is why it needs this fix first.
(2) repos.toml carries remote= while the portfolio never clones, which
sits oddly next to "zero network required" — one clause saying remote is
identity (the cross-repo namespace and the future mirror's target), never
a fetch target, fixes it.

## log
- 2026-08-10T16:31Z created
