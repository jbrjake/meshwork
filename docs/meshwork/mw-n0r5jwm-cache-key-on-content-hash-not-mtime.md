---
id: mw-n0r5jwm
title: "Cache key on content hash, not mtime"
category: core/perf
needs: [mw-4m169xc]
verify: run cargo test cache::checkout_does_not_invalidate
seq: 240
docs:
  - FORMAT.md#projection
status: open
created: 2026-08-08T14:09Z
handoff: |
  Unblocked 2026-08-19: your blocker (gate §7 covers prime, mw-4m169xc)
  landed with numbers — prime at 1K comment-tailed tasks is 54ms cold,
  so
  the projection cache this task front-runs stays deferrable (evidence in
  docs/bench-startup.md and DESIGN §14 row 7). That reframes you: this is
  a DESIGN-DECISION task (pick and record the key shape), not an
  implementation one — FORMAT.md#projection reserves .cache/tasks.jsonl
  as never-authoritative, and nothing builds it yet. The verify names
  cache::checkout_does_not_invalidate — a key-derivation unit test is
  enough to pin the decision; don't build the cache to close this.
---
Review finding (2026-08-08). `git checkout` rewrites mtimes on every
branch switch, so a (count, bytes, max-mtime) key thrashes exactly in
the worktree-heavy workflow meshwork is designed for. Decide the key —
content hash, not mtime — before the `.cache/tasks.jsonl` projection
lands.

## log
- 2026-08-08T14:09Z created
