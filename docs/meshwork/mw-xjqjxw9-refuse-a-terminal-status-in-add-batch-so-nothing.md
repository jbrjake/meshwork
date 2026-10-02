---
id: mw-xjqjxw9
title: Refuse a terminal status in add --batch so nothing closes without close
category: core/authoring
docs: [docs/DESIGN-meshwork.md#6-cli-surface-complete-for-v1--anything-not-here-is-a-non-goal]
verify: run cargo test batch_refuses_terminal_status
status: open
created: 2026-10-02T13:39Z
---
A batch document carrying `status: done` is accepted and written into the store root as a done task: no verify ran, no `→done` log line, and lint then reports `misplaced` and `status-unlogged` (reproduced in a scratch store). `close` is the only way to done; the batch should refuse `done`/`dropped` (and arguably any status but `open`) with the same "nothing written" atomicity as its other refusals.

## log
- 2026-10-02T13:39Z created
