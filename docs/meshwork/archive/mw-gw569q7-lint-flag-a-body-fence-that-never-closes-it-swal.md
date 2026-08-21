---
id: mw-gw569q7
title: "lint: flag a body fence that never closes — it swallows every tail section"
category: core/hygiene
discovered-from: mw-nfv26ss
verify: run cargo test fence_unclosed_warn
seq: 55
status: done
created: 2026-08-21T20:30Z
---
Found repairing mw-nzeezr8: a body whose fenced code block never closes hides `## log`, `## comments`, and `handoff` content from every fence-aware reader — silently. The task stays queryable but its history reads as empty. The status-unlogged check catches the terminal case only; a live task with an open fence shows nothing at all.

Warn (`fence-unclosed`) when a task body reaches EOF inside an open fence. Not a --fix repair: only a human knows where the close belongs.

## log
- 2026-08-21T21:03Z open→done — verify exit 0 @ ac1f153+1
