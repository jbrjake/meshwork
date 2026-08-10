---
id: mw-17hnhzk
title: "import todo: nested checkboxes silently dropped — child tasks or a loud refusal"
status: open
category: core/import
verify: cargo test e2e::import_nested_checkboxes
discovered-from: mw-ntt5
seq: 120
docs:
  - DESIGN-meshwork.md#§-10-migration
created: 2026-08-07T13:47Z
handoff: |
  You are next by the M2 ordering constraint: this must land before leras
  migrates (2.6 is blocked on you). Import parser is src/cli/import.rs.
  Decide: child tasks via parent: (B8 supports arbitrary depth) or a loud
  refusal listing the dropped lines — silent loss is the one forbidden
  outcome. Red-first; goldens for import live in
  fixtures/golden/import-todo.md.
---
Pilot evidence (sazed, 2026-08-07, abe358b): TODO.md had 124 checkboxes,
109 top-level + 15 indented. `import todo` took exactly the 109 and folded
the 15 nested ones into parent bodies as prose — exit 0, count plausible,
no warning. 13 dropped items were OPEN, including the file's single
highest-flagged item; 3 sat inside a parent imported as done and
auto-archived — open work entombed in a closed task, the exact failure the
store exists to prevent. Undetectable without diffing the source. Fix:
import nested items as `parent:` children of the enclosing task (status
from their own checkbox, not the parent's), or refuse the file loudly
naming the nested lines. Never fold open work into prose with exit 0.
Must land before the leras migration (M2) — same import path, unattended.

## log
- 2026-08-07T13:47Z created
