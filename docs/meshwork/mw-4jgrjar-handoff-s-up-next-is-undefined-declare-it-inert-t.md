---
id: mw-4jgrjar
title: "handoff's 'up next' is undefined — declare it inert to the format"
category: core/format
verify: grep -q 'inert' FORMAT.md
docs:
  - FORMAT.md#task-file
status: open
created: 2026-08-09T23:17Z
---
Review finding (2026-08-09). `handoff` is "meaningful only while the
task is up next" without defining up-next. Per seq? Per ready ordering?
Per prime's pick? Three different answers, and a reader implementing
from this file can't render it. Say instead: the key is inert to the
format — carried, never interpreted; only `prime` (a consumer, DESIGN
§7b) decides what up-next means. One sentence, removes a judgment call
no third-party reader can make.

## log
- 2026-08-09T23:17Z created
