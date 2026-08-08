---
id: mw-n0r5jwm
title: "Cache key on content hash, not mtime"
category: core/perf
needs: [mw-4m169xc]
verify: cargo test cache::checkout_does_not_invalidate
seq: 240
docs:
  - FORMAT.md#projection
status: open
created: 2026-08-08T14:09Z
---
Review finding (2026-08-08). `git checkout` rewrites mtimes on every
branch switch, so a (count, bytes, max-mtime) key thrashes exactly in
the worktree-heavy workflow meshwork is designed for. Decide the key —
content hash, not mtime — before the `.cache/tasks.jsonl` projection
lands.

## log
- 2026-08-08T14:09Z created
