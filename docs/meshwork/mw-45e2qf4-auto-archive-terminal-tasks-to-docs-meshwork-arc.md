---
id: mw-45e2qf4
title: Auto-archive terminal tasks to docs/meshwork/archive/
status: doing
category: core/store
verify: cargo test e2e::archive_on_close
docs:
  - DESIGN-meshwork.md#§-1-on-disk-layout
  - REQUIREMENTS-meshwork.md#§-a-store # MW-A1/A4
seq: 17
created: 2026-08-06
---
Owner-requested 2026-08-06: docs/meshwork/ gets messy as tickets pile
up — terminal tasks (done/dropped) move to docs/meshwork/archive/
automagically. OWNER-CONFIRMED constraint: archived tasks stay fully
queryable — the loader reads archive/ every invocation, so tables,
needs-resolution (a dep on an archived done task counts as met), and
prime's recently-done are location-blind; ONLY the file path changes.
Mechanics: close/drop move the file, reopen moves it back, import
routes already-terminal tasks straight to archive/, lint warns on
misplaced files (terminal in root, live in archive) and --fix moves
them. IDs stay unique across root+archive (mint collision-checks both
— MW-A4 never-reused). .gitattributes gains /archive/*.md merge=union.
No new verb — the frozen surface is untouched; automagic rides the
existing lifecycle verbs.

## log
- 2026-08-06 created
- 2026-08-06 open→doing
