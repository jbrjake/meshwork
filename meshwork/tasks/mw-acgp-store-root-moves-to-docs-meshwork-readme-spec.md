---
id: mw-acgp
title: Store root moves to docs/meshwork/ (README spec)
status: open
category: meta/readme
verify: cargo test e2e::store_at_docs_meshwork
seq: 220
docs:
  - DESIGN-meshwork.md#§-1-on-disk-layout
  - DESIGN-meshwork.md#§-5-canned-verbs-frozen-sql
created: 2026-08-06
---
README specs `docs/meshwork/<id>-<slug>.md` — flat, no tasks/ level shown.
Decide the full layout first (config.toml / attachments / .cache placement;
does tasks/ survive under docs/meshwork/). Code hard-codes meshwork/:
store.rs root probe, init, add's path echo; DESIGN §1 layout, §5 cross-repo
lookup `<repo-path>/meshwork/tasks/<id>-*.md`, §6 init row move with it.
Same change: git mv this repo's own store, update skill references +
CLAUDE.md, drop the README footnote's store exception. Consider landing
before mw-ntt5 so the sazed store never needs migrating.

## log
- 2026-08-06 created
