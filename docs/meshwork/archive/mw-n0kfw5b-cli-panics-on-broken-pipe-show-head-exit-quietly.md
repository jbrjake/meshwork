---
id: mw-n0kfw5b
title: CLI panics on broken pipe (show | head) — exit quietly on SIGPIPE
status: done
category: core/render
verify: run cargo test e2e::sigpipe_quiet
seq: 280
created: 2026-08-08T14:18Z
---

## log
- 2026-08-08T14:18Z created
- 2026-08-19T21:23Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-19T21:27Z doing→done — verify exit 0 @ 0d0b9cd+7
