---
id: az-f1nd
title: Fix flaky governor test
status: open
category: engine/spill
labels: [bug]
discovered-from: az-d0w1
verify: "cargo test -p alpha-spill governor:: -- --test-threads=1"
created: 2026-08-03
---
Found while wiring telemetry; provenance recorded (MW-E4).

## log
- 2026-08-05T14:03Z close attempt — verify exit 1
