---
id: mw-0y66mhb
title: Split parse.rs below the 500-line target (grammar/hash out)
status: done
category: core/arch
verify: out=$(./scripts/smoke.sh 2>&1) && ! echo "$out" | grep -q "src/parse.rs"
created: 2026-08-07T03:06Z
seq: 350
---

## log
- 2026-08-07T03:06Z created
- 2026-08-23T19:55Z open→doing — claimed by claude (session_016iEafFdzwyKAtsU3AEMhaU)
- 2026-08-23T20:15Z doing→done — verify exit 0 @ 783776b+6
