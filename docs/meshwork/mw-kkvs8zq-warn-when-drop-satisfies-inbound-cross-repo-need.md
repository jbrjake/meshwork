---
id: mw-kkvs8zq
title: Warn when drop satisfies inbound cross-repo needs
category: plan/m2
seq: 60
verify: cargo test e2e::drop_inbound_cross_repo_warns
docs:
  - DESIGN-meshwork.md#§-9-portfolio-master-sequencing
  - DESIGN-meshwork.md#§-12b-trust-boundary
status: open
created: 2026-08-10T16:31Z
---
Only done/dropped satisfies a dependency, and drop crosses a trust
boundary that done does not: inside one repo, needs cleared by a drop is
odd but survivable — same person, same head. Across repos, whoever drops
beta#bz-c0r3 silently unblocks alpha#az-x9b2 with no visibility on either
side that alpha depended on it; the thing that was needed never happened
and alpha's queue now says go. On drop, when portfolio context is
resolvable, scan registered present repos for inbound cross-repo needs on
the dropped id and print each as a warning (repo#id, one per line; absent
checkouts noted as unscanned). A warning is behavior, no new surface.
Refusing without a flag is the stronger answer and a §6 question — take
the ruling to the owner before implementing refusal.

## log
- 2026-08-10T16:31Z created
