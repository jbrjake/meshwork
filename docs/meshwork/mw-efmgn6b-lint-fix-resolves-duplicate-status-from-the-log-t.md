---
id: mw-efmgn6b
title: "lint --fix resolves duplicate status: from the ## log tail"
category: core/store
relates: [mw-3wnhhvp]
verify: cargo test e2e::merge_union_poison_status_from_log
seq: 250
docs:
  - FORMAT.md#merge-semantics
status: open
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). Union-merge can leave two `status:` lines.
Picking arbitrarily silently loses a transition; the log has the answer
and is now a table (mw-3wnhhvp) — replay the log tail to derive the
true status.

## log
- 2026-08-08T14:09Z created
