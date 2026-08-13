---
id: mw-dkwf26w
title: "lint: no-verify warning must cover doing tasks, not just open"
status: open
category: core/verify
verify: out=$(cargo test e2e::lint_doing_missing_verify 2>&1) && echo "$out" | grep -qE 'ok\. [1-9][0-9]* passed'
discovered-from: mw-ntt5
seq: 260
docs:
  - REQUIREMENTS-meshwork.md#§-e-lifecycle-discipline
created: 2026-08-07T13:47Z
---
Pilot evidence (sazed): 94 live tasks lacked verifies after import; lint
warned on the open ones but exempted all 8 `doing` tasks — the tasks
closest to being closed, where a missing definition of done matters
most. Related pressure: [[capture-before-verifiable-start-gates-on-verify-n]]
(mw-6wdpz1b) wants `start` to refuse open→doing while verify: is empty;
whatever lands there, lint's warning should still cover doing tasks that
predate the gate (imports create them directly).

## log
- 2026-08-07T13:47Z created
