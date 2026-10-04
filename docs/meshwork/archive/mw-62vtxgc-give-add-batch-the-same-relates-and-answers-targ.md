---
id: mw-62vtxgc
title: Give add --batch the same relates and answers target checks as add
category: core/authoring
docs: [docs/DESIGN-meshwork.md#6-cli-surface-complete-for-v1--anything-not-here-is-a-non-goal]
verify: run cargo test batch_checks_relates_and_answers_targets
status: done
created: 2026-10-02T13:39Z
---
`add --relates pr-zzzzzzz` refuses a same-repo target that does not exist; a batch document with `relates: [pr-zzzzzzz]` is written without a word (reproduced). Likewise `add --answers <gid>` warns when the repo is unregistered and the batch says nothing. `add_batch.rs` checks only `needs`, `parent` and `discovered-from` targets; DESIGN §6 says the batch runs the same checks as `add`.

## log
- 2026-10-02T13:39Z created
- 2026-10-04T14:56Z open→doing — claimed by claude (70b8c3c2-a3ae-4cee-81aa-681b84b44405)
- 2026-10-04T15:01Z doing→done — verify exit 0 @ e978c43+4
