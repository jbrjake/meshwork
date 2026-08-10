---
id: mw-2nmsys2
title: Surface dangling sequence.md entries as a lint finding
category: plan/m2
seq: 40
verify: cargo test e2e::portfolio_sequence_dangling
docs:
  - DESIGN-meshwork.md#§-9-portfolio-master-sequencing
status: open
created: 2026-08-10T16:31Z
---
sequence.md is authored, denormalized, cross-repo state: hand-maintained
repo#id bullets that must stay coherent with 20 repos' worth of lifecycle
it cannot observe. A typo'd or deleted id is the dangling-edge class lint
already catches inside a repo; in the overlay it is silent today ("first
ready one wins" just skips it). Distinguish three cases: id resolves
nowhere in a registered, present repo (dangling — the finding); repo
absent from disk (unresolvable — the existing skipped-repo notice, not a
finding); entry resolves to done/dropped (satisfied — prune's business,
not an error). Where it runs (lint inside the portfolio repo vs the
portfolio verb family) touches §6 — needs its own nod before the surface
is picked.

## log
- 2026-08-10T16:31Z created
