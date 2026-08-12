---
id: mw-p6atpxh
title: prime names the invalid files it counts
status: open
category: product/prime
verify: grep -rq prime_names_invalid tests/ && cargo test --quiet prime_names_invalid
created: 2026-08-12T20:48Z
---
prime's header said "1 invalid" for two full sazed sessions
(4b5a9264, f6e7cfbc) and nobody acted: the count names no id, no path,
no next step, while the broken task — seq 6, near the top of the
queue — silently vanished from ready and q. Diagnosis finally came
from an unrelated lint run. When the invalid count is nonzero, prime
should spend the bytes to name each file (id-or-path) and say "run
lint": an unreadable task is exactly the loud-row case FORMAT.md
already mandates for listings.

## log
- 2026-08-12T20:48Z created
