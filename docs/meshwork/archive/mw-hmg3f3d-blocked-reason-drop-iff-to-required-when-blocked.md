---
id: mw-hmg3f3d
title: "blocked-reason: drop iff to required-when-blocked"
category: core/format
verify: run cargo test format::stale_blocked_reason_legal
docs:
  - FORMAT.md#task-file
status: done
created: 2026-08-09T23:17Z
seq: 60
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
- 2026-08-21T21:03Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-21T21:06Z doing→done — verify exit 0 @ 52bbf88+2
