---
id: mw-8kfqz2z
title: "SKILL.md: name the vacuous-verify traps — zero-match cargo filters, self-satisfied greps, piped exits"
status: done
category: skill
discovered-from: mw-9zrd
relates: [mw-175bn4c, mw-221f3jt]
verify: grep -qi 'vacuous' .claude/skills/meshwork/SKILL.md
seq: 90
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline
created: 2026-08-10T22:22Z
---
The doctrine line exists ("must FAIL while the work is undone") but names no
traps, and both live stores prove abstraction isn't enough: this store swept
39 of 48 open-task verifies green (2026-08-10), leras shipped 6 on day one.
Name the anti-patterns with their fail-closed idioms: (1) bare cargo test
FILTER — zero matching tests still exit 0; require observed passes, e.g.
out=$(cargo test F 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'.
(2) acceptance greps satisfiable by prose that already exists — including
the task file itself and rotated archives; scope to artifacts that cannot
pre-exist. (3) piped tails (cmd | grep -v noise) — the pipe's exit replaces
the gate's. (4) tool-environment mismatch — the authoring shell carries
functions the close-time sh does not: rg on this machine is a Claude-shell
zsh function, absent from PATH, so leras shipped 28 rg-recast verifies that
exit 127 under close forever; author against the toolset sh -c actually has
(grep, test, cargo). Close with the ritual sentence: run the verify THROUGH
sh -c at authoring time and watch it fail — exit 1, not 127.

## log
- 2026-08-10T22:22Z created
- 2026-08-14T13:08Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-14T13:10Z doing→done — verify exit 0 @ e043e2d+2
