---
id: mw-3t9qbxp
title: Stop import todo gluing column-zero prose onto a wrapped headline's title
category: core/import
docs: [docs/DESIGN-meshwork.md#10-migration-one-session-per-repo-mw-j3]
verify: run cargo test import_column_zero_prose_not_in_title
status: done
created: 2026-10-02T13:39Z
---
In a TODO.md where `- [ ] Wrapped headline that` / `  continues here` is followed directly (no blank line) by `Column-zero prose under Later that no item owns.`, import mints the title `Wrapped headline that continues here Column-zero prose under Later that no item owns.` (reproduced). Column-zero prose ends the headline. While here: references/adopt.md step 3 says import absorbs all prose between checkboxes into the preceding task's body — check it against what import does with column-zero prose and make the skill say what the binary does.

## log
- 2026-10-02T13:39Z created
- 2026-10-04T14:56Z open→doing — claimed by claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)
- 2026-10-04T15:01Z doing→done — verify exit 0 @ e978c43+5
