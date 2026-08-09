---
id: mw-hmg3f3d
title: "blocked-reason: drop iff to required-when-blocked"
category: core/format
verify: cargo test format::stale_blocked_reason_legal
docs:
  - FORMAT.md#task-file
status: open
created: 2026-08-09T23:17Z
---
Review finding (2026-08-09). The schema says `blocked-reason` is
"required non-empty iff `status: blocked`". The only-if half means a
task that was blocked, got unblocked, and kept its reason is a schema
violation → invalid row → loud in every listing. Almost certainly not
intended, and it makes a third-party writer fail stores a human would
call fine. Drop to "required when blocked"; a stale reason on a
non-blocked task is at most a lint warning, never invalid.

## log
- 2026-08-09T23:17Z created
