---
id: mw-2nmsys2
title: Surface dangling sequence.md entries as a lint finding
category: plan/m2
seq: 40
verify: cargo test e2e::portfolio_sequence_dangling
docs:
  - DESIGN-meshwork.md#§-9-portfolio-master-sequencing
status: done
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
- 2026-08-10T19:04Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-10T19:09Z doing→done — verify exit 0 @ 5ac63be+5

## comments
- 2026-08-10T19:06Z [claude (session_016iEafFdzwyKAtsU3AEMhaU)] Surface picked: the existing registry-aware lint pass under MESHWORK_PORTFOLIO — §9's ruled home for registry hygiene findings (mw-mrjccx2 precedent). No §6 delta, no new verb; the nod the body anticipated is only needed if a portfolio-verb surface (e.g. portfolio lint) is ever wanted instead. Severity warning, not error: a dangling entry degrades ordering (the overlay skips it), never readiness semantics. Absent repos stay the skipped-repo notice; done/dropped entries are satisfied — prune's business.
