---
id: ax-un10
title: Union-poisoned status
status: doing
status: blocked
verify: true
created: 2026-08-01
---
Two clones edited the same status line; merge=union kept both (duplicate
key). Strict parse rejects; lint --fix repairs (MW-I1/I2).
