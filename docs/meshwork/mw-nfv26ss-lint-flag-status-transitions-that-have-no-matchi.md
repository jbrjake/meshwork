---
id: mw-nfv26ss
title: "lint: flag status transitions that have no matching log entry"
status: open
category: core/hygiene
verify: run cargo test status_flip_without_log
relates:
  - mw-1byhnj1
  - mw-efmgn6b
created: 2026-08-12T20:48Z
---
A sazed session bypassed the CLI entirely: flipped `status: open` →
`done` with a raw Edit (no verify ran, no close anchor written) and
hand-minted a task file with an invented id (1dc9fa1f 22:58). The
store cannot prevent hand edits — they are legal — but lint can catch
the signature: a terminal status whose `## log` has no corresponding
transition entry. That distinguishes "hand-flipped, verify never ran"
from a real close, and gives the skill's close-only rule a
deterministic backstop.
