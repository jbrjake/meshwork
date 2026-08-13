---
id: mw-0y66mhb
title: Split parse.rs below the 500-line target (grammar/hash out)
status: open
category: core/arch
verify: out=$(./scripts/smoke.sh 2>&1) && ! echo "$out" | grep -q "src/parse.rs"
created: 2026-08-07T03:06Z
---

## log
- 2026-08-07T03:06Z created
