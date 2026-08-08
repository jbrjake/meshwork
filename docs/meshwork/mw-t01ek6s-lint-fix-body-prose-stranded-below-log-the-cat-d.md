---
id: mw-t01ek6s
title: "lint --fix: body prose stranded below ## log — the cat >> damage class"
category: core/store
relates: [mw-efmgn6b, mw-3wnhhvp]
verify: cargo test lint::stray_prose_below_log
seq: 310
docs:
  - FORMAT.md#tail-section-grammars
status: open
created: 2026-08-08T16:42Z
---
Field evidence (sazed, 2026-08-08): agents extend task descriptions with
`cat >> file.md`, which appends BELOW the tail sections. 8 of ~40
post-migration task files accumulated body prose under `## log` within
two days, and a session hand-rolled a bulk python repair (moved the
prose above the tail, then verified nothing was lost by comparing
sorted-line digests). The damage is mechanical and the repair is
deterministic: prose after the trailing run of `- ` log lines belongs to
the body. Detect it in lint, relocate it in lint --fix, per the FORMAT
tail-section grammar.
