---
id: mw-g4a9
title: "Backfill docs: refs on the 15 seeded plan tasks (PLAN refs column was dropped in the 1.8 conversion)"
status: done
category: meta/store
verify: test -z "$(grep -L 'docs:' meshwork/tasks/mw-*.md)"
docs:
  - DESIGN-meshwork.md#2-task-file-format-normative-example
created: 2026-08-06
---
The 1.8 plan→task conversion kept titles and verifies but dropped each PLAN
row's MW/DESIGN refs. The format supports task→requirement linkage today
(docs: anchors, MW-F1); show --docs (4.1) and lint anchors (4.2) will consume
them. Copy each item's refs from the PLAN table into its task's docs: list.

## log
- 2026-08-06 created
- 2026-08-06 open→done — verify exit 0
